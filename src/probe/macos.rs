use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use netstat2::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind, Users};

use super::{ListeningSocket, Probe, ProbeError, ProbeOutput, ProcessInfo, Protocol};

/// macOS probe backed by netstat2 for the socket table (kernel pcb list via
/// sysctl, visible for all users without root) and sysinfo for process
/// metadata on the pids netstat2 could join.
pub struct MacProbe;

impl Probe for MacProbe {
    fn name(&self) -> &'static str {
        "macos-netstat"
    }

    fn probe(&self) -> Result<ProbeOutput, ProbeError> {
        let mut sockets = listening_sockets()?;
        sockets.sort_by_key(|s| s.port);
        Ok(ProbeOutput { sockets })
    }
}

struct RawListener {
    local_addr: std::net::IpAddr,
    port: u16,
    pid: Option<u32>,
}

fn listening_sockets() -> Result<Vec<ListeningSocket>, ProbeError> {
    let raw = raw_listeners()?;

    let pids: Vec<Pid> = raw
        .iter()
        .filter_map(|l| l.pid)
        .map(Pid::from_u32)
        .collect();
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&pids),
        false,
        ProcessRefreshKind::nothing()
            .with_cmd(UpdateKind::Always)
            .with_cwd(UpdateKind::Always)
            .with_user(UpdateKind::Always),
    );
    let users = Users::new_with_refreshed_list();
    let now = now_epoch_secs();
    let mut info_cache: HashMap<u32, Option<ProcessInfo>> = HashMap::new();

    Ok(raw
        .into_iter()
        .map(|listener| {
            let process = listener.pid.and_then(|p| {
                info_cache
                    .entry(p)
                    .or_insert_with(|| process_info(p, &system, &users, now))
                    .clone()
            });
            ListeningSocket {
                protocol: Protocol::Tcp,
                local_addr: listener.local_addr,
                port: listener.port,
                pid: listener.pid,
                // netstat2 does not expose the socket owner uid; when the pid
                // join fails the owner stays honestly unknown, when it works
                // the process user is the owner.
                uid: None,
                user: process.as_ref().and_then(|p| p.user.clone()),
                process,
            }
        })
        .collect())
}

fn raw_listeners() -> Result<Vec<RawListener>, ProbeError> {
    let families = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let infos =
        netstat2::get_sockets_info(families, ProtocolFlags::TCP).map_err(|e| ProbeError::Io {
            path: "net.inet.tcp.pcblist_n".to_string(),
            source: std::io::Error::other(e.to_string()),
        })?;

    Ok(infos
        .into_iter()
        .filter_map(|info| {
            let ProtocolSocketInfo::Tcp(tcp) = &info.protocol_socket_info else {
                return None;
            };
            if tcp.state != TcpState::Listen {
                return None;
            }
            Some(RawListener {
                local_addr: tcp.local_addr,
                port: tcp.local_port,
                pid: info.associated_pids.first().copied(),
            })
        })
        .collect())
}

/// Best-effort process metadata; any unreadable piece degrades to None.
fn process_info(pid: u32, system: &System, users: &Users, now: u64) -> Option<ProcessInfo> {
    let process = system.process(Pid::from_u32(pid))?;
    let cmd: Vec<String> = process
        .cmd()
        .iter()
        .map(|part| part.to_string_lossy().into_owned())
        .collect();

    Some(ProcessInfo {
        pid,
        name: expand_name(process.name().to_str(), cmd.first().map(String::as_str)),
        command: (!cmd.is_empty()).then(|| cmd.join(" ")),
        cwd: process.cwd().map(std::path::Path::to_path_buf),
        user: process
            .user_id()
            .and_then(|uid| users.get_user_by_id(uid))
            .map(|user| user.name().to_string()),
        started_secs_ago: secs_ago(now, process.start_time()),
    })
}

/// The kernel caps p_comm at 16 bytes, so sysinfo's name can arrive
/// truncated; expand it from the command's executable basename when that is
/// clearly the untruncated form, otherwise keep sysinfo's name.
fn expand_name(name: Option<&str>, first_arg: Option<&str>) -> Option<String> {
    let name = name.filter(|n| !n.trim().is_empty());
    let token = first_arg.map(name_token).filter(|t| !t.trim().is_empty());
    match (name, token) {
        (Some(name), Some(token)) if token.starts_with(name) && token.len() > name.len() => {
            Some(token.to_string())
        }
        (Some(name), _) => Some(name.to_string()),
        (None, token) => token.map(str::to_string),
    }
}

/// Path basename of the command's first whitespace token
/// ("/usr/local/bin/node --flag" -> "node").
fn name_token(first_arg: &str) -> &str {
    let token = first_arg.split_whitespace().next().unwrap_or(first_arg);
    token.rsplit('/').next().unwrap_or(token)
}

fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A zero start time means sysinfo could not read it; future times clamp.
fn secs_ago(now: u64, start_epoch: u64) -> Option<u64> {
    (start_epoch > 0).then(|| now.saturating_sub(start_epoch))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_sees_own_listener() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind test listener");
        let port = listener.local_addr().expect("local addr").port();

        let output = MacProbe.probe().expect("probe should succeed");

        let own = output
            .sockets
            .iter()
            .find(|s| s.port == port)
            .unwrap_or_else(|| panic!("probe should see the test listener on port {port}"));
        assert_eq!(
            own.pid,
            Some(std::process::id()),
            "own listener should be joined to our pid"
        );

        let info = own
            .process
            .as_ref()
            .expect("own process should have metadata");
        assert!(info.name.is_some(), "own process should have a name");
        let command = info
            .command
            .as_deref()
            .expect("own process should have a command");
        assert!(
            command.contains("portdoc"),
            "test binary command should mention portdoc, got: {command}"
        );
        assert!(info.cwd.is_some(), "own process cwd should be readable");
        assert!(info.user.is_some(), "own process user should resolve");
        assert_eq!(
            own.user, info.user,
            "socket user and process user should agree for our own listener"
        );
        assert!(
            info.started_secs_ago.is_some(),
            "own process age should compute"
        );

        eprintln!(
            "probe found {} listening sockets on this machine",
            output.sockets.len()
        );
    }

    #[test]
    fn expand_name_grows_truncated_names_from_the_command() {
        assert_eq!(
            expand_name(
                Some("com.docker.backe"),
                Some("/Applications/com.docker.backend")
            )
            .as_deref(),
            Some("com.docker.backend")
        );
        assert_eq!(
            expand_name(Some("node"), Some("/usr/local/bin/node --flag")).as_deref(),
            Some("node")
        );
    }

    #[test]
    fn expand_name_keeps_sysinfo_name_when_the_command_disagrees() {
        assert_eq!(
            expand_name(Some("next-server"), Some("/usr/local/bin/node")).as_deref(),
            Some("next-server"),
            "a rewritten process title beats the interpreter basename"
        );
    }

    #[test]
    fn expand_name_handles_missing_sides() {
        assert_eq!(
            expand_name(None, Some("/bin/thing --flag")).as_deref(),
            Some("thing")
        );
        assert_eq!(expand_name(Some("node"), None).as_deref(), Some("node"));
        assert_eq!(expand_name(None, None), None);
        assert_eq!(expand_name(Some("  "), Some("")), None);
    }

    #[test]
    fn secs_ago_handles_edges() {
        assert_eq!(secs_ago(1_000, 400), Some(600));
        assert_eq!(secs_ago(1_000, 0), None, "unreadable start time stays None");
        assert_eq!(secs_ago(10, 5_000), Some(0), "future start clamps to zero");
    }
}
