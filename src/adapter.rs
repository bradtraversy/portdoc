//! Adapts raw probe output into the locked `DevSnapshot` contract
//! (feature 6). Grouping, framework labels, richer exposure, conflicts,
//! and stale hints are later features; their fields stay stubbed.

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::label::{ProjectLabels, detect_framework, http_looking, is_dev_server, project_labels};
use crate::probe::{ListeningSocket, ProbeError, ProbeOutput, ProcessInfo, platform_probe};
use crate::project::{Marker, detect_root, fs_marker};
use crate::snapshot::{DevSnapshot, Exposure, ProjectGroup, Service, StaleHint};

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
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let mut services = services_from(output.sockets);
    let projects = group_projects(&mut services, home.as_deref(), fs_marker, project_labels);
    let conflicts = crate::hint::detect_conflicts(&services, &projects);
    DevSnapshot {
        generated_at: now_ms(),
        services,
        projects,
        conflicts,
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

/// Bind addresses answer reachability regardless of owner; docker-proxy
/// ownership is the one provable Docker signal without root.
fn exposure(addrs: &[IpAddr], process_name: Option<&str>) -> Exposure {
    if process_name == Some("docker-proxy") {
        return Exposure::Docker;
    }
    if addrs.is_empty() {
        return Exposure::Unknown;
    }
    if addrs.iter().all(|a| a.to_canonical().is_loopback()) {
        Exposure::Local
    } else {
        Exposure::Lan
    }
}

/// Detect each service's project root, emit one sorted `ProjectGroup` per
/// distinct root, and point member services at their group.
fn group_projects(
    services: &mut [Service],
    home: Option<&Path>,
    marker_at: impl Fn(&Path) -> Option<Marker>,
    labels_at: impl Fn(&Path) -> ProjectLabels,
) -> Vec<ProjectGroup> {
    let mut root_by_cwd: HashMap<String, Option<PathBuf>> = HashMap::new();
    let mut members_by_root: Vec<(PathBuf, Vec<usize>)> = Vec::new();
    let mut index_of_root: HashMap<PathBuf, usize> = HashMap::new();

    for (i, service) in services.iter().enumerate() {
        let Some(cwd) = service.cwd.as_deref() else {
            continue;
        };
        let root = root_by_cwd
            .entry(cwd.to_string())
            .or_insert_with(|| detect_root(Path::new(cwd), home, &marker_at))
            .clone();
        let Some(root) = root else { continue };
        match index_of_root.get(&root) {
            Some(&idx) => members_by_root[idx].1.push(i),
            None => {
                index_of_root.insert(root.clone(), members_by_root.len());
                members_by_root.push((root, vec![i]));
            }
        }
    }

    let ids = project_ids(&members_by_root);
    let mut groups: Vec<ProjectGroup> = members_by_root
        .iter()
        .zip(&ids)
        .map(|((root, members), id)| {
            for &i in members {
                services[i].project_id = Some(id.clone());
            }
            let labels = labels_at(root);
            ProjectGroup {
                id: id.clone(),
                name: basename(root),
                root: display_root(root, home),
                package_manager: labels.package_manager,
                git_branch: labels.git_branch,
                service_ids: members.iter().map(|&i| services[i].id.clone()).collect(),
            }
        })
        .collect();

    groups.sort_by(|a, b| a.name.cmp(&b.name));
    groups
}

/// `proj-{slug(basename)}`, parent-qualified only when two distinct roots
/// share a basename slug.
fn project_ids(members_by_root: &[(PathBuf, Vec<usize>)]) -> Vec<String> {
    let slugs: Vec<String> = members_by_root
        .iter()
        .map(|(root, _)| slug(Some(basename(root).as_str())))
        .collect();

    let mut counts: HashMap<&str, u32> = HashMap::new();
    for slug in &slugs {
        *counts.entry(slug).or_default() += 1;
    }

    members_by_root
        .iter()
        .zip(&slugs)
        .map(|((root, _), s)| {
            if counts[s.as_str()] > 1 {
                let parent = root
                    .parent()
                    .map(|p| slug(Some(basename(p).as_str())))
                    .unwrap_or_else(|| "root".into());
                format!("proj-{parent}-{s}")
            } else {
                format!("proj-{s}")
            }
        })
        .collect()
}

/// detect_root never returns a path without a final component.
fn basename(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn display_root(root: &Path, home: Option<&Path>) -> String {
    if let Some(home) = home
        && let Ok(rel) = root.strip_prefix(home)
    {
        return format!("~/{}", rel.display());
    }
    root.display().to_string()
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
    let process = merged.process;
    let name = process.as_ref().and_then(|p| p.name.as_deref());
    let exposure = exposure(&merged.addrs, name);
    let framework = detect_framework(name, process.as_ref().and_then(|p| p.command.as_deref()));
    let url = http_looking(merged.port, name, framework.as_deref())
        .then(|| url(&merged.addrs, merged.port));
    let started_secs = process.as_ref().and_then(|p| p.started_secs_ago);
    let stale = stale_hint(framework.as_deref(), started_secs);
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
        framework,
        exposure,
        url,
        started_age: started_secs.map(humanize_age),
        stale,
    }
}

const STALE_AFTER_SECS: u64 = 3 * 24 * 60 * 60;

/// Only known dev servers get accused, and only with the provable fact
/// (age) - never traffic claims we cannot observe.
fn stale_hint(framework: Option<&str>, started_secs_ago: Option<u64>) -> Option<StaleHint> {
    let framework = framework.filter(|f| is_dev_server(f))?;
    let secs = started_secs_ago.filter(|&s| s >= STALE_AFTER_SECS)?;
    Some(StaleHint {
        reason: format!("{framework} dev server running for {}", humanize_age(secs)),
    })
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
    fn unknown_owner_wildcard_binds_merge_and_read_as_lan() {
        let services = services_from(vec![
            sock("0.0.0.0", 8080, None, None),
            sock("::", 8080, None, None),
        ]);
        assert_eq!(services.len(), 1);
        assert_eq!(services[0].id, "svc-8080-unknown");
        assert!(services[0].pid.is_none());
        assert!(
            matches!(services[0].exposure, Exposure::Lan),
            "the bind address answers reachability even without an owner"
        );
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
    fn mixed_loopback_and_lan_binds_classify_lan() {
        let services = services_from(vec![
            sock("127.0.0.1", 5173, Some(10), Some(proc_info(10, "node"))),
            sock("192.168.1.5", 5173, Some(10), None),
        ]);
        assert_eq!(services.len(), 1);
        assert!(matches!(services[0].exposure, Exposure::Lan));
    }

    #[test]
    fn specific_lan_address_classifies_lan() {
        let services = services_from(vec![sock(
            "192.168.1.5",
            5173,
            Some(10),
            Some(proc_info(10, "node")),
        )]);
        assert!(matches!(services[0].exposure, Exposure::Lan));
    }

    #[test]
    fn docker_proxy_owner_classifies_docker_even_on_wildcard() {
        let services = services_from(vec![sock(
            "0.0.0.0",
            5432,
            Some(77),
            Some(proc_info(77, "docker-proxy")),
        )]);
        assert!(matches!(services[0].exposure, Exposure::Docker));
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

    fn proc_at(pid: u32, name: &str, cwd: &str) -> ProcessInfo {
        ProcessInfo {
            cwd: Some(cwd.into()),
            ..proc_info(pid, name)
        }
    }

    fn grouped(
        sockets: Vec<ListeningSocket>,
        markers: &[(&str, Marker)],
    ) -> (Vec<Service>, Vec<ProjectGroup>) {
        let map: HashMap<PathBuf, Marker> = markers
            .iter()
            .map(|(p, m)| (PathBuf::from(p), *m))
            .collect();
        let mut services = services_from(sockets);
        let projects = group_projects(
            &mut services,
            Some(Path::new("/home/brad")),
            |dir| map.get(dir).copied(),
            |_| ProjectLabels::default(),
        );
        (services, projects)
    }

    #[test]
    fn project_labels_flow_onto_groups() {
        let mut services = services_from(vec![sock(
            "127.0.0.1",
            3000,
            Some(10),
            Some(proc_at(10, "node", "/home/brad/Code/app")),
        )]);
        let projects = group_projects(
            &mut services,
            Some(Path::new("/home/brad")),
            |dir| (dir == Path::new("/home/brad/Code/app")).then_some(Marker::Repo),
            |_| ProjectLabels {
                package_manager: Some("npm".into()),
                git_branch: Some("main".into()),
            },
        );
        assert_eq!(projects[0].package_manager.as_deref(), Some("npm"));
        assert_eq!(projects[0].git_branch.as_deref(), Some("main"));
    }

    #[test]
    fn services_group_by_detected_root_sorted_by_name() {
        let (services, projects) = grouped(
            vec![
                sock(
                    "127.0.0.1",
                    3000,
                    Some(10),
                    Some(proc_at(10, "node", "/home/brad/Code/startdev")),
                ),
                sock(
                    "127.0.0.1",
                    3001,
                    Some(20),
                    Some(proc_at(20, "node", "/home/brad/Code/certificreate")),
                ),
                sock(
                    "127.0.0.1",
                    3002,
                    Some(30),
                    Some(proc_at(30, "node", "/home/brad/Code/startdev/apps/web")),
                ),
                sock("127.0.0.1", 8080, None, None),
                sock(
                    "127.0.0.1",
                    9000,
                    Some(40),
                    Some(proc_at(40, "misc", "/home/brad/Downloads")),
                ),
            ],
            &[
                ("/home/brad/Code/startdev", Marker::Repo),
                ("/home/brad/Code/certificreate", Marker::Repo),
            ],
        );

        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].name, "certificreate");
        assert_eq!(projects[0].id, "proj-certificreate");
        assert_eq!(projects[0].root, "~/Code/certificreate");
        assert_eq!(projects[1].name, "startdev");
        assert_eq!(projects[1].service_ids.len(), 2);

        let by_port: HashMap<u16, &Service> = services.iter().map(|s| (s.port, s)).collect();
        assert_eq!(by_port[&3000].project_id.as_deref(), Some("proj-startdev"));
        assert_eq!(by_port[&3002].project_id.as_deref(), Some("proj-startdev"));
        assert_eq!(
            by_port[&3001].project_id.as_deref(),
            Some("proj-certificreate")
        );
        assert!(
            by_port[&8080].project_id.is_none(),
            "no cwd stays ungrouped"
        );
        assert!(
            by_port[&9000].project_id.is_none(),
            "no marker stays ungrouped"
        );
    }

    #[test]
    fn group_service_ids_match_member_services_in_port_order() {
        let (services, projects) = grouped(
            vec![
                sock(
                    "127.0.0.1",
                    3002,
                    Some(30),
                    Some(proc_at(30, "node", "/home/brad/Code/app/web")),
                ),
                sock(
                    "127.0.0.1",
                    3000,
                    Some(10),
                    Some(proc_at(10, "node", "/home/brad/Code/app")),
                ),
            ],
            &[("/home/brad/Code/app", Marker::Repo)],
        );
        let expected: Vec<String> = services.iter().map(|s| s.id.clone()).collect();
        assert_eq!(projects[0].service_ids, expected);
        assert_eq!(services[0].port, 3000, "services stay port-sorted");
    }

    #[test]
    fn colliding_basenames_get_parent_qualified_ids() {
        let (_, projects) = grouped(
            vec![
                sock(
                    "127.0.0.1",
                    3000,
                    Some(10),
                    Some(proc_at(10, "node", "/home/brad/Code/app")),
                ),
                sock(
                    "127.0.0.1",
                    4000,
                    Some(20),
                    Some(proc_at(20, "node", "/home/brad/Work/app")),
                ),
            ],
            &[
                ("/home/brad/Code/app", Marker::Repo),
                ("/home/brad/Work/app", Marker::Repo),
            ],
        );
        let ids: Vec<&str> = projects.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["proj-code-app", "proj-work-app"]);
    }

    #[test]
    fn roots_outside_home_display_absolute() {
        let (_, projects) = grouped(
            vec![sock(
                "127.0.0.1",
                3000,
                Some(10),
                Some(proc_at(10, "node", "/srv/tool/sub")),
            )],
            &[("/srv/tool", Marker::Package)],
        );
        assert_eq!(projects[0].root, "/srv/tool");
        assert_eq!(projects[0].id, "proj-tool");
    }

    #[test]
    fn framework_flows_from_the_command_line() {
        let vite = ProcessInfo {
            command: Some("node /home/brad/Code/app/node_modules/.bin/vite".into()),
            ..proc_info(10, "node")
        };
        let services = services_from(vec![sock("127.0.0.1", 5173, Some(10), Some(vite))]);
        assert_eq!(services[0].framework.as_deref(), Some("Vite"));
        assert_eq!(services[0].process_name.as_deref(), Some("node"));
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
    fn non_http_services_get_no_url_but_still_classify() {
        let services = services_from(vec![
            sock("0.0.0.0", 22, None, None),
            sock(
                "127.0.0.1",
                54329,
                Some(42),
                Some(proc_info(42, "postgres")),
            ),
            sock("127.0.0.1", 3000, Some(10), Some(proc_info(10, "node"))),
        ]);
        let by_port: HashMap<u16, &Service> = services.iter().map(|s| (s.port, s)).collect();

        assert!(by_port[&22].url.is_none(), "ssh gets no link");
        assert!(matches!(by_port[&22].exposure, Exposure::Lan));
        assert!(
            by_port[&54329].url.is_none(),
            "postgres framework kills the url on an odd port"
        );
        assert_eq!(by_port[&54329].framework.as_deref(), Some("Postgres"));
        assert_eq!(
            by_port[&3000].url.as_deref(),
            Some("http://localhost:3000"),
            "dev servers keep their urls"
        );
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
    fn stale_accuses_only_old_dev_servers() {
        const DAY: u64 = 86_400;
        let stale = stale_hint(Some("Astro"), Some(3 * DAY)).expect("3d astro is stale");
        assert_eq!(stale.reason, "Astro dev server running for 3d");
        assert!(
            stale_hint(Some("Vite"), Some(2 * DAY)).is_none(),
            "under threshold"
        );
        assert!(
            stale_hint(Some("Postgres"), Some(8 * DAY)).is_none(),
            "databases run long legitimately"
        );
        assert!(
            stale_hint(None, Some(30 * DAY)).is_none(),
            "unlabeled processes are never accused"
        );
        assert!(
            stale_hint(Some("Next.js"), None).is_none(),
            "unknown age is not stale"
        );
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
