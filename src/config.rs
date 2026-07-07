//! Persisted local config (feature 13b): a small JSON file in the platform
//! config directory. Unreadable or malformed config degrades to the default
//! instead of failing; the file only matters once the user changes something.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub ignored_services: Vec<String>,
}

impl Config {
    /// Returns true when the config actually changed.
    pub fn set_ignored(&mut self, service_id: &str, ignored: bool) -> bool {
        let present = self.ignored_services.iter().any(|s| s == service_id);
        match (ignored, present) {
            (true, false) => {
                self.ignored_services.push(service_id.to_string());
                true
            }
            (false, true) => {
                self.ignored_services.retain(|s| s != service_id);
                true
            }
            _ => false,
        }
    }
}

/// `~/.config/portdoc/config.json` on Linux; the platform equivalent elsewhere.
pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("portdoc").join("config.json"))
}

pub fn load(path: &Path) -> Config {
    fs::read_to_string(path)
        .ok()
        .and_then(|body| serde_json::from_str(&body).ok())
        .unwrap_or_default()
}

/// Write-then-rename so a crash mid-save never leaves a truncated config.
pub fn save(path: &Path, config: &Config) -> io::Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidInput))?;
    fs::create_dir_all(dir)?;
    let body = serde_json::to_string_pretty(config).map_err(io::Error::other)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, body)?;
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_base(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("portdoc-config-test-{tag}-{}", std::process::id()))
    }

    #[test]
    fn missing_file_loads_default() {
        assert_eq!(load(Path::new("/nonexistent/config.json")), Config::default());
    }

    #[test]
    fn malformed_and_unknown_fields_degrade_gracefully() {
        let base = temp_base("malformed");
        std::fs::create_dir_all(&base).expect("mkdir");
        let path = base.join("config.json");

        std::fs::write(&path, "{ not json").expect("write");
        assert_eq!(load(&path), Config::default(), "malformed json is default");

        std::fs::write(
            &path,
            r#"{ "ignored_services": ["svc-1"], "future_key": true }"#,
        )
        .expect("write");
        assert_eq!(
            load(&path).ignored_services,
            vec!["svc-1"],
            "unknown keys are tolerated"
        );

        std::fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn save_round_trips_and_creates_the_directory() {
        let base = temp_base("roundtrip");
        let path = base.join("nested").join("config.json");

        let mut config = Config::default();
        config.set_ignored("svc-3000-node", true);
        save(&path, &config).expect("save");
        assert_eq!(load(&path), config);

        std::fs::remove_dir_all(&base).expect("cleanup");
    }

    #[test]
    fn set_ignored_dedupes_and_unignores() {
        let mut config = Config::default();
        assert!(config.set_ignored("a", true));
        assert!(!config.set_ignored("a", true), "second ignore is a no-op");
        assert_eq!(config.ignored_services, vec!["a"]);

        assert!(config.set_ignored("a", false));
        assert!(!config.set_ignored("a", false), "second unignore is a no-op");
        assert!(config.ignored_services.is_empty());
    }
}
