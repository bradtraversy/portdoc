//! Project root detection (feature 7): walk up from a service's cwd until
//! a repo or package marker names the project root.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    /// `.git` - the repo root wins over nearer package markers, so monorepo
    /// members group under the repo.
    Repo,
    /// A manifest, lockfile, or workspace file; nearest one wins when no
    /// repo is found.
    Package,
}

/// Pure walk-up over an injected marker lookup. The home directory itself
/// and anything at or above it is never a root (a dotfiles `~/.git` must
/// not swallow every service).
pub fn detect_root(
    cwd: &Path,
    home: Option<&Path>,
    marker_at: impl Fn(&Path) -> Option<Marker>,
) -> Option<PathBuf> {
    let mut nearest_package: Option<PathBuf> = None;

    for dir in cwd.ancestors() {
        if home.is_some_and(|h| dir == h) || dir.file_name().is_none() {
            break;
        }
        match marker_at(dir) {
            Some(Marker::Repo) => return Some(dir.to_path_buf()),
            Some(Marker::Package) => {
                nearest_package.get_or_insert_with(|| dir.to_path_buf());
            }
            None => {}
        }
    }

    nearest_package
}

const PACKAGE_MARKERS: [&str; 15] = [
    "package.json",
    "Cargo.toml",
    "pyproject.toml",
    "go.mod",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "bun.lockb",
    "bun.lock",
    "Cargo.lock",
    "uv.lock",
    "pnpm-workspace.yaml",
    "turbo.json",
    "nx.json",
    "lerna.json",
];

/// Real filesystem lookup; unreadable paths degrade to "no marker".
/// `.git` may be a file (worktrees), so `exists()` is the right check.
pub fn fs_marker(dir: &Path) -> Option<Marker> {
    if dir.join(".git").exists() {
        return Some(Marker::Repo);
    }
    PACKAGE_MARKERS
        .iter()
        .any(|m| dir.join(m).exists())
        .then_some(Marker::Package)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn tree(entries: &[(&str, Marker)]) -> impl Fn(&Path) -> Option<Marker> {
        let map: HashMap<PathBuf, Marker> = entries
            .iter()
            .map(|(p, m)| (PathBuf::from(p), *m))
            .collect();
        move |dir: &Path| map.get(dir).copied()
    }

    const HOME: &str = "/home/brad";

    fn detect(cwd: &str, marker_at: impl Fn(&Path) -> Option<Marker>) -> Option<PathBuf> {
        detect_root(Path::new(cwd), Some(Path::new(HOME)), marker_at)
    }

    #[test]
    fn repo_beats_nearer_package_marker() {
        let lookup = tree(&[
            ("/home/brad/Code/mono", Marker::Repo),
            ("/home/brad/Code/mono/apps/web", Marker::Package),
        ]);
        assert_eq!(
            detect("/home/brad/Code/mono/apps/web", lookup),
            Some(PathBuf::from("/home/brad/Code/mono"))
        );
    }

    #[test]
    fn nearest_package_marker_wins_without_a_repo() {
        let lookup = tree(&[("/home/brad/Code/scratch", Marker::Package)]);
        assert_eq!(
            detect("/home/brad/Code/scratch/sub/dir", lookup),
            Some(PathBuf::from("/home/brad/Code/scratch"))
        );
    }

    #[test]
    fn marker_in_the_cwd_itself_is_found() {
        let lookup = tree(&[("/home/brad/Code/app", Marker::Repo)]);
        assert_eq!(
            detect("/home/brad/Code/app", lookup),
            Some(PathBuf::from("/home/brad/Code/app"))
        );
    }

    #[test]
    fn home_and_above_are_never_roots() {
        // dotfiles repo at ~ plus a package.json in /home must both be ignored
        let lookup = tree(&[("/home/brad", Marker::Repo), ("/home", Marker::Package)]);
        assert_eq!(detect("/home/brad/Downloads", &lookup), None);
        assert_eq!(detect("/home/brad", &lookup), None);
    }

    #[test]
    fn filesystem_root_is_never_a_root() {
        let lookup = tree(&[("/", Marker::Package)]);
        assert_eq!(detect("/opt/thing", lookup), None);
    }

    #[test]
    fn no_markers_means_no_root() {
        assert_eq!(detect("/home/brad/Code/plain/dir", tree(&[])), None);
    }

    #[test]
    fn without_home_the_walk_still_stops_at_filesystem_root() {
        let lookup = tree(&[("/srv/app", Marker::Package)]);
        assert_eq!(
            detect_root(Path::new("/srv/app/sub"), None, lookup),
            Some(PathBuf::from("/srv/app"))
        );
    }

    #[test]
    fn real_fs_detects_this_repo_from_src() {
        let repo = std::env::current_dir().expect("test cwd");
        let home = std::env::var_os("HOME").map(PathBuf::from);
        let root = detect_root(&repo.join("src"), home.as_deref(), fs_marker)
            .expect("should detect the portdoc repo root");
        assert_eq!(root, repo);
        assert!(root.join("Cargo.toml").exists());
    }
}
