//! Advanced-tab facts: the raw pre-merge socket list, well-known-port hints,
//! and honest unknown-owner diagnostics. Everything here ships on
//! /api/sockets, never inside the locked DevSnapshot.

use serde::Serialize;

use crate::probe::{ListeningSocket, ProbeError, ProbeOutput, Protocol, platform_probe};

#[derive(Serialize)]
pub struct SocketsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probe: Option<&'static str>,
    pub sockets: Vec<SocketDetail>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Serialize)]
pub struct SocketDetail {
    pub protocol: &'static str,
    pub local_addr: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

#[derive(Serialize)]
pub struct Diagnostic {
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<&'static str>,
    pub reason: String,
}

pub fn live_sockets() -> Result<SocketsResponse, ProbeError> {
    let (probe_name, output) = match platform_probe() {
        Some(probe) => (Some(probe.name()), probe.probe()?),
        None => (None, ProbeOutput::default()),
    };
    Ok(sockets_response(probe_name, output, &running_user()))
}

fn running_user() -> String {
    std::env::var("USER").unwrap_or_else(|_| "the current user".into())
}

pub fn sockets_response(
    probe: Option<&'static str>,
    output: ProbeOutput,
    running_user: &str,
) -> SocketsResponse {
    let diagnostics = diagnostics(&output.sockets, running_user);
    let sockets = output.sockets.into_iter().map(socket_detail).collect();
    SocketsResponse {
        probe,
        sockets,
        diagnostics,
    }
}

fn socket_detail(socket: ListeningSocket) -> SocketDetail {
    SocketDetail {
        protocol: match socket.protocol {
            Protocol::Tcp => "tcp",
        },
        local_addr: socket.local_addr.to_string(),
        port: socket.port,
        pid: socket.pid,
        process_name: socket.process.and_then(|p| p.name),
        uid: socket.uid,
        user: socket.user,
    }
}

/// One entry per distinct unknown-owner port, in port order. Relies on the
/// probe's port-sorted output for the dedupe.
fn diagnostics(sockets: &[ListeningSocket], running_user: &str) -> Vec<Diagnostic> {
    let mut out: Vec<Diagnostic> = Vec::new();
    for socket in sockets.iter().filter(|s| s.pid.is_none()) {
        if out.last().is_some_and(|d| d.port == socket.port) {
            continue;
        }
        out.push(Diagnostic {
            port: socket.port,
            hint: well_known(socket.port),
            reason: unknown_owner_reason(socket.uid, socket.user.as_deref(), running_user),
        });
    }
    out
}

const WELL_KNOWN: &[(u16, &str)] = &[
    (22, "usually SSH"),
    (25, "usually SMTP mail"),
    (53, "usually DNS"),
    (80, "usually HTTP"),
    (111, "usually rpcbind/NFS"),
    (443, "usually HTTPS"),
    (587, "usually SMTP mail submission"),
    (631, "usually printing (IPP/CUPS)"),
    (3306, "usually MySQL"),
    (5353, "usually mDNS"),
    (5432, "usually Postgres"),
    (6379, "usually Redis"),
    (27017, "usually MongoDB"),
];

pub fn well_known(port: u16) -> Option<&'static str> {
    WELL_KNOWN
        .iter()
        .find_map(|&(p, hint)| (p == port).then_some(hint))
}

/// Why the pid join failed, from the one fact the kernel still gives us:
/// the socket owner's uid. Never speculates past what is knowable.
fn unknown_owner_reason(uid: Option<u32>, user: Option<&str>, running_user: &str) -> String {
    match (uid, user) {
        (Some(0), _) => format!(
            "Owned by root - PortDoc runs as {running_user} and cannot read root-owned process details"
        ),
        (Some(_), Some(user)) if user == running_user => format!(
            "Owned by {running_user} but no owning process was found - it may have exited during the scan"
        ),
        (Some(uid), Some(user)) => format!(
            "Owned by {user} (uid {uid}) - PortDoc runs as {running_user} and cannot read other users' process details"
        ),
        (Some(uid), None) => format!(
            "Owned by uid {uid} - PortDoc runs as {running_user} and cannot read other users' process details"
        ),
        (None, _) => "The socket owner could not be determined from the kernel table".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sock(port: u16, pid: Option<u32>, uid: Option<u32>, user: Option<&str>) -> ListeningSocket {
        ListeningSocket {
            protocol: Protocol::Tcp,
            local_addr: "127.0.0.1".parse().expect("test addr"),
            port,
            pid,
            process: None,
            uid,
            user: user.map(Into::into),
        }
    }

    #[test]
    fn well_known_covers_the_classics_and_nothing_else() {
        assert_eq!(well_known(22), Some("usually SSH"));
        assert_eq!(well_known(631), Some("usually printing (IPP/CUPS)"));
        assert_eq!(well_known(5432), Some("usually Postgres"));
        assert_eq!(well_known(3000), None, "dev ports get no generic label");
    }

    #[test]
    fn reasons_name_the_owner_honestly() {
        let root = unknown_owner_reason(Some(0), Some("root"), "brad");
        assert!(root.contains("root") && root.contains("brad"), "{root}");

        let other = unknown_owner_reason(Some(112), Some("gdm"), "brad");
        assert!(other.contains("gdm") && other.contains("112"), "{other}");

        let own = unknown_owner_reason(Some(1000), Some("brad"), "brad");
        assert!(own.contains("may have exited"), "{own}");

        let unresolved = unknown_owner_reason(Some(4242), None, "brad");
        assert!(unresolved.contains("uid 4242"), "{unresolved}");

        let unknown = unknown_owner_reason(None, None, "brad");
        assert!(unknown.contains("could not be determined"), "{unknown}");
    }

    #[test]
    fn diagnostics_cover_only_unknown_owners_deduped_per_port() {
        let sockets = [
            sock(22, None, Some(0), Some("root")),
            sock(22, None, Some(0), Some("root")), // v6 twin
            sock(3000, Some(10), Some(1000), Some("brad")),
            sock(5432, None, Some(0), Some("root")),
        ];
        let diags = diagnostics(&sockets, "brad");
        let ports: Vec<u16> = diags.iter().map(|d| d.port).collect();
        assert_eq!(ports, vec![22, 5432]);
        assert_eq!(diags[0].hint, Some("usually SSH"));
        assert_eq!(diags[1].hint, Some("usually Postgres"));
        assert!(diags[0].reason.contains("root"));
    }

    #[test]
    fn response_maps_sockets_through_and_keeps_the_probe_name() {
        let output = ProbeOutput {
            sockets: vec![
                sock(22, None, Some(0), Some("root")),
                sock(3000, Some(10), Some(1000), Some("brad")),
            ],
        };
        let response = sockets_response(Some("linux-proc"), output, "brad");
        assert_eq!(response.probe, Some("linux-proc"));
        assert_eq!(response.sockets.len(), 2);
        assert_eq!(response.sockets[0].protocol, "tcp");
        assert_eq!(response.sockets[0].local_addr, "127.0.0.1");
        assert_eq!(response.sockets[0].user.as_deref(), Some("root"));
        assert_eq!(response.diagnostics.len(), 1);

        let empty = sockets_response(None, ProbeOutput::default(), "brad");
        assert!(empty.probe.is_none());
        assert!(empty.sockets.is_empty() && empty.diagnostics.is_empty());
    }
}
