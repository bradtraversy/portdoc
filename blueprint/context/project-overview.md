# PortDoc - Project Overview

> A local dev server control panel: one Rust binary serving a web dashboard that
> shows what dev apps are running, which project owns each port, what URL to
> open, and how to stop stale services safely.

## Problem

Developers run several local dev servers, APIs, databases, Docker services, and
stale processes at once, so port conflicts and forgotten servers are annoying to
debug. PortDoc answers the everyday questions fast: what is running, what URL to
open, why a port is taken, and what can be stopped safely.

## Users

- Web developers who switch between local projects and run React, Next.js, Vite,
  Express, Node, Bun, Convex, Docker, Postgres, Redis, Prisma, and similar tools.
- First target user is Brad: many repos under `~/Code`, frequent port conflicts,
  needs fast visual answers while recording, debugging, or switching projects.
- No accounts or tiers; everything is local to the machine.

## Features

In build-plan order. The headline feature is the **project-grouped dashboard**
(item 3, made real by items 5-9).

1. **Mock snapshot contract** - lock the shared `DevSnapshot` shape; `/api/health`, `/api/snapshot` with mocked services, and `portdoc --json` printing the same snapshot.
2. **Embedded web shell** - production Vite build served from the Rust binary on `127.0.0.1:7788`, browser opened by default.
3. **Mock dashboard UI** - summary cards, project groups, and a services table driven by mocked snapshot data.
4. **Platform probe boundary** - probing abstraction with a Linux-first implementation path.
5. **Linux listening-port probe** - collect listening TCP ports, join to owning PIDs, attach process name, command, cwd, user.
6. **Snapshot adapter** - real probe output becomes `DevSnapshot`: unknown owners, predictable IDs, localhost URLs, refresh-safe responses.
7. **Project grouping** - detect project roots (`package.json`, `.git`, workspace files, lockfiles, cwd) and group services by project.
8. **Developer labels** - detect package manager, git branch, frameworks/runtimes (Next.js, Vite, React scripts, Express, Bun, Convex, Prisma Studio, Astro, Remix, Nuxt).
9. **URL and exposure labels** - classify local / LAN visible / Docker / unknown; generate local URLs for HTTP-looking services.
10. **Conflicts and stale hints** - port conflicts, stale dev servers, old project processes, expected-but-missing apps, conflict actions.
11. **Search and filters** - filters for framework, runtime, API, database, Docker, unknown, LAN visible, stale, conflict, plus text search.
12. **Safe stop action** - stop a service or port with confirmation: show PID/command/cwd first, graceful stop, verify port release, force kill only behind a second explicit confirmation.
13. **Inspect drawer and quick actions** - service details, open/copy URL, reveal project folder, copy kill command, ignore service, advanced process details.
14. **Docker and Advanced tabs** - Docker/Compose hints, raw socket details, JSON export, unknown-owner diagnostics (well-known-port hints like ":22 - usually SSH" plus why the owner is unreadable), desktop app labels (VS Code, Discord) extending the feature 8 vocabulary, kept out of the main dashboard.
15. **Release and install path** - Linux/macOS/Windows release builds, checksums, `install.sh`, `install.ps1`, README docs, toward one-command installs from `portdoc.dev`.

## Data model

Ephemeral only in v0.1: the `DevSnapshot` is built per request from live system
probing, never persisted, and no telemetry is sent. Field names below are the
working shape; **feature 1 locks the JSON contract** and every later feature
(UI, probes, `--json`) depends on it, so change it in feature 1 or not at all.

### DevSnapshot

- `generated_at` (timestamp) - when the probe ran
- `services` (Service[]) - every detected listening service
- `projects` (ProjectGroup[]) - services grouped by owning project
- `conflicts` (Conflict[]) - detected port conflicts
- `docker_hints` (DockerHint[]) - Docker/Compose context

### Service

- `id` (string) - predictable across refreshes (feature 6)
- `port` (number) - listening TCP port
- `pid` (number, optional) - owner PID when resolvable; unknown owners allowed
- `process_name`, `command`, `cwd`, `user` (strings, optional) - process metadata
- `project_id` (string, optional) - owning ProjectGroup
- `framework` (string, optional) - detected label such as "Next.js" or "Vite"
- `exposure` (`local` | `lan` | `docker` | `unknown`) - who can reach it
- `url` (string, optional) - clickable local URL for HTTP-looking services
- `started_age` (string/duration, optional) - when available
- `stale` (bool + reason, optional) - stale-process heuristic result

### ProjectGroup

- `id`, `name` (strings) - derived from the project root
- `root` (path) - detected from `package.json`, `.git`, workspace files, lockfiles, cwd
- `package_manager` (string, optional)
- `git_branch` (string, optional)
- `service_ids` (string[]) - member services

### Conflict

- `port` (number) - the contested port
- `service_ids` (string[]) - contenders
- `hint` (string) - what happened and the suggested action

### DockerHint

- container/compose identification tied to a service or port

> TODO (later, local-only config): ignored services, UI preferences, trusted
> project roots. File location and format undecided; first needed by feature 13
> ("ignore service").

## Tech stack

- **Rust** (edition 2024) - single-binary backend in `src/`
- **axum** - HTTP server for the API and the embedded UI
- **tokio** - async runtime
- **clap** - CLI: `portdoc`, `portdoc ui`, `--no-open`, `--port`, `--json`
- **serde / serde_json** - `DevSnapshot` serialization (add with feature 1)
- **/proc, procfs** - Linux probing behind a platform abstraction (the `probe/` module) for later macOS/Windows support
- **rust-embed** - embeds `web/dist` in release builds (decided at feature 2); debug builds read it from disk
- **React + TypeScript + Vite** - frontend in `web/`
- **Tailwind CSS, shadcn/ui-style components, TanStack Table, Lucide** - the control panel UI stack, installed at feature 3

## Monetization

Not in v0.1. Open developer tool first; possible later paths are sponsorship,
GitHub Sponsors, paid team features, or a pro layer. The core local dashboard
stays useful without accounts or cloud services.

## UI/UX

A quiet, dense, scannable developer control panel, not a raw sysadmin tool.
Single-page app served at `http://127.0.0.1:7788` with tabs:

- **Dashboard** - default view: summary cards, project groups, active services, conflicts, stale apps, LAN-visible services, Docker hints
- **Projects** - services grouped by owning project
- **Services** - full table: process, PID, command, cwd, user, project, framework, exposure, started age, quick actions
- **Conflicts** - port conflicts and conflict-focused actions
- **Docker** - container/Compose view
- **Advanced** - raw sockets, JSON export, unknown-owner diagnostics

API surface: `/api/health`, `/api/snapshot`. Destructive actions (stop/kill)
always confirm, graceful first, force only behind a second explicit yes.

## Open questions

> Resolve in the plans, then re-run /overview.

- **macOS/Windows gap:** feature 15 ships macOS and Windows release builds, but no build item implements probing for those platforms (feature 4 is explicitly Linux-first). Either add probe items for those platforms before 15, or scope 15's installers to Linux-first.
- **Local config storage:** "ignore service" (feature 13) implies persisted local config, but its location/format is undecided.

> Resolved: `portdoc ui` shipped at feature 2 as an explicit alias of the
> default launch command.
