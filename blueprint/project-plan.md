# Project Plan

> One of the two planning docs you provide. Answer each section in a line or two
> (a worksheet, not an essay). Draft it yourself or let the AI help you expand and
> sharpen it; either way, the content is yours to direct. When it's filled in, run
> `/overview` to generate the project overview from this plus `build-plan.md`.

## 1. Problem - What problem are we solving?

Developers often have several local dev servers, APIs, databases, Docker services,
and stale processes running at once, which makes port conflicts and forgotten
servers annoying to debug. PortDoc gives a calm local web control panel that
shows what is running, which project owns each port, what URL to open, and how to
stop stale services safely.

## 2. Users - Who is this for?

PortDoc is for web developers who constantly switch between local projects and
run tools like React, Next.js, Vite, Express, Node, Bun, Convex, Docker,
Postgres, Redis, and Prisma. The first target user is Brad: multiple repos under
`~/Code`, frequent local port conflicts, and a need for fast visual answers while
recording, debugging, or jumping between projects.

## 3. Features - What does the MVP need?

- Local web UI served from `127.0.0.1:7788`.
- CLI launcher with `portdoc`, `portdoc ui`, `--no-open`, `--port`, and `--json`.
- Snapshot API that feeds both the browser UI and JSON output.
- Dashboard grouped by project, with active services, ports, URLs, stale apps,
  LAN-visible services, and Docker hints.
- Services table with process, PID, command, cwd, user, project, framework,
  exposure, started age when available, and quick actions.
- Project root, package manager, git branch, and framework detection.
- Shared-port detection and stale-process heuristics for common dev workflows.
  (Decided 2026-07-07: bump-conflict inference was dropped - bind-time
  EADDRINUSE fights are unobservable from a snapshot, so PortDoc reports only
  the factual shared-port case as a neutral badge; port lookup is the
  EADDRINUSE debugging path.)
- Safe stop by service or port with confirmation, graceful stop first, forced stop
  only behind a second explicit confirmation, and copy kill command fallback.
- Inspect drawer, search, filters, and tabs for Dashboard, Projects, Services,
  Docker, and Advanced. (The Conflicts tab was retired 2026-07-07 with the
  conflicts rework.)
- Release path with single-binary builds and OS-specific installer scripts.

## 4. Data - What are we storing?

v0.1 stores no cloud data and sends no telemetry. The main data is an ephemeral
local `DevSnapshot`: listening ports, process metadata, project grouping,
framework labels, URLs, exposure labels, conflicts, stale hints, Docker hints, and
inspection details. Local-only config (first needed by build item 13b, ignore
service) stores ignored services and later UI preferences and trusted project
roots. Decided 2026-07-07: it lives as JSON in the platform config directory
via the `dirs` crate - `~/.config/portdoc/config.json` on Linux, Application
Support on macOS, AppData on Windows.

## 5. Tech - What stack are we using?

Rust single binary with `axum`, `tokio`, `clap`, `serde`, and `serde_json`.
Linux probing starts with `/proc`, `procfs`, `sysinfo`, and a platform abstraction
for later macOS and Windows support. The frontend is React, TypeScript, and Vite
in `web/`, with Tailwind CSS, shadcn/ui style components, TanStack Table, and
Lucide planned for the control panel UI. Static assets are embedded with
`rust-embed` (decided at build item 2; debug builds read `web/dist` from disk).

## 6. Monetize - How will this make money?

No monetization in v0.1. Build the open developer tool first. Possible future
paths are sponsorship, GitHub Sponsors, paid team features, or a pro layer, but
the core local dashboard should stay useful without accounts or cloud services.

## 7. UI/UX - How should this look and feel?

PortDoc should feel like a modern developer control panel, not a raw networking
or sysadmin tool. The default dashboard should be quiet, dense, and easy to scan:
project groups, service rows, badges, filters, quick actions, and clear
confirmations. Advanced networking data belongs behind an Advanced tab so the
main view stays focused on everyday dev questions like what is running, what URL
to open, why a port is taken, and what can be stopped safely.
