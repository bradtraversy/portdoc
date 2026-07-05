//! The locked `DevSnapshot` JSON contract (build-plan feature 1). Every
//! consumer (UI, `--json`, probe adapters) depends on this shape; change it
//! here or not at all. Optional fields are omitted when absent, never null.

use serde::Serialize;

#[derive(Serialize)]
pub struct DevSnapshot {
    /// Unix epoch milliseconds when the snapshot was built.
    pub generated_at: u64,
    pub services: Vec<Service>,
    pub projects: Vec<ProjectGroup>,
    pub conflicts: Vec<Conflict>,
    pub docker_hints: Vec<DockerHint>,
}

#[derive(Serialize)]
pub struct Service {
    pub id: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    pub exposure: Exposure,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_age: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale: Option<StaleHint>,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Exposure {
    Local,
    Lan,
    Docker,
    Unknown,
}

/// Present on a service means stale; absent means not stale.
#[derive(Serialize)]
pub struct StaleHint {
    pub reason: String,
}

#[derive(Serialize)]
pub struct ProjectGroup {
    pub id: String,
    pub name: String,
    pub root: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_manager: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    pub service_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct Conflict {
    pub port: u16,
    pub service_ids: Vec<String>,
    pub hint: String,
}

#[derive(Serialize)]
pub struct DockerHint {
    pub port: u16,
    pub container: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compose_project: Option<String>,
}
