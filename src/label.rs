//! Developer-facing labels (feature 8): framework detection from command
//! lines, plus package manager and git branch detection at project roots.
//! Feature 14a extends the vocabulary with desktop apps (VS Code, Discord,
//! browsers) so their helper processes stop reading as mystery rows.

use std::path::Path;

/// Ordered by specificity: tools before runtimes, so "bunx vite" labels as
/// Vite. Identifiers match command-token basenames with js extensions
/// stripped, never raw substrings.
const FRAMEWORKS: [(&str, &[&str]); 11] = [
    ("Next.js", &["next", "next-server"]),
    ("Vite", &["vite"]),
    ("Astro", &["astro"]),
    ("Remix", &["remix"]),
    ("Nuxt", &["nuxt", "nuxi"]),
    ("React scripts", &["react-scripts"]),
    ("Convex", &["convex", "convex-local-backend"]),
    ("Express", &["express"]),
    ("Bun", &["bun", "bunx"]),
    ("Postgres", &["postgres", "postmaster"]),
    ("Redis", &["redis-server"]),
];

/// Desktop apps whose helpers hold local ports. Fallback vocabulary: any
/// framework match wins first. Path-valued flags carry signal through the
/// basename cleaning ("--user-data-dir=.../.config/discord" -> "discord").
const DESKTOP_APPS: [(&str, &[&str]); 9] = [
    ("VS Code", &["code", "code-insiders"]),
    ("Discord", &["discord"]),
    ("Chrome", &["chrome"]),
    ("Chromium", &["chromium", "chromium-browser"]),
    ("Firefox", &["firefox"]),
    ("Brave", &["brave", "brave-browser"]),
    ("Slack", &["slack"]),
    ("Spotify", &["spotify"]),
    ("Electron", &["electron"]),
];

pub fn detect_framework(name: Option<&str>, command: Option<&str>) -> Option<String> {
    let tokens: Vec<String> = name
        .into_iter()
        .chain(command)
        .flat_map(str::split_whitespace)
        .map(clean_token)
        .collect();

    // the one two-word tool; both tokens must appear
    if tokens.iter().any(|t| t == "prisma") && tokens.iter().any(|t| t == "studio") {
        return Some("Prisma Studio".into());
    }

    FRAMEWORKS
        .iter()
        .chain(DESKTOP_APPS.iter())
        .find(|(_, ids)| ids.iter().any(|id| tokens.iter().any(|t| t == id)))
        .map(|(label, _)| (*label).into())
}

/// Path basename, lowercased, with node-ish extensions stripped
/// (".../astro/bin/astro.mjs" -> "astro").
fn clean_token(token: &str) -> String {
    let base = token.rsplit('/').next().unwrap_or(token).to_lowercase();
    for ext in [".mjs", ".cjs", ".js"] {
        if let Some(stripped) = base.strip_suffix(ext) {
            return stripped.to_string();
        }
    }
    base
}

/// Non-HTTP signals that should kill URL generation. Anything not caught
/// here is treated as HTTP-looking - an unknown :8080 is more likely a dev
/// server than not. 631 (CUPS) is deliberately absent: it serves a web UI.
const NON_HTTP_PORTS: [u16; 18] = [
    22, 25, 53, 110, 143, 465, 587, 993, 995, 1433, 2049, 3306, 5432, 5672, 6379, 9092, 11211,
    27017,
];
const NON_HTTP_FRAMEWORKS: [&str; 2] = ["Postgres", "Redis"];
const NON_HTTP_PROCESSES: [&str; 3] = ["sshd", "dnsmasq", "systemd-resolved"];

/// Frameworks that are dev servers - the only things the stale heuristic
/// will accuse. Databases, editors, and unlabeled processes run long
/// legitimately.
const DEV_SERVERS: [&str; 6] = ["Next.js", "Vite", "Astro", "Remix", "Nuxt", "React scripts"];

pub fn is_dev_server(framework: &str) -> bool {
    DEV_SERVERS.contains(&framework)
}

pub fn http_looking(port: u16, process_name: Option<&str>, framework: Option<&str>) -> bool {
    if NON_HTTP_PORTS.contains(&port) {
        return false;
    }
    if framework.is_some_and(|f| NON_HTTP_FRAMEWORKS.contains(&f)) {
        return false;
    }
    if process_name.is_some_and(|n| NON_HTTP_PROCESSES.contains(&n)) {
        return false;
    }
    true
}

#[derive(Default)]
pub struct ProjectLabels {
    pub package_manager: Option<String>,
    pub git_branch: Option<String>,
}

pub fn project_labels(root: &Path) -> ProjectLabels {
    ProjectLabels {
        package_manager: package_manager(root),
        git_branch: git_branch(root),
    }
}

/// Lockfile/manifest -> package manager, most specific first (a root with
/// both bun.lock and package.json is a bun project).
const PACKAGE_MANAGERS: [(&str, &str); 9] = [
    ("bun.lockb", "bun"),
    ("bun.lock", "bun"),
    ("pnpm-lock.yaml", "pnpm"),
    ("pnpm-workspace.yaml", "pnpm"),
    ("yarn.lock", "yarn"),
    ("package-lock.json", "npm"),
    ("package.json", "npm"),
    ("Cargo.toml", "cargo"),
    ("Cargo.lock", "cargo"),
];

fn package_manager(root: &Path) -> Option<String> {
    PACKAGE_MANAGERS
        .iter()
        .find(|(file, _)| root.join(file).exists())
        .map(|(_, pm)| (*pm).into())
}

/// Read the branch from `.git/HEAD` without shelling out. A `.git` file
/// (worktree) is followed one level via its `gitdir:` pointer.
fn git_branch(root: &Path) -> Option<String> {
    let git = root.join(".git");
    let head_path = if git.is_file() {
        let pointer = std::fs::read_to_string(&git).ok()?;
        let dir = Path::new(pointer.strip_prefix("gitdir:")?.trim());
        let dir = if dir.is_absolute() {
            dir.to_path_buf()
        } else {
            root.join(dir)
        };
        dir.join("HEAD")
    } else {
        git.join("HEAD")
    };
    parse_head(&std::fs::read_to_string(head_path).ok()?)
}

/// "ref: refs/heads/main" -> "main"; detached HEAD -> short hash.
fn parse_head(content: &str) -> Option<String> {
    let line = content.lines().next()?.trim();
    if let Some(reference) = line.strip_prefix("ref:") {
        let reference = reference.trim();
        return reference
            .strip_prefix("refs/heads/")
            .map(str::to_string)
            .or_else(|| reference.rsplit('/').next().map(str::to_string));
    }
    (line.len() >= 7 && line.chars().all(|c| c.is_ascii_hexdigit())).then(|| line[..7].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detect(command: &str) -> Option<String> {
        detect_framework(None, Some(command))
    }

    #[test]
    fn detects_the_named_tools_from_real_command_shapes() {
        let cases = [
            ("node /repo/node_modules/.bin/next dev", "Next.js"),
            ("next-server (v16.2.9)", "Next.js"),
            ("node /repo/node_modules/.bin/vite", "Vite"),
            (
                "node /repo/node_modules/astro/bin/astro.mjs preview",
                "Astro",
            ),
            ("node /repo/node_modules/.bin/remix dev", "Remix"),
            ("node /repo/node_modules/.bin/nuxt dev", "Nuxt"),
            ("node /repo/node_modules/.bin/nuxi dev", "Nuxt"),
            (
                "node /repo/node_modules/.bin/react-scripts start",
                "React scripts",
            ),
            ("node /repo/node_modules/.bin/convex dev", "Convex"),
            (
                "/home/x/.cache/convex/binaries/p-1/convex-local-backend --port 3210",
                "Convex",
            ),
            (
                "node /repo/node_modules/.bin/prisma studio",
                "Prisma Studio",
            ),
            ("node /repo/node_modules/.bin/express", "Express"),
            ("bun run dev", "Bun"),
            ("bunx serve", "Bun"),
            ("/usr/lib/postgresql/16/bin/postgres -D /data", "Postgres"),
            ("redis-server *:6379", "Redis"),
        ];
        for (command, expected) in cases {
            assert_eq!(detect(command).as_deref(), Some(expected), "for: {command}");
        }
    }

    #[test]
    fn specific_tools_beat_runtimes() {
        assert_eq!(detect("bunx vite dev").as_deref(), Some("Vite"));
    }

    #[test]
    fn detects_desktop_apps_from_real_command_shapes() {
        let cases = [
            (
                Some("code"),
                Some("/snap/code/247/usr/share/code/code /home/brad/.vscode/extensions/ms-python.vscode-pylance-2026.2.1/dist/server.bundle.js --node-ipc"),
                "VS Code",
            ),
            (
                Some("exe"),
                Some("/proc/self/exe --type=renderer --user-data-dir=/home/brad/.config/discord --app-path=/home/brad/.config/discord/app-1.0.143/resources/app.asar"),
                "Discord",
            ),
            (Some("chrome"), Some("/opt/google/chrome/chrome --type=utility"), "Chrome"),
            (Some("firefox"), Some("/usr/lib/firefox/firefox -new-instance"), "Firefox"),
            (Some("slack"), Some("/usr/lib/slack/slack --type=renderer"), "Slack"),
            (None, Some("/usr/bin/electron /home/brad/app/main.js"), "Electron"),
        ];
        for (name, command, expected) in cases {
            assert_eq!(
                detect_framework(name, command).as_deref(),
                Some(expected),
                "for: {command:?}"
            );
        }
    }

    #[test]
    fn frameworks_beat_desktop_apps() {
        assert_eq!(
            detect_framework(
                Some("code"),
                Some("/snap/code/247/usr/share/code/code /repo/node_modules/.bin/vite dev"),
            )
            .as_deref(),
            Some("Vite"),
            "a framework token wins even when an app token is present"
        );
    }

    #[test]
    fn desktop_apps_match_basenames_not_substrings() {
        assert_eq!(detect("node /opt/discord-clone/server.js"), None);
        assert_eq!(detect("node /home/brad/chrome-extension-api/index.js"), None);
        assert_eq!(detect("ssh -L 8642:localhost:8642 trav-ai"), None);
    }

    #[test]
    fn prisma_without_studio_is_not_labeled() {
        assert_eq!(
            detect("node /repo/node_modules/.bin/prisma migrate dev"),
            None
        );
    }

    #[test]
    fn tokens_match_basenames_not_substrings() {
        assert_eq!(detect("/opt/nextcloud/server --port 8080"), None);
        assert_eq!(detect("node /home/brad/next-project/server.js"), None);
        assert_eq!(detect("python3 -m http.server 8123"), None);
        assert_eq!(
            detect("node express-server.js"),
            None,
            "an express app script is not the express token"
        );
    }

    #[test]
    fn name_alone_can_carry_the_signal() {
        assert_eq!(
            detect_framework(Some("next-server (v16.2.9)"), None).as_deref(),
            Some("Next.js")
        );
        assert_eq!(detect_framework(Some("node"), None), None);
        assert_eq!(detect_framework(None, None), None);
    }

    #[test]
    fn http_looking_denies_known_non_http_signals() {
        assert!(!http_looking(22, None, None), "ssh port");
        assert!(!http_looking(5432, None, None), "postgres port");
        assert!(
            !http_looking(54329, Some("postgres"), Some("Postgres")),
            "postgres framework on an odd port"
        );
        assert!(!http_looking(6380, None, Some("Redis")), "redis framework");
        assert!(
            !http_looking(2222, Some("sshd"), None),
            "sshd on an odd port"
        );
    }

    #[test]
    fn http_looking_defaults_to_yes_for_dev_servers_and_unknowns() {
        assert!(http_looking(3000, Some("node"), Some("Next.js")));
        assert!(http_looking(5173, Some("node"), Some("Vite")));
        assert!(http_looking(8080, None, None), "unknown wildcard listener");
        assert!(http_looking(631, None, None), "CUPS serves a real web UI");
    }

    #[test]
    fn parse_head_reads_branches_and_detached_shas() {
        assert_eq!(
            parse_head("ref: refs/heads/main\n").as_deref(),
            Some("main")
        );
        assert_eq!(
            parse_head("ref: refs/heads/feature/developer-labels\n").as_deref(),
            Some("feature/developer-labels"),
            "slashed branch names survive"
        );
        assert_eq!(
            parse_head("3f2a9c1d8e7b6a5f4d3c2b1a0e9f8d7c6b5a4e3d\n").as_deref(),
            Some("3f2a9c1")
        );
        assert_eq!(parse_head("not a head file"), None);
        assert_eq!(parse_head(""), None);
    }

    #[test]
    fn real_fs_labels_this_repo() {
        let repo = std::env::current_dir().expect("test cwd");
        assert_eq!(package_manager(&repo).as_deref(), Some("cargo"));
        assert_eq!(package_manager(&repo.join("web")).as_deref(), Some("npm"));
        assert_eq!(package_manager(&repo.join("src")), None);
        let branch = git_branch(&repo).expect("repo should have a branch");
        assert!(!branch.is_empty());
    }

    #[test]
    fn package_manager_precedence_and_worktree_git_file() {
        let base = std::env::temp_dir().join(format!("portdoc-label-test-{}", std::process::id()));
        let project = base.join("project");
        let gitdir = base.join("gitdir");
        std::fs::create_dir_all(&project).expect("mkdir project");
        std::fs::create_dir_all(&gitdir).expect("mkdir gitdir");

        std::fs::write(project.join("package.json"), "{}").expect("write");
        std::fs::write(project.join("bun.lock"), "").expect("write");
        assert_eq!(
            package_manager(&project).as_deref(),
            Some("bun"),
            "bun lockfile beats package.json"
        );

        std::fs::write(
            project.join(".git"),
            format!("gitdir: {}\n", gitdir.display()),
        )
        .expect("write .git file");
        std::fs::write(gitdir.join("HEAD"), "ref: refs/heads/wt-branch\n").expect("write HEAD");
        assert_eq!(git_branch(&project).as_deref(), Some("wt-branch"));

        std::fs::remove_dir_all(&base).expect("cleanup");
    }
}
