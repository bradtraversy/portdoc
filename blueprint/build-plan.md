# Build Plan

> One of the two planning docs you provide. Write it yourself or with the AI's help.
>
> Keep this as a checkbox list. Run `/feature` with no number to spec the next
> unchecked item, or `/feature 3` / `/feature "linux probe"` to pick a specific
> one. Completed features get checked off here, so the build plan doubles as the
> progress tracker.

- [x] 1. **Mock snapshot contract** - define the shared `DevSnapshot` shape, add `/api/health`, add `/api/snapshot` with mocked services, and make `portdoc --json` print the same snapshot.
- [x] 2. **Embedded web shell** - build the Vite app for production, serve it from the Rust binary, keep the local server on `127.0.0.1:7788`, and open the browser by default.
- [x] 3. **Mock dashboard UI** - replace the Vite starter screen with dashboard summary cards, project groups, and a services table driven by mocked snapshot data.
- [x] 4. **Platform probe boundary** - create the platform probing abstraction and a Linux-first implementation path without committing to macOS or Windows internals yet.
- [x] 5. **Linux listening-port probe** - collect listening TCP ports on Linux, join sockets to owning PIDs where possible, and attach process name, command, cwd, and user.
- [x] 6. **Snapshot adapter** - convert real probe output into `DevSnapshot`, including unknown owners, predictable IDs, localhost URLs, and refresh-safe API responses.
- [x] 7. **Project grouping** - detect project roots from `package.json`, `.git`, workspace files, lockfiles, and cwd, then group services by project.
- [x] 8. **Developer labels** - detect package manager, git branch, common frameworks, runtimes, and tools such as Next.js, Vite, React scripts, Express, Bun, Convex, Prisma Studio, Astro, Remix, and Nuxt.
- [x] 9. **URL and exposure labels** - classify services as local, LAN visible, Docker, or unknown, and generate useful local URLs for HTTP-looking services.
- [x] 10. **Conflicts and stale hints** - surface port conflicts, stale dev servers, old project processes, expected-but-missing apps, and conflict-focused actions.
- [x] 11. **Search and filters** - add service filters for framework, runtime, API, database, Docker, unknown, LAN visible, stale, and conflict, plus quick text search.
- [x] 12. **Safe stop action** - stop one service or port with confirmation, show exact PID/command/cwd before stopping, try graceful termination first, verify the port releases, and keep force kill behind a second explicit confirmation.
- [x] 13a. **Inspect drawer and port lookup** - a dedicated "look up a port" field on the dashboard opens an inspect drawer for that exact port (one listener, nothing, or a conflict's contenders); clicking any service row opens the same drawer. Drawer shows full details (PID, command, cwd, user, project, framework, exposure, age) and non-persistent quick actions: open URL, copy URL, copy kill command, reveal project folder, plus the feature 12 stop. No config storage needed.
- [x] 13b. **Ignore service** - hide a service from the dashboard, persisted in local config. Config decided 2026-07-07: JSON at the platform config dir via the `dirs` crate (`~/.config/portdoc/config.json` on Linux, Application Support on macOS, AppData on Windows).
- [x] 14. **Docker and Advanced tabs** - add Docker and Compose hints, a Docker tab, raw socket details, JSON export, unknown owner diagnostics, and Advanced data without crowding the dashboard.
  - [x] 14a. **Desktop app labels** - extend the feature 8 label vocabulary with desktop app identification (VS Code, Discord, browsers, and similar Electron apps) from process names and command paths, so editor and app internals stop reading as mystery rows.
  - [x] 14b. **Docker tab** - Docker and Compose hints tied to services or ports, and the Docker tab view. Mechanism decided 2026-07-08: shell out to the docker CLI (`docker ps --format json`), no socket client; missing or unreachable Docker degrades to empty hints.
  - [x] 14c. **Advanced tab** - raw socket details, JSON export, and unknown-owner diagnostics: well-known-port hints (":22 - usually SSH", ":53 - usually DNS", ":631 - usually printing") plus why an owner is unreadable (root-owned process, non-root probe).
- [ ] 15. **Release and install path** - add Linux, macOS, and Windows release builds, checksums, `install.sh`, `install.ps1`, README install docs, screenshots, and the path toward one-command installs from `portdoc.dev`.
- [ ] 16. **Enriched projects tab** - make the Projects tab more than the dashboard's grouped block: per-project header rollups (service count, stale/LAN counts, port summary), package.json facts (dev/build scripts, key deps, workspaces, node version), last commit age and dirty/clean indicator from local git, and project-level actions (open in editor, copy cd command, stop all services in the project). Needs an explicit contract decision at spec time: additive optional fields on the locked `ProjectGroup` shape. Probes stay cheap (single file reads, `git log -1`); intended to land before item 15 ships - pick it with `/feature 16`.
  - [x] 16a. **Project rollups** - frontend-only Projects tab enrichment from data already in the snapshot: per-project service count, stale and LAN counts, and clickable port chips; the Dashboard's grouped block stays compact.
  - [ ] 16b. **Project facts probe** - additive optional `ProjectGroup` fields (explicit contract decision): project description (package.json/Cargo.toml, README first line fallback), dev/build/start scripts, key deps, workspaces, node version, plus last commit age and dirty/clean from local git. Description renders inline in the Projects tab headers; full facts live in a project drawer (decided 2026-07-08: slide-over like the inspect drawer, no routing/detail page - the app has no router and the drawer pattern is established).
  - [ ] 16c. **Project actions** - open in editor (new endpoint), copy cd command, and stop-all-services-in-project built on the feature 12 confirmation contract.
