//! Docker container hints from the docker CLI (decided 2026-07-08 over a
//! socket client). No docker binary, a stopped daemon, or garbage output all
//! degrade to no hints - the snapshot must stay healthy without Docker.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::snapshot::{DockerHint, Service};

pub struct Container {
    pub name: String,
    pub image: Option<String>,
    pub compose_project: Option<String>,
    pub ports: Vec<u16>,
}

#[derive(Deserialize)]
struct PsLine {
    #[serde(rename = "Names", default)]
    names: String,
    #[serde(rename = "Image", default)]
    image: String,
    #[serde(rename = "Labels", default)]
    labels: String,
    #[serde(rename = "Ports", default)]
    ports: String,
}

pub fn running_containers() -> Vec<Container> {
    match ps_output(Duration::from_secs(2)) {
        Some(stdout) => parse_containers(&stdout),
        None => Vec::new(),
    }
}

/// Run `docker ps` with a hard deadline so a hung daemon can never stall
/// the snapshot. Stdout is drained on a thread because a killed child only
/// unblocks the reader once its pipe closes.
fn ps_output(timeout: Duration) -> Option<String> {
    let mut child = Command::new("docker")
        .args(["ps", "--format", "{{json .}}"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = stdout.read_to_string(&mut buf);
        buf
    });

    let deadline = Instant::now() + timeout;
    let success = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.success(),
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                break false;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(25)),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                break false;
            }
        }
    };

    let output = reader.join().ok()?;
    success.then_some(output)
}

/// One hint per published container port, joined to the listener that holds
/// it - the `docker-proxy` owner when several share the port, since that is
/// the provable Docker-side socket.
pub fn hints(containers: Vec<Container>, services: &[Service]) -> Vec<DockerHint> {
    let mut hints: Vec<DockerHint> = containers
        .into_iter()
        .flat_map(|container| {
            container
                .ports
                .iter()
                .map(|&port| DockerHint {
                    port,
                    container: container.name.clone(),
                    service_id: owner_id(port, services),
                    image: container.image.clone(),
                    compose_project: container.compose_project.clone(),
                })
                .collect::<Vec<_>>()
        })
        .collect();
    hints.sort_by(|a, b| (a.port, &a.container).cmp(&(b.port, &b.container)));
    hints
}

fn owner_id(port: u16, services: &[Service]) -> Option<String> {
    let on_port = || services.iter().filter(|s| s.port == port);
    on_port()
        .find(|s| s.process_name.as_deref() == Some("docker-proxy"))
        .or_else(|| on_port().next())
        .map(|s| s.id.clone())
}

/// Parse `docker ps --format {{json .}}` output (one JSON object per line).
/// Lines that fail to parse are skipped, never fatal.
pub fn parse_containers(stdout: &str) -> Vec<Container> {
    stdout.lines().filter_map(container_from_line).collect()
}

fn container_from_line(line: &str) -> Option<Container> {
    let ps: PsLine = serde_json::from_str(line.trim()).ok()?;
    let name = ps.names.split(',').next().unwrap_or("").trim().to_string();
    if name.is_empty() {
        return None;
    }
    Some(Container {
        name,
        image: Some(ps.image).filter(|i| !i.is_empty()),
        compose_project: compose_project(&ps.labels),
        ports: published_ports(&ps.ports),
    })
}

fn compose_project(labels: &str) -> Option<String> {
    labels
        .split(',')
        .find_map(|label| label.trim().strip_prefix("com.docker.compose.project="))
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

/// Published host TCP ports from a `Ports` string like
/// `0.0.0.0:5432->5432/tcp, [::]:5432->5432/tcp` (-> [5432], deduped).
/// Unpublished entries (`6379/tcp`), udp, and unparseable forms such as
/// compressed port ranges are skipped.
fn published_ports(ports: &str) -> Vec<u16> {
    let mut out: Vec<u16> = Vec::new();
    for entry in ports.split(',') {
        let Some((mapping, proto)) = entry.trim().rsplit_once('/') else {
            continue;
        };
        if proto != "tcp" {
            continue;
        }
        let Some((host, _)) = mapping.split_once("->") else {
            continue;
        };
        let Some((_, port)) = host.rsplit_once(':') else {
            continue;
        };
        let Ok(port) = port.parse::<u16>() else {
            continue;
        };
        if !out.contains(&port) {
            out.push(port);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::Exposure;

    fn svc(id: &str, port: u16, name: Option<&str>) -> Service {
        Service {
            id: id.into(),
            port,
            pid: Some(1),
            process_name: name.map(Into::into),
            command: None,
            cwd: None,
            user: None,
            project_id: None,
            framework: None,
            exposure: Exposure::Local,
            url: None,
            started_age: None,
            stale: None,
        }
    }

    fn container(name: &str, compose: Option<&str>, ports: &[u16]) -> Container {
        Container {
            name: name.into(),
            image: Some(format!("{name}:latest")),
            compose_project: compose.map(Into::into),
            ports: ports.to_vec(),
        }
    }

    #[test]
    fn one_hint_per_published_port_sorted_by_port() {
        let hints = hints(
            vec![
                container("proxy", None, &[8443, 8080]),
                container("db", Some("shop"), &[5432]),
            ],
            &[],
        );
        let ports: Vec<u16> = hints.iter().map(|h| h.port).collect();
        assert_eq!(ports, vec![5432, 8080, 8443]);
        assert_eq!(hints[0].container, "db");
        assert_eq!(hints[0].image.as_deref(), Some("db:latest"));
        assert_eq!(hints[0].compose_project.as_deref(), Some("shop"));
        assert!(hints[0].service_id.is_none(), "no listener, no join");
        assert!(hints[1].compose_project.is_none());
    }

    #[test]
    fn join_prefers_the_docker_proxy_listener() {
        let services = [
            svc("svc-5432-postgres", 5432, Some("postgres")),
            svc("svc-5432-docker-proxy", 5432, Some("docker-proxy")),
            svc("svc-8025-node", 8025, Some("node")),
        ];
        let hints = hints(
            vec![
                container("pg", None, &[5432]),
                container("mail", None, &[8025]),
                container("ghost", None, &[9999]),
            ],
            &services,
        );
        assert_eq!(
            hints[0].service_id.as_deref(),
            Some("svc-5432-docker-proxy")
        );
        assert_eq!(
            hints[1].service_id.as_deref(),
            Some("svc-8025-node"),
            "falls back to whoever holds the port"
        );
        assert!(hints[2].service_id.is_none());
    }

    fn ps_line(names: &str, image: &str, labels: &str, ports: &str) -> String {
        serde_json::json!({
            "Names": names, "Image": image, "Labels": labels, "Ports": ports,
            "ID": "abc123", "State": "running",
        })
        .to_string()
    }

    #[test]
    fn published_ports_dedupe_dual_stack_and_skip_unpublished() {
        assert_eq!(
            published_ports("0.0.0.0:5432->5432/tcp, [::]:5432->5432/tcp"),
            vec![5432]
        );
        assert_eq!(
            published_ports("127.0.0.1:8025->8025/tcp, 1025/tcp"),
            vec![8025]
        );
        assert!(published_ports("6379/tcp").is_empty());
        assert!(published_ports("").is_empty());
    }

    #[test]
    fn published_ports_skip_udp_and_tolerate_garbage() {
        assert!(published_ports("0.0.0.0:53->53/udp").is_empty());
        assert!(published_ports("0.0.0.0:8080-8090->8080-8090/tcp").is_empty());
        assert!(published_ports("not ports at all").is_empty());
        assert_eq!(published_ports("junk, 0.0.0.0:3000->3000/tcp"), vec![3000]);
    }

    #[test]
    fn published_ports_keep_distinct_ports_in_order() {
        assert_eq!(
            published_ports("0.0.0.0:8080->80/tcp, 0.0.0.0:8443->443/tcp"),
            vec![8080, 8443]
        );
    }

    #[test]
    fn compose_project_comes_from_the_label_list() {
        let labels = "maintainer=x,com.docker.compose.project=myapp,com.docker.compose.service=db";
        assert_eq!(compose_project(labels).as_deref(), Some("myapp"));
        assert!(compose_project("maintainer=x").is_none());
        assert!(compose_project("com.docker.compose.project=").is_none());
        assert!(compose_project("").is_none());
    }

    #[test]
    fn containers_parse_per_line_and_skip_malformed() {
        let out = format!(
            "{}\nnot json\n{}\n",
            ps_line(
                "pg-main",
                "postgres:16",
                "com.docker.compose.project=shop",
                "0.0.0.0:5432->5432/tcp"
            ),
            ps_line(
                "mailpit",
                "axllent/mailpit",
                "",
                "127.0.0.1:8025->8025/tcp, 1025/tcp"
            ),
        );
        let containers = parse_containers(&out);
        assert_eq!(containers.len(), 2);
        assert_eq!(containers[0].name, "pg-main");
        assert_eq!(containers[0].image.as_deref(), Some("postgres:16"));
        assert_eq!(containers[0].compose_project.as_deref(), Some("shop"));
        assert_eq!(containers[0].ports, vec![5432]);
        assert_eq!(containers[1].name, "mailpit");
        assert!(containers[1].compose_project.is_none());
        assert_eq!(containers[1].ports, vec![8025]);
    }

    #[test]
    fn first_name_wins_and_nameless_lines_are_skipped() {
        let multi = ps_line("web,web-alias", "nginx", "", "");
        let containers = parse_containers(&multi);
        assert_eq!(containers[0].name, "web");
        assert!(containers[0].image.as_deref() == Some("nginx"));
        assert!(containers[0].ports.is_empty());

        let nameless = ps_line("", "nginx", "", "0.0.0.0:80->80/tcp");
        assert!(parse_containers(&nameless).is_empty());
        assert!(parse_containers("").is_empty());
    }
}
