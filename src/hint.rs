//! Per-snapshot shared-port detection: distinct listeners on one port
//! (different-owner v4/v6 splits, SO_REUSEPORT pools). A neutral fact, not
//! an alarm; bind-time EADDRINUSE fights are unobservable from a snapshot.
//! Pure: services in, conflicts out.

use crate::snapshot::{Conflict, Service};

pub fn detect_conflicts(services: &[Service]) -> Vec<Conflict> {
    shared_port_conflicts(services)
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

fn display_name(service: &Service) -> &str {
    service
        .framework
        .as_deref()
        .or(service.process_name.as_deref())
        .unwrap_or("unknown")
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
        let conflicts = detect_conflicts(&services);
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
        assert!(detect_conflicts(&services).is_empty());
    }

    #[test]
    fn near_default_port_is_not_inferred_as_a_bump() {
        // Regression for the dropped bump inference: a framework service one
        // above its default while another service holds the default is not a
        // conflict - causation is unobservable from a snapshot.
        let services = [
            svc(
                "holder",
                3000,
                Some(1),
                Some("next-server"),
                Some("Next.js"),
                None,
            ),
            svc(
                "near",
                5174,
                Some(2),
                Some("node"),
                Some("Vite"),
                Some("node .bin/vite"),
            ),
            svc(
                "vite-default",
                5173,
                Some(3),
                Some("node"),
                Some("Vite"),
                None,
            ),
        ];
        assert!(detect_conflicts(&services).is_empty());
    }
}
