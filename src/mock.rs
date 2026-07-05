//! Mocked `DevSnapshot` used by `/api/snapshot` and `--json` until real
//! probing lands (features 4-6). Mirrors the prototype dummy data so the mock
//! dashboard (feature 3) can render straight from this payload.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::snapshot::{
    Conflict, DevSnapshot, DockerHint, Exposure, ProjectGroup, Service, StaleHint,
};

pub fn snapshot() -> DevSnapshot {
    DevSnapshot {
        generated_at: now_ms(),
        services: services(),
        projects: projects(),
        conflicts: conflicts(),
        docker_hints: docker_hints(),
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn services() -> Vec<Service> {
    vec![
        Service {
            id: "svc-3000-next-server".into(),
            port: 3000,
            pid: Some(12904),
            process_name: Some("next-server".into()),
            command: Some("next-server (v15.3.2)".into()),
            cwd: Some("~/Code/startdev".into()),
            user: Some("brad".into()),
            project_id: Some("proj-startdev".into()),
            framework: Some("Next.js".into()),
            exposure: Exposure::Local,
            url: Some("http://localhost:3000".into()),
            started_age: Some("6d".into()),
            stale: Some(StaleHint {
                reason: "started 6 days ago, no requests observed".into(),
            }),
        },
        Service {
            id: "svc-3001-next-dev".into(),
            port: 3001,
            pid: Some(84117),
            process_name: Some("node".into()),
            command: Some("node ~/Code/startdev/node_modules/.bin/next dev".into()),
            cwd: Some("~/Code/startdev".into()),
            user: Some("brad".into()),
            project_id: Some("proj-startdev".into()),
            framework: Some("Next.js".into()),
            exposure: Exposure::Local,
            url: Some("http://localhost:3001".into()),
            started_age: Some("2h".into()),
            stale: None,
        },
        Service {
            id: "svc-7788-portdoc".into(),
            port: 7788,
            pid: Some(90312),
            process_name: Some("portdoc".into()),
            command: Some("target/debug/portdoc --port 7788".into()),
            cwd: Some("~/Code/portdoc".into()),
            user: Some("brad".into()),
            project_id: Some("proj-portdoc".into()),
            framework: Some("Rust".into()),
            exposure: Exposure::Local,
            url: Some("http://127.0.0.1:7788".into()),
            started_age: Some("4m".into()),
            stale: None,
        },
        Service {
            id: "svc-5174-vite".into(),
            port: 5174,
            pid: Some(90355),
            process_name: Some("node".into()),
            command: Some("node ~/Code/portdoc/web/node_modules/.bin/vite".into()),
            cwd: Some("~/Code/portdoc/web".into()),
            user: Some("brad".into()),
            project_id: Some("proj-portdoc".into()),
            framework: Some("Vite".into()),
            exposure: Exposure::Local,
            url: Some("http://localhost:5174".into()),
            started_age: Some("4m".into()),
            stale: None,
        },
        Service {
            id: "svc-5173-vite-host".into(),
            port: 5173,
            pid: Some(88231),
            process_name: Some("node".into()),
            command: Some("node ~/Code/react-crash-2026/node_modules/.bin/vite --host".into()),
            cwd: Some("~/Code/react-crash-2026".into()),
            user: Some("brad".into()),
            project_id: Some("proj-react-crash".into()),
            framework: Some("Vite".into()),
            exposure: Exposure::Lan,
            url: Some("http://192.168.1.44:5173".into()),
            started_age: Some("3d".into()),
            stale: Some(StaleHint {
                reason: "running 3 days in a project not touched since".into(),
            }),
        },
        Service {
            id: "svc-5432-postgres".into(),
            port: 5432,
            pid: None,
            process_name: None,
            command: None,
            cwd: None,
            user: None,
            project_id: None,
            framework: Some("Postgres".into()),
            exposure: Exposure::Docker,
            url: None,
            started_age: Some("2d".into()),
            stale: None,
        },
        Service {
            id: "svc-6379-redis".into(),
            port: 6379,
            pid: None,
            process_name: None,
            command: None,
            cwd: None,
            user: None,
            project_id: None,
            framework: Some("Redis".into()),
            exposure: Exposure::Docker,
            url: None,
            started_age: Some("2d".into()),
            stale: None,
        },
        Service {
            id: "svc-8080-unknown".into(),
            port: 8080,
            pid: None,
            process_name: None,
            command: None,
            cwd: None,
            user: None,
            project_id: None,
            framework: None,
            exposure: Exposure::Unknown,
            url: None,
            started_age: None,
            stale: None,
        },
    ]
}

fn projects() -> Vec<ProjectGroup> {
    vec![
        ProjectGroup {
            id: "proj-startdev".into(),
            name: "startdev".into(),
            root: "~/Code/startdev".into(),
            package_manager: Some("npm".into()),
            git_branch: Some("feature/course-player".into()),
            service_ids: vec!["svc-3000-next-server".into(), "svc-3001-next-dev".into()],
        },
        ProjectGroup {
            id: "proj-portdoc".into(),
            name: "portdoc".into(),
            root: "~/Code/portdoc".into(),
            package_manager: Some("cargo".into()),
            git_branch: Some("main".into()),
            service_ids: vec!["svc-7788-portdoc".into(), "svc-5174-vite".into()],
        },
        ProjectGroup {
            id: "proj-react-crash".into(),
            name: "react-crash-2026".into(),
            root: "~/Code/react-crash-2026".into(),
            package_manager: Some("npm".into()),
            git_branch: Some("main".into()),
            service_ids: vec!["svc-5173-vite-host".into()],
        },
    ]
}

fn conflicts() -> Vec<Conflict> {
    vec![
        Conflict {
            port: 3000,
            service_ids: vec!["svc-3000-next-server".into(), "svc-3001-next-dev".into()],
            hint: "next dev wanted :3000 but a stale next-server holds it; stop the stale \
                   holder to reclaim the port"
                .into(),
        },
        Conflict {
            port: 5173,
            service_ids: vec!["svc-5173-vite-host".into(), "svc-5174-vite".into()],
            hint: "vite wanted :5173 but another project's vite --host holds it; it \
                   auto-bumped to :5174"
                .into(),
        },
    ]
}

fn docker_hints() -> Vec<DockerHint> {
    vec![
        DockerHint {
            port: 5432,
            container: "pg-startdev".into(),
            service_id: Some("svc-5432-postgres".into()),
            image: Some("postgres:16".into()),
            compose_project: Some("startdev".into()),
        },
        DockerHint {
            port: 6379,
            container: "redis-cache".into(),
            service_id: Some("svc-6379-redis".into()),
            image: Some("redis:7".into()),
            compose_project: None,
        },
    ]
}
