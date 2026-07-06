//! Adapts raw probe output into the locked `DevSnapshot` contract
//! (feature 6). Grouping, framework labels, richer exposure, conflicts,
//! and stale hints are later features; their fields stay stubbed.

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::probe::{ListeningSocket, ProbeError, ProbeOutput, ProcessInfo, platform_probe};
use crate::snapshot::{DevSnapshot, Exposure, Service};

/// Probe this machine and adapt the result. A platform without a probe
/// yields an empty snapshot, not an error.
pub fn live_snapshot() -> Result<DevSnapshot, ProbeError> {
    let output = match platform_probe() {
        Some(probe) => probe.probe()?,
        None => ProbeOutput::default(),
    };
    Ok(from_probe(output))
}

fn from_probe(output: ProbeOutput) -> DevSnapshot {
    DevSnapshot {
        generated_at: now_ms(),
        services: services_from(output.sockets),
        projects: Vec::new(),
        conflicts: Vec::new(),
        docker_hints: Vec::new(),
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn services_from(sockets: Vec<ListeningSocket>) -> Vec<Service> {
    let merged = merge_sockets(sockets);
    let ids = assign_ids(&merged);
    merged
        .into_iter()
        .zip(ids)
        .map(|(socket, id)| service_from(socket, id))
        .collect()
}

/// One service per (port, owner); dual-stack v4+v6 listeners collapse here.
struct MergedSocket {
    port: u16,
    pid: Option<u32>,
    addrs: Vec<IpAddr>,
    process: Option<ProcessInfo>,
}

fn merge_sockets(sockets: Vec<ListeningSocket>) -> Vec<MergedSocket> {
    let mut merged: Vec<MergedSocket> = Vec::new();
    let mut index: HashMap<(u16, Option<u32>), usize> = HashMap::new();

    for socket in sockets {
        match index.get(&(socket.port, socket.pid)) {
            Some(&i) => {
                let group = &mut merged[i];
                group.addrs.push(socket.local_addr);
                if group.process.is_none() {
                    group.process = socket.process;
                }
            }
            None => {
                index.insert((socket.port, socket.pid), merged.len());
                merged.push(MergedSocket {
                    port: socket.port,
                    pid: socket.pid,
                    addrs: vec![socket.local_addr],
                    process: socket.process,
                });
            }
        }
    }

    merged.sort_by_key(|m| m.port);
    merged
}

/// `svc-{port}-{slug}`, suffixed with the pid only when two services on the
/// same port share a slug (SO_REUSEPORT and friends).
fn assign_ids(merged: &[MergedSocket]) -> Vec<String> {
    let slugs: Vec<String> = merged
        .iter()
        .map(|m| slug(m.process.as_ref().and_then(|p| p.name.as_deref())))
        .collect();

    let mut counts: HashMap<(u16, &str), u32> = HashMap::new();
    for (m, slug) in merged.iter().zip(&slugs) {
        *counts.entry((m.port, slug)).or_default() += 1;
    }

    merged
        .iter()
        .zip(&slugs)
        .map(
            |(m, slug)| match (counts[&(m.port, slug.as_str())] > 1, m.pid) {
                (true, Some(pid)) => format!("svc-{}-{slug}-{pid}", m.port),
                _ => format!("svc-{}-{slug}", m.port),
            },
        )
        .collect()
}

fn slug(name: Option<&str>) -> String {
    let cleaned = name
        .map(|n| {
            n.to_lowercase()
                .split(|c: char| !c.is_ascii_alphanumeric())
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join("-")
        })
        .unwrap_or_default();

    if cleaned.is_empty() {
        "unknown".into()
    } else {
        cleaned
    }
}

fn exposure(addrs: &[IpAddr]) -> Exposure {
    if addrs.iter().all(|a| a.to_canonical().is_loopback()) {
        Exposure::Local
    } else {
        Exposure::Unknown
    }
}

fn humanize_age(secs: u64) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;
    match secs {
        s if s < MINUTE => format!("{s}s"),
        s if s < HOUR => format!("{}m", s / MINUTE),
        s if s < DAY => format!("{}h", s / HOUR),
        s => format!("{}d", s / DAY),
    }
}

/// localhost when the service is reachable there (loopback or wildcard
/// bind), otherwise the literal bind address. Feature 9 refines this to
/// HTTP-looking services only.
fn url(addrs: &[IpAddr], port: u16) -> String {
    let localhost_reachable = addrs.iter().any(|a| {
        let a = a.to_canonical();
        a.is_loopback() || a.is_unspecified()
    });
    if localhost_reachable {
        return format!("http://localhost:{port}");
    }
    match addrs.first().map(|a| a.to_canonical()) {
        Some(IpAddr::V6(v6)) => format!("http://[{v6}]:{port}"),
        Some(IpAddr::V4(v4)) => format!("http://{v4}:{port}"),
        None => format!("http://localhost:{port}"),
    }
}

fn service_from(merged: MergedSocket, id: String) -> Service {
    let exposure = exposure(&merged.addrs);
    let url = url(&merged.addrs, merged.port);
    let process = merged.process;
    Service {
        id,
        port: merged.port,
        pid: merged.pid,
        process_name: process.as_ref().and_then(|p| p.name.clone()),
        command: process.as_ref().and_then(|p| p.command.clone()),
        cwd: process
            .as_ref()
            .and_then(|p| p.cwd.as_ref().map(|c| c.to_string_lossy().into_owned())),
        user: process.as_ref().and_then(|p| p.user.clone()),
        project_id: None,
        framework: None,
        exposure,
        url: Some(url),
        started_age: process
            .as_ref()
            .and_then(|p| p.started_secs_ago)
            .map(humanize_age),
        stale: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::Protocol;

    fn proc_info(pid: u32, name: &str) -> ProcessInfo {
        ProcessInfo {
            pid,
            name: Some(name.into()),
            command: Some(format!("{name} --serve")),
            cwd: Some("/home/brad/Code/app".into()),
            user: Some("brad".into()),
            started_secs_ago: Some(240),
        }
    }

    fn sock(
        addr: &str,
        port: u16,
        pid: Option<u32>,
        process: Option<ProcessInfo>,
    ) -> ListeningSocket {
        ListeningSocket {
            protocol: Protocol::Tcp,
            local_addr: addr.parse().expect("test addr"),
            port,
            pid,
            process,
        }
    }

    #[test]
    fn dual_stack_listener_merges_into_one_service() {
        let services = services_from(vec![
            sock("127.0.0.1", 3000, Some(10), Some(proc_info(10, "node"))),
            sock("::1", 3000, Some(10), Some(proc_info(10, "node"))),
        ]);
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].id, "svc-3000-node");
        assert!(matches!(services[0].exposure, Exposure::Local));
    }

    #[test]
    fn unknown_owners_on_one_port_merge_and_stay_unknown() {
        let services = services_from(vec![
            sock("0.0.0.0", 8080, None, None),
            sock("::", 8080, None, None),
        ]);
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].id, "svc-8080-unknown");
        assert!(services[0].pid.is_none());
        assert!(matches!(services[0].exposure, Exposure::Unknown));
    }

    #[test]
    fn different_pids_on_one_port_stay_separate_with_pid_suffixed_ids() {
        let services = services_from(vec![
            sock("127.0.0.1", 3000, Some(10), Some(proc_info(10, "node"))),
            sock("127.0.0.1", 3000, Some(20), Some(proc_info(20, "node"))),
        ]);
        assert_eq!(services.len(), 2);
        assert_eq!(services[0].id, "svc-3000-node-10");
        assert_eq!(services[1].id, "svc-3000-node-20");
    }

    #[test]
    fn same_name_on_different_ports_needs_no_suffix() {
        let services = services_from(vec![
            sock("127.0.0.1", 3000, Some(10), Some(proc_info(10, "node"))),
            sock("127.0.0.1", 3001, Some(20), Some(proc_info(20, "node"))),
        ]);
        assert_eq!(services[0].id, "svc-3000-node");
        assert_eq!(services[1].id, "svc-3001-node");
    }

    #[test]
    fn slug_sanitizes_names() {
        assert_eq!(slug(Some("Next-Server (v15)")), "next-server-v15");
        assert_eq!(slug(Some("___")), "unknown");
        assert_eq!(slug(Some("")), "unknown");
        assert_eq!(slug(None), "unknown");
    }

    #[test]
    fn mixed_loopback_and_lan_binds_are_not_local() {
        let services = services_from(vec![
            sock("127.0.0.1", 5173, Some(10), Some(proc_info(10, "node"))),
            sock("192.168.1.5", 5173, Some(10), None),
        ]);
        assert_eq!(services.len(), 1);
        assert!(matches!(services[0].exposure, Exposure::Unknown));
    }

    #[test]
    fn process_metadata_maps_through_and_fills_from_any_socket_in_the_group() {
        // first socket lacks process info; the merge keeps the second's
        let services = services_from(vec![
            sock("::1", 5432, Some(42), None),
            sock("127.0.0.1", 5432, Some(42), Some(proc_info(42, "postgres"))),
        ]);
        let svc = &services[0];
        assert_eq!(svc.process_name.as_deref(), Some("postgres"));
        assert_eq!(svc.command.as_deref(), Some("postgres --serve"));
        assert_eq!(svc.cwd.as_deref(), Some("/home/brad/Code/app"));
        assert_eq!(svc.user.as_deref(), Some("brad"));
        assert_eq!(svc.id, "svc-5432-postgres");
    }

    #[test]
    fn humanize_age_picks_the_right_unit_at_boundaries() {
        assert_eq!(humanize_age(0), "0s");
        assert_eq!(humanize_age(59), "59s");
        assert_eq!(humanize_age(60), "1m");
        assert_eq!(humanize_age(240), "4m");
        assert_eq!(humanize_age(3599), "59m");
        assert_eq!(humanize_age(3600), "1h");
        assert_eq!(humanize_age(86399), "23h");
        assert_eq!(humanize_age(86400), "1d");
        assert_eq!(humanize_age(6 * 86400), "6d");
    }

    #[test]
    fn url_uses_localhost_for_loopback_and_wildcard_binds() {
        let loopback = ["127.0.0.1".parse().expect("addr")];
        assert_eq!(url(&loopback, 3000), "http://localhost:3000");
        let wildcard = ["0.0.0.0".parse().expect("addr")];
        assert_eq!(url(&wildcard, 5173), "http://localhost:5173");
        let v6_wildcard = ["::".parse().expect("addr")];
        assert_eq!(url(&v6_wildcard, 8080), "http://localhost:8080");
    }

    #[test]
    fn url_uses_the_literal_address_for_specific_non_loopback_binds() {
        let lan_v4 = ["192.168.1.5".parse().expect("addr")];
        assert_eq!(url(&lan_v4, 5173), "http://192.168.1.5:5173");
        let lan_v6 = ["fe80::1".parse().expect("addr")];
        assert_eq!(url(&lan_v6, 5173), "http://[fe80::1]:5173");
    }

    #[test]
    fn services_carry_url_and_humanized_age() {
        let services = services_from(vec![sock(
            "127.0.0.1",
            3000,
            Some(10),
            Some(proc_info(10, "node")),
        )]);
        assert_eq!(services[0].url.as_deref(), Some("http://localhost:3000"));
        assert_eq!(services[0].started_age.as_deref(), Some("4m"));
    }

    #[test]
    fn services_come_out_sorted_by_port() {
        let services = services_from(vec![
            sock("127.0.0.1", 9000, None, None),
            sock("127.0.0.1", 80, None, None),
            sock("127.0.0.1", 443, None, None),
        ]);
        let ports: Vec<u16> = services.iter().map(|s| s.port).collect();
        assert_eq!(ports, vec![80, 443, 9000]);
    }
}
