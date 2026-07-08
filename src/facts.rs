//! Per-project facts (feature 16b): what a project is, its stack, and its
//! repo state - cheap single-file reads at the root plus two deadline-guarded
//! git commands. Anything unreadable leaves its field absent.

use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::snapshot::{KeyDep, Script};

#[derive(Default)]
pub struct ProjectFacts {
    pub description: Option<String>,
    pub scripts: Vec<Script>,
    pub key_deps: Vec<KeyDep>,
    pub workspaces: Vec<String>,
    pub node_version: Option<String>,
    pub last_commit_secs_ago: Option<u64>,
    pub dirty: Option<bool>,
}

pub fn project_facts(root: &Path) -> ProjectFacts {
    let read = |file: &str| std::fs::read_to_string(root.join(file)).ok();
    let package = read("package.json")
        .map(|s| package_facts(&s))
        .unwrap_or_default();
    let cargo = read("Cargo.toml")
        .map(|s| cargo_facts(&s))
        .unwrap_or_default();
    let (last_commit_secs_ago, dirty) = git_facts(root);

    let mut key_deps = package.key_deps;
    key_deps.extend(cargo.key_deps);
    let mut workspaces = package.workspaces;
    if let Some(pnpm) = read("pnpm-workspace.yaml") {
        workspaces.extend(pnpm_packages(&pnpm));
    }

    ProjectFacts {
        description: package
            .description
            .or(cargo.description)
            .or_else(|| read("README.md").as_deref().and_then(readme_tagline)),
        scripts: package.scripts,
        key_deps,
        workspaces,
        node_version: read(".nvmrc")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .or(package.node_engine),
        last_commit_secs_ago,
        dirty,
    }
}

const GIT_TIMEOUT: Duration = Duration::from_secs(2);

fn git_facts(root: &Path) -> (Option<u64>, Option<bool>) {
    let age = crate::exec::run(
        "git",
        &["log", "-1", "--format=%ct"],
        Some(root),
        GIT_TIMEOUT,
    )
    .and_then(|out| out.trim().parse::<u64>().ok())
    .map(|committed| now_secs().saturating_sub(committed));
    let dirty = crate::exec::run(
        "git",
        &["status", "--porcelain", "-uno"],
        Some(root),
        GIT_TIMEOUT,
    )
    .map(|out| !out.trim().is_empty());
    (age, dirty)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[derive(Default)]
struct PackageFacts {
    description: Option<String>,
    scripts: Vec<Script>,
    key_deps: Vec<KeyDep>,
    workspaces: Vec<String>,
    node_engine: Option<String>,
}

/// The dev-facing scripts, in the order a developer reaches for them.
const SCRIPT_NAMES: [&str; 3] = ["dev", "start", "build"];

/// Exact-name allowlists (same lesson as the label tables: never substrings).
const JS_KEY_DEPS: [&str; 16] = [
    "next",
    "react",
    "vue",
    "svelte",
    "astro",
    "vite",
    "express",
    "fastify",
    "hono",
    "prisma",
    "drizzle-orm",
    "tailwindcss",
    "typescript",
    "convex",
    "nuxt",
    "@remix-run/react",
];

const RUST_KEY_DEPS: [&str; 9] = [
    "axum",
    "tokio",
    "serde",
    "clap",
    "actix-web",
    "rocket",
    "sqlx",
    "diesel",
    "rust-embed",
];

fn package_facts(json: &str) -> PackageFacts {
    let Ok(root) = serde_json::from_str::<Value>(json) else {
        return PackageFacts::default();
    };
    let str_field = |v: &Value| {
        v.as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
    };

    let scripts = SCRIPT_NAMES
        .iter()
        .filter_map(|&name| {
            str_field(&root["scripts"][name]).map(|command| Script {
                name: name.into(),
                command,
            })
        })
        .collect();

    let mut key_deps = Vec::new();
    for section in ["dependencies", "devDependencies"] {
        if let Some(deps) = root[section].as_object() {
            for &name in &JS_KEY_DEPS {
                if key_deps.iter().any(|d: &KeyDep| d.name == name) {
                    continue;
                }
                if let Some(version) = deps.get(name) {
                    key_deps.push(KeyDep {
                        name: name.into(),
                        version: str_field(version),
                    });
                }
            }
        }
    }

    // workspaces is either an array of globs or {"packages": [...]}
    let workspace_globs = root["workspaces"]
        .as_array()
        .or_else(|| root["workspaces"]["packages"].as_array());
    let workspaces = workspace_globs
        .map(|globs| globs.iter().filter_map(str_field).collect())
        .unwrap_or_default();

    PackageFacts {
        description: str_field(&root["description"]),
        scripts,
        key_deps,
        workspaces,
        node_engine: str_field(&root["engines"]["node"]),
    }
}

#[derive(Default)]
struct CargoFacts {
    description: Option<String>,
    key_deps: Vec<KeyDep>,
}

/// Line-level Cargo.toml read: the description under [package] and curated
/// names under [dependencies]. No toml crate for two fields.
fn cargo_facts(toml: &str) -> CargoFacts {
    let mut facts = CargoFacts::default();
    let mut section = "";
    for line in toml.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            section = line;
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        match section {
            "[package]" if key == "description" => {
                facts.description = quoted(value);
            }
            "[dependencies]" if RUST_KEY_DEPS.contains(&key) => {
                facts.key_deps.push(KeyDep {
                    name: key.into(),
                    version: quoted(value),
                });
            }
            _ => {}
        }
    }
    facts
}

/// First quoted string in a toml value: `"0.8"` or `{ version = "1", .. }`.
fn quoted(value: &str) -> Option<String> {
    let start = value.find('"')? + 1;
    let end = start + value[start..].find('"')?;
    let s = &value[start..end];
    (!s.is_empty()).then(|| s.to_string())
}

/// `packages:` list entries from pnpm-workspace.yaml - only that block;
/// the file also carries catalogs and other lists that are not workspaces.
fn pnpm_packages(yaml: &str) -> Vec<String> {
    let mut in_packages = false;
    let mut out = Vec::new();
    for line in yaml.lines() {
        let trimmed = line.trim();
        if let Some(entry) = trimmed.strip_prefix("- ") {
            if in_packages {
                let entry = entry.trim_matches(|c| c == '"' || c == '\'');
                if !entry.is_empty() {
                    out.push(entry.to_string());
                }
            }
        } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
            in_packages = trimmed == "packages:";
        }
    }
    out
}

const TAGLINE_MAX: usize = 140;

/// First content line of a README: headings, badges, and HTML skipped;
/// a leading blockquote (the common tagline shape) is unwrapped.
fn readme_tagline(readme: &str) -> Option<String> {
    let line = readme.lines().map(str::trim).find(|line| {
        !line.is_empty()
            && !line.starts_with('#')
            && !line.starts_with("[![")
            && !line.starts_with("![")
            && !line.starts_with('<')
            && !line.starts_with("---")
    })?;
    let line = line.strip_prefix('>').unwrap_or(line).trim();
    if line.is_empty() {
        return None;
    }
    let mut tagline = line.to_string();
    if tagline.len() > TAGLINE_MAX {
        let cut = tagline
            .char_indices()
            .take_while(|&(i, _)| i < TAGLINE_MAX)
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
        tagline.truncate(cut);
        tagline.push('\u{2026}');
    }
    Some(tagline)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_facts_read_the_dev_fields() {
        let json = r#"{
            "name": "startdev",
            "description": "the start.dev monorepo",
            "scripts": {"build": "turbo build", "dev": "turbo dev", "lint": "oxlint"},
            "dependencies": {"react": "^19.0.0", "left-pad": "1.0.0"},
            "devDependencies": {"typescript": "~5.6.2", "vite": "^6.0.1"},
            "workspaces": ["apps/*", "packages/*"],
            "engines": {"node": ">=22"}
        }"#;
        let facts = package_facts(json);
        assert_eq!(facts.description.as_deref(), Some("the start.dev monorepo"));
        let names: Vec<&str> = facts.scripts.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["dev", "build"],
            "dev-facing order, lint skipped"
        );
        assert_eq!(facts.scripts[0].command, "turbo dev");
        let deps: Vec<(&str, Option<&str>)> = facts
            .key_deps
            .iter()
            .map(|d| (d.name.as_str(), d.version.as_deref()))
            .collect();
        assert_eq!(
            deps,
            vec![
                ("react", Some("^19.0.0")),
                ("vite", Some("^6.0.1")),
                ("typescript", Some("~5.6.2")),
            ],
            "curated deps only, both sections, no duplicates"
        );
        assert_eq!(facts.workspaces, vec!["apps/*", "packages/*"]);
        assert_eq!(facts.node_engine.as_deref(), Some(">=22"));
    }

    #[test]
    fn package_facts_tolerate_missing_and_malformed_input() {
        assert!(package_facts("not json").description.is_none());
        let sparse = package_facts(r#"{"name": "x"}"#);
        assert!(sparse.description.is_none());
        assert!(sparse.scripts.is_empty());
        assert!(sparse.key_deps.is_empty());
        assert!(sparse.workspaces.is_empty());
        let object_workspaces = package_facts(r#"{"workspaces": {"packages": ["pkgs/*"]}}"#);
        assert_eq!(object_workspaces.workspaces, vec!["pkgs/*"]);
    }

    #[test]
    fn cargo_facts_read_description_and_curated_deps() {
        let toml = r#"
[package]
name = "portdoc"
description = "Local dev server control panel"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
open = "5"

[dev-dependencies]
serde = "1"
"#;
        let facts = cargo_facts(toml);
        assert_eq!(
            facts.description.as_deref(),
            Some("Local dev server control panel")
        );
        let deps: Vec<(&str, Option<&str>)> = facts
            .key_deps
            .iter()
            .map(|d| (d.name.as_str(), d.version.as_deref()))
            .collect();
        assert_eq!(
            deps,
            vec![("axum", Some("0.8")), ("tokio", Some("1"))],
            "curated names only, table values give their version, dev-deps skipped"
        );
    }

    #[test]
    fn readme_tagline_skips_noise_and_unwraps_blockquotes() {
        let readme = "# PortDoc\n\n[![CI](badge.svg)](x)\n<p align=\"center\">logo</p>\n\n> A local dev server control panel.\n\nMore prose.";
        assert_eq!(
            readme_tagline(readme).as_deref(),
            Some("A local dev server control panel.")
        );
        assert_eq!(readme_tagline("# Only a heading"), None);
        assert_eq!(readme_tagline(""), None);
        let long = format!("# t\n\n{}", "word ".repeat(60));
        let tagline = readme_tagline(&long).expect("long line still yields");
        assert!(tagline.chars().count() <= TAGLINE_MAX + 1);
        assert!(tagline.ends_with('\u{2026}'));
    }

    #[test]
    fn pnpm_packages_parse_only_the_packages_block() {
        let yaml = "packages:\n  - \"apps/*\"\n  - 'packages/*'\n  - tools\nonlyBuiltDependencies:\n  - esbuild\ncatalog:\n  - astro@7.0.4\n";
        assert_eq!(pnpm_packages(yaml), vec!["apps/*", "packages/*", "tools"]);
        assert!(pnpm_packages("").is_empty());
        assert!(
            pnpm_packages("overrides:\n  - thing\n").is_empty(),
            "lists outside packages: are not workspaces"
        );
    }

    #[test]
    fn real_repo_facts_for_portdoc() {
        let repo = std::env::current_dir().expect("test cwd");
        let facts = project_facts(&repo);
        assert_eq!(
            facts.description.as_deref(),
            Some("Local dev server control panel"),
            "Cargo.toml description flows through"
        );
        assert!(
            facts.key_deps.iter().any(|d| d.name == "axum"),
            "cargo deps detected"
        );
        assert!(
            facts.last_commit_secs_ago.is_some(),
            "this repo has commits"
        );
        assert!(facts.dirty.is_some(), "git status should answer here");
    }
}
