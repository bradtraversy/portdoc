//! Mocked `DevSnapshot` used by `/api/snapshot` and `--json` until real
//! probing lands (features 4-6). Mirrors the prototype dummy data so the mock
//! dashboard (feature 3) can render straight from this payload.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::snapshot::{
    Conflict, DevSnapshot, DockerHint, Exposure, ProjectGroup, Service, StaleHint,
};

const SVC_NEXT_SERVER: &str = "svc-3000-next-server";
const SVC_NEXT_DEV: &str = "svc-3001-next-dev";
const SVC_PORTDOC: &str = "svc-7788-portdoc";
const SVC_VITE: &str = "svc-5174-vite";
const SVC_VITE_HOST: &str = "svc-5173-vite-host";
const SVC_POSTGRES: &str = "svc-5432-postgres";
const SVC_REDIS: &str = "svc-6379-redis";
const SVC_UNKNOWN: &str = "svc-8080-unknown";
const PROJ_STARTDEV: &str = "proj-startdev";
const PROJ_PORTDOC: &str = "proj-portdoc";
const PROJ_REACT_CRASH: &str = "proj-react-crash";

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
            id: SVC_NEXT_SERVER.into(),
            port: 3000,
            pid: Some(12904),
            process_name: Some("next-server".into()),
            command: Some("next-server (v15.3.2)".into()),
            cwd: Some("~/Code/startdev".into()),
            user: Some("brad".into()),
            project_id: Some(PROJ_STARTDEV.into()),
            framework: Some("Next.js".into()),
            exposure: Exposure::Local,
            url: Some("http://localhost:3000".into()),
            started_age: Some("6d".into()),
            stale: Some(StaleHint {
                reason: "started 6 days ago, no requests observed".into(),
            }),
        },
        Service {
            id: SVC_NEXT_DEV.into(),
            port: 3001,
            pid: Some(84117),
            process_name: Some("node".into()),
            command: Some("node ~/Code/startdev/node_modules/.bin/next dev".into()),
            cwd: Some("~/Code/startdev".into()),
            user: Some("brad".into()),
            project_id: Some(PROJ_STARTDEV.into()),
            framework: Some("Next.js".into()),
            exposure: Exposure::Local,
            url: Some("http://localhost:3001".into()),
            started_age: Some("2h".into()),
            stale: None,
        },
        Service {
            id: SVC_PORTDOC.into(),
            port: 7788,
            pid: Some(90312),
            process_name: Some("portdoc".into()),
            command: Some("target/debug/portdoc --port 7788".into()),
            cwd: Some("~/Code/portdoc".into()),
            user: Some("brad".into()),
            project_id: Some(PROJ_PORTDOC.into()),
            framework: Some("Rust".into()),
            exposure: Exposure::Local,
            url: Some("http://127.0.0.1:7788".into()),
            started_age: Some("4m".into()),
            stale: None,
        },
        Service {
            id: SVC_VITE.into(),
            port: 5174,
            pid: Some(90355),
            process_name: Some("node".into()),
            command: Some("node ~/Code/portdoc/web/node_modules/.bin/vite".into()),
            cwd: Some("~/Code/portdoc/web".into()),
            user: Some("brad".into()),
            project_id: Some(PROJ_PORTDOC.into()),
            framework: Some("Vite".into()),
            exposure: Exposure::Local,
            url: Some("http://localhost:5174".into()),
            started_age: Some("4m".into()),
            stale: None,
        },
        Service {
            id: SVC_VITE_HOST.into(),
            port: 5173,
            pid: Some(88231),
            process_name: Some("node".into()),
            command: Some("node ~/Code/react-crash-2026/node_modules/.bin/vite --host".into()),
            cwd: Some("~/Code/react-crash-2026".into()),
            user: Some("brad".into()),
            project_id: Some(PROJ_REACT_CRASH.into()),
            framework: Some("Vite".into()),
            exposure: Exposure::Lan,
            url: Some("http://192.168.1.44:5173".into()),
            started_age: Some("3d".into()),
            stale: Some(StaleHint {
                reason: "running 3 days in a project not touched since".into(),
            }),
        },
        Service {
            id: SVC_POSTGRES.into(),
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
            id: SVC_REDIS.into(),
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
            id: SVC_UNKNOWN.into(),
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
            id: PROJ_STARTDEV.into(),
            name: "startdev".into(),
            root: "~/Code/startdev".into(),
            package_manager: Some("npm".into()),
            git_branch: Some("feature/course-player".into()),
            service_ids: vec![SVC_NEXT_SERVER.into(), SVC_NEXT_DEV.into()],
        },
        ProjectGroup {
            id: PROJ_PORTDOC.into(),
            name: "portdoc".into(),
            root: "~/Code/portdoc".into(),
            package_manager: Some("cargo".into()),
            git_branch: Some("main".into()),
            service_ids: vec![SVC_PORTDOC.into(), SVC_VITE.into()],
        },
        ProjectGroup {
            id: PROJ_REACT_CRASH.into(),
            name: "react-crash-2026".into(),
            root: "~/Code/react-crash-2026".into(),
            package_manager: Some("npm".into()),
            git_branch: Some("main".into()),
            service_ids: vec![SVC_VITE_HOST.into()],
        },
    ]
}

fn conflicts() -> Vec<Conflict> {
    vec![
        Conflict {
            port: 3000,
            service_ids: vec![SVC_NEXT_SERVER.into(), SVC_NEXT_DEV.into()],
            hint: "next dev wanted :3000 but a stale next-server holds it; stop the stale \
                   holder to reclaim the port"
                .into(),
        },
        Conflict {
            port: 5173,
            service_ids: vec![SVC_VITE_HOST.into(), SVC_VITE.into()],
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
            service_id: Some(SVC_POSTGRES.into()),
            image: Some("postgres:16".into()),
            compose_project: Some("startdev".into()),
        },
        DockerHint {
            port: 6379,
            container: "redis-cache".into(),
            service_id: Some(SVC_REDIS.into()),
            image: Some("redis:7".into()),
            compose_project: None,
        },
    ]
}
