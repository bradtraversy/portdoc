use std::collections::HashMap;

use procfs::Current;
use procfs::net::TcpState;
use procfs::process::FDTarget;

use super::{ListeningSocket, Probe, ProbeError, ProbeOutput, ProcessInfo, Protocol};

/// Linux probe backed by /proc: listening TCP sockets from /proc/net/tcp{,6}.
pub struct LinuxProbe;

impl Probe for LinuxProbe {
    fn name(&self) -> &'static str {
        "linux-proc"
    }

    fn probe(&self) -> Result<ProbeOutput, ProbeError> {
        let mut sockets = listening_sockets()?;
        sockets.sort_by_key(|s| s.port);
        Ok(ProbeOutput { sockets })
    }
}

fn listening_sockets() -> Result<Vec<ListeningSocket>, ProbeError> {
    let v4 = procfs::net::tcp().map_err(|e| proc_err("/proc/net/tcp", e))?;
    let v6 = procfs::net::tcp6().map_err(|e| proc_err("/proc/net/tcp6", e))?;
    let owners = socket_owners();
    let host = HostContext::read();
    let mut info_cache: HashMap<u32, Option<ProcessInfo>> = HashMap::new();

    Ok(v4
        .into_iter()
        .chain(v6)
        .filter(|entry| entry.state == TcpState::Listen)
        .map(|entry| {
            let pid = owners.get(&entry.inode).copied();
            let process = pid.and_then(|p| {
                info_cache
                    .entry(p)
                    .or_insert_with(|| process_info(p, &host))
                    .clone()
            });
            ListeningSocket {
                protocol: Protocol::Tcp,
                local_addr: entry.local_address.ip(),
                port: entry.local_address.port(),
                pid,
                process,
                uid: Some(entry.uid),
                user: user_for_uid(entry.uid, &host.passwd),
            }
        })
        .collect())
}

/// Host facts read once per probe instead of once per process.
struct HostContext {
    passwd: String,
    uptime_secs: Option<f64>,
    ticks_per_second: u64,
}

impl HostContext {
    fn read() -> Self {
        Self {
            passwd: std::fs::read_to_string("/etc/passwd").unwrap_or_default(),
            uptime_secs: procfs::Uptime::current().ok().map(|u| u.uptime),
            ticks_per_second: procfs::ticks_per_second(),
        }
    }
}

/// Best-effort process metadata; any unreadable piece degrades to None.
fn process_info(pid: u32, host: &HostContext) -> Option<ProcessInfo> {
    let process = procfs::process::Process::new(pid as i32).ok()?;
    let stat = process.stat().ok();
    let cmdline = process.cmdline().ok().filter(|c| !c.is_empty());

    Some(ProcessInfo {
        pid,
        name: best_name(
            stat.as_ref().map(|s| s.comm.as_str()),
            cmdline.as_ref().and_then(|c| c.first()).map(String::as_str),
        ),
        command: cmdline.as_ref().map(|c| c.join(" ")),
        cwd: process.cwd().ok(),
        user: process
            .uid()
            .ok()
            .and_then(|uid| user_for_uid(uid, &host.passwd)),
        started_secs_ago: match (&stat, host.uptime_secs) {
            (Some(s), Some(uptime)) => Some(secs_ago(uptime, s.starttime, host.ticks_per_second)),
            _ => None,
        },
    })
}

/// The kernel caps comm at 15 bytes and threads may rename it (node's
/// "MainThread"); prefer the process title when it is clearly the better
/// truth, otherwise keep comm.
fn best_name(comm: Option<&str>, first_arg: Option<&str>) -> Option<String> {
    let first_arg = first_arg.filter(|a| !a.trim().is_empty());
    let Some(comm) = comm.filter(|c| !c.is_empty()) else {
        return first_arg.map(|a| name_token(a).to_string());
    };
    let Some(arg) = first_arg else {
        return Some(comm.to_string());
    };
    let token = name_token(arg);

    const COMM_CAP: usize = 15;
    if comm.len() == COMM_CAP {
        // truncated comm: expand from the title or the executable basename
        if arg.starts_with(comm) {
            return Some(arg.to_string());
        }
        if token.starts_with(comm) {
            return Some(token.to_string());
        }
    }
    if !token.starts_with(comm) && !comm.starts_with(token) {
        return Some(token.to_string());
    }
    Some(comm.to_string())
}

/// Path basename of the title's first whitespace token
/// ("/usr/bin/node --flag" -> "node", "sshd: brad@pts/0" -> "sshd:").
fn name_token(first_arg: &str) -> &str {
    let token = first_arg.split_whitespace().next().unwrap_or(first_arg);
    token.rsplit('/').next().unwrap_or(token)
}

fn user_for_uid(uid: u32, passwd: &str) -> Option<String> {
    passwd.lines().find_map(|line| {
        let mut fields = line.split(':');
        let name = fields.next()?;
        let _password = fields.next()?;
        let line_uid: u32 = fields.next()?.parse().ok()?;
        (line_uid == uid).then(|| name.to_string())
    })
}

fn secs_ago(uptime_secs: f64, starttime_ticks: u64, ticks_per_second: u64) -> u64 {
    if ticks_per_second == 0 {
        return 0;
    }
    let started = starttime_ticks as f64 / ticks_per_second as f64;
    (uptime_secs - started).max(0.0) as u64
}

/// Map socket inodes to owning PIDs by scanning readable fd tables.
/// Processes we can't read (other users, exited mid-scan) are skipped;
/// their sockets simply stay unowned.
fn socket_owners() -> HashMap<u64, u32> {
    let mut owners = HashMap::new();
    let Ok(processes) = procfs::process::all_processes() else {
        return owners;
    };
    for process in processes.flatten() {
        let Ok(fds) = process.fd() else { continue };
        for fd in fds.flatten() {
            if let FDTarget::Socket(inode) = fd.target {
                owners.insert(inode, process.pid as u32);
            }
        }
    }
    owners
}

fn proc_err(path: &str, err: procfs::ProcError) -> ProbeError {
    let source = match err {
        procfs::ProcError::Io(io, _) => io,
        other => std::io::Error::other(other.to_string()),
    };
    ProbeError::Io {
        path: path.to_string(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_sees_own_listener() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind test listener");
        let port = listener.local_addr().expect("local addr").port();

        let output = LinuxProbe.probe().expect("probe should succeed");

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
        let my_uid = procfs::process::Process::myself()
            .and_then(|p| p.uid())
            .expect("own uid should be readable");
        assert_eq!(
            own.uid,
            Some(my_uid),
            "own socket should carry our uid from the kernel table"
        );
        assert_eq!(
            own.user, info.user,
            "socket owner and process user should agree for our own listener"
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
    fn best_name_expands_truncated_comm_from_the_title() {
        assert_eq!(
            best_name(Some("next-server (v1"), Some("next-server (v16.2.9)")).as_deref(),
            Some("next-server (v16.2.9)")
        );
        assert_eq!(
            best_name(
                Some("containerd-shim"),
                Some("/usr/bin/containerd-shim-runc-v2")
            )
            .as_deref(),
            Some("containerd-shim-runc-v2")
        );
    }

    #[test]
    fn best_name_replaces_renamed_thread_labels_with_the_executable() {
        assert_eq!(
            best_name(Some("MainThread"), Some("/usr/bin/node")).as_deref(),
            Some("node")
        );
    }

    #[test]
    fn best_name_keeps_comm_when_it_matches_the_title() {
        assert_eq!(
            best_name(Some("node"), Some("node")).as_deref(),
            Some("node")
        );
        assert_eq!(
            best_name(Some("sshd"), Some("sshd: brad@pts/0")).as_deref(),
            Some("sshd"),
            "rewritten daemon titles keep the comm"
        );
        assert_eq!(
            best_name(Some("python3"), Some("/usr/bin/python3.12")).as_deref(),
            Some("python3")
        );
    }

    #[test]
    fn best_name_handles_missing_sides() {
        assert_eq!(
            best_name(Some("kworker/0:1"), None).as_deref(),
            Some("kworker/0:1")
        );
        assert_eq!(
            best_name(None, Some("/bin/thing --flag")).as_deref(),
            Some("thing")
        );
        assert_eq!(best_name(None, None), None);
    }

    #[test]
    fn user_for_uid_resolves_and_skips_malformed() {
        let passwd = "root:x:0:0:root:/root:/bin/bash\nbroken-line-no-fields\nbrad:x:1000:1000:Brad:/home/brad:/bin/bash\n";
        assert_eq!(user_for_uid(0, passwd).as_deref(), Some("root"));
        assert_eq!(user_for_uid(1000, passwd).as_deref(), Some("brad"));
        assert_eq!(user_for_uid(4242, passwd), None);
        assert_eq!(user_for_uid(0, ""), None);
    }

    #[test]
    fn secs_ago_handles_edges() {
        assert_eq!(secs_ago(1000.0, 50_000, 100), 500);
        assert_eq!(
            secs_ago(1000.0, 0, 0),
            0,
            "zero ticks per second must not divide"
        );
        assert_eq!(
            secs_ago(10.0, 5_000, 100),
            0,
            "future starttime clamps to zero"
        );
    }
}
