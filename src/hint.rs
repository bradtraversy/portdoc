//! Per-snapshot conflict detection (feature 10): same-port multi-owner
//! listeners and framework default-port bump inference. Pure: services and
//! projects in, conflicts out.

use std::collections::HashMap;

use crate::snapshot::{Conflict, ProjectGroup, Service};

/// Framework -> the port it starts on by default.
const DEFAULT_PORTS: [(&str, u16); 6] = [
    ("Next.js", 3000),
    ("Remix", 3000),
    ("Nuxt", 3000),
    ("React scripts", 3000),
    ("Vite", 5173),
    ("Astro", 4321),
];

/// How far above its default a bumped dev server is still recognized.
const BUMP_RANGE: u16 = 10;

pub fn detect_conflicts(services: &[Service], projects: &[ProjectGroup]) -> Vec<Conflict> {
    let mut conflicts = shared_port_conflicts(services);
    conflicts.extend(bumped_port_conflicts(services, projects));
    conflicts.sort_by_key(|c| c.port);
    conflicts
}

/// Two or more merged services on one port (SO_REUSEPORT, v4/v6 split
/// ownership). Relies on the adapter's port-sorted output.
fn shared_port_conflicts(services: &[Service]) -> Vec<Conflict> {
    let mut conflicts = Vec::new();
    let mut i = 0;
    while i < services.len() {
        let run = services[i..]
            .iter()
            .take_while(|s| s.port == services[i].port)
            .count();
        if run > 1 {
            let group = &services[i..i + run];
            let names: Vec<&str> = group.iter().map(display_name).collect();
            conflicts.push(Conflict {
                port: services[i].port,
                service_ids: group.iter().map(|s| s.id.clone()).collect(),
                hint: format!(
                    "{run} processes are listening on :{} ({}).",
                    services[i].port,
                    names.join(", ")
                ),
            });
        }
        i += run;
    }
    conflicts
}

/// A framework service running just above its default port while another
/// service holds the default. Conservative: an explicit port in the
/// command means intentional, and a same-pid holder is one app with two
/// ports, not a conflict.
fn bumped_port_conflicts(services: &[Service], projects: &[ProjectGroup]) -> Vec<Conflict> {
    let holder_for_port: HashMap<u16, &Service> =
        services.iter().rev().map(|s| (s.port, s)).collect();

    // default port -> (holder, bumped instances), in service order
    let mut by_default: Vec<(u16, &Service, Vec<&Service>)> = Vec::new();

    for service in services {
        let Some(default) = default_port(service) else {
            continue;
        };
        if service.port <= default || service.port > default + BUMP_RANGE {
            continue;
        }
        if command_mentions_own_port(service) {
            continue;
        }
        let Some(holder) = holder_for_port.get(&default) else {
            continue;
        };
        if holder.pid.is_some() && holder.pid == service.pid {
            continue;
        }
        match by_default.iter_mut().find(|(d, _, _)| *d == default) {
            Some((_, _, bumped)) => bumped.push(service),
            None => by_default.push((default, holder, vec![service])),
        }
    }

    by_default
        .into_iter()
        .map(|(default, holder, bumped)| {
            let framework = bumped[0].framework.as_deref().unwrap_or("this dev server");
            let moved = bumped
                .iter()
                .map(|s| format!(":{}", s.port))
                .collect::<Vec<_>>()
                .join(", ");
            let hint = if bumped.len() == 1 {
                format!(
                    "{framework} defaults to :{default}, held by {}; this instance moved to {moved}.",
                    holder_label(holder, projects)
                )
            } else {
                format!(
                    "{framework} defaults to :{default}, held by {}; {} instances moved to {moved}.",
                    holder_label(holder, projects),
                    bumped.len()
                )
            };
            let mut service_ids = vec![holder.id.clone()];
            service_ids.extend(bumped.iter().map(|s| s.id.clone()));
            Conflict {
                port: default,
                service_ids,
                hint,
            }
        })
        .collect()
}

fn default_port(service: &Service) -> Option<u16> {
    let framework = service.framework.as_deref()?;
    DEFAULT_PORTS
        .iter()
        .find(|(f, _)| *f == framework)
        .map(|(_, port)| *port)
}

/// "--port 4322" (or any exact numeric token equal to the port) in the
/// command means the port was chosen on purpose.
fn command_mentions_own_port(service: &Service) -> bool {
    let port = service.port.to_string();
    service
        .command
        .as_deref()
        .is_some_and(|c| c.split(|ch: char| !ch.is_ascii_digit()).any(|t| t == port))
}

fn display_name(service: &Service) -> &str {
    service
        .framework
        .as_deref()
        .or(service.process_name.as_deref())
        .unwrap_or("unknown")
}

fn holder_label(holder: &Service, projects: &[ProjectGroup]) -> String {
    let name = display_name(holder);
    let project = holder
        .project_id
        .as_deref()
        .and_then(|id| projects.iter().find(|p| p.id == id));
    match project {
        Some(p) => format!("{name} ({})", p.name),
        None => name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::Exposure;

    fn svc(
        id: &str,
        port: u16,
        pid: Option<u32>,
        name: Option<&str>,
        framework: Option<&str>,
        command: Option<&str>,
    ) -> Service {
        Service {
            id: id.into(),
            port,
            pid,
            process_name: name.map(Into::into),
            command: command.map(Into::into),
            cwd: None,
            user: None,
            project_id: None,
            framework: framework.map(Into::into),
            exposure: Exposure::Local,
            url: None,
            started_age: None,
            stale: None,
        }
    }

    #[test]
    fn two_owners_on_one_port_conflict() {
        let services = [
            svc(
                "a",
                3000,
                Some(1),
                Some("next-server"),
                Some("Next.js"),
                None,
            ),
            svc("b", 3000, Some(2), Some("node"), None, None),
            svc("c", 4000, Some(3), Some("node"), None, None),
        ];
        let conflicts = detect_conflicts(&services, &[]);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].port, 3000);
        assert_eq!(conflicts[0].service_ids, vec!["a", "b"]);
        assert!(
            conflicts[0].hint.contains("2 processes"),
            "{}",
            conflicts[0].hint
        );
        assert!(
            conflicts[0].hint.contains("Next.js, node"),
            "{}",
            conflicts[0].hint
        );
    }

    #[test]
    fn distinct_ports_do_not_conflict() {
        let services = [
            svc("a", 3000, Some(1), Some("node"), None, None),
            svc("b", 3001, Some(2), Some("node"), None, None),
        ];
        assert!(detect_conflicts(&services, &[]).is_empty());
    }

    #[test]
    fn bumped_vite_points_at_the_holder() {
        let services = [
            svc("holder", 5173, Some(1), Some("node"), Some("Vite"), None),
            svc(
                "bumped",
                5174,
                Some(2),
                Some("node"),
                Some("Vite"),
                Some("node .bin/vite"),
            ),
        ];
        let conflicts = detect_conflicts(&services, &[]);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].port, 5173);
        assert_eq!(conflicts[0].service_ids, vec!["holder", "bumped"]);
        assert!(
            conflicts[0].hint.contains("Vite defaults to :5173"),
            "{}",
            conflicts[0].hint
        );
    }

    #[test]
    fn explicit_port_flag_is_not_a_bump() {
        let services = [
            svc("holder", 4321, Some(1), Some("node"), Some("Astro"), None),
            svc(
                "explicit",
                4322,
                Some(2),
                Some("node"),
                Some("Astro"),
                Some("node astro.mjs dev --port 4322"),
            ),
        ];
        assert!(detect_conflicts(&services, &[]).is_empty());
    }

    #[test]
    fn same_pid_on_two_ports_is_one_app_not_a_conflict() {
        let services = [
            svc("a", 3000, Some(9), Some("node"), Some("Next.js"), None),
            svc(
                "b",
                3001,
                Some(9),
                Some("node"),
                Some("Next.js"),
                Some("node .bin/next dev"),
            ),
        ];
        assert!(detect_conflicts(&services, &[]).is_empty());
    }

    #[test]
    fn multiple_bumps_fold_into_one_conflict() {
        let services = [
            svc("holder", 5173, Some(1), Some("node"), Some("Vite"), None),
            svc(
                "b1",
                5174,
                Some(2),
                Some("node"),
                Some("Vite"),
                Some("node .bin/vite"),
            ),
            svc(
                "b2",
                5175,
                Some(3),
                Some("node"),
                Some("Vite"),
                Some("node .bin/vite"),
            ),
        ];
        let conflicts = detect_conflicts(&services, &[]);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].service_ids, vec!["holder", "b1", "b2"]);
        assert!(
            conflicts[0].hint.contains("2 instances"),
            "{}",
            conflicts[0].hint
        );
    }

    #[test]
    fn far_from_default_is_not_a_bump() {
        let services = [
            svc("holder", 5173, Some(1), Some("node"), Some("Vite"), None),
            svc(
                "far",
                5190,
                Some(2),
                Some("node"),
                Some("Vite"),
                Some("node .bin/vite"),
            ),
        ];
        assert!(detect_conflicts(&services, &[]).is_empty());
    }

    #[test]
    fn holder_label_includes_the_project_name() {
        let mut holder = svc("holder", 5173, Some(1), Some("node"), Some("Vite"), None);
        holder.project_id = Some("proj-startdev".into());
        let services = [
            holder,
            svc(
                "bumped",
                5174,
                Some(2),
                Some("node"),
                Some("Vite"),
                Some("node .bin/vite"),
            ),
        ];
        let projects = [ProjectGroup {
            id: "proj-startdev".into(),
            name: "startdev".into(),
            root: "~/Code/startdev".into(),
            package_manager: None,
            git_branch: None,
            service_ids: vec!["holder".into()],
        }];
        let conflicts = detect_conflicts(&services, &projects);
        assert!(
            conflicts[0].hint.contains("Vite (startdev)"),
            "{}",
            conflicts[0].hint
        );
    }
}
