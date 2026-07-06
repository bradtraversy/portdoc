# Feature: Project grouping

**From build-plan:** feature 7
**Status:** complete

## Goal

Services get grouped by owning project: detect each service's project root by
walking up from its cwd, name the project after the root folder, and emit real
`ProjectGroup` entries with member `service_ids`. The dashboard's Projects
section stops saying "Ungrouped" for everything and shows groups like
`certificreate`, `startdev`, and `ai-skills-directory` (decided in chat: the
folder basename is the name, not the package.json `name` field).

## In scope

- New flat module `src/project.rs`: root detection as pure walk-up logic
  (testable with a fake marker lookup), plus the real filesystem lookup
- Detection rule, walking up from the service's `cwd` toward the filesystem
  root, stopping before the home directory:
  - the first directory containing `.git` (dir or file, so worktrees count)
    is the project root - the repo wins over nearer package markers, which
    keeps monorepo members (`apps/web`) grouped under the repo
  - otherwise the nearest directory with a package marker: `package.json`,
    `Cargo.toml`, `pyproject.toml`, `go.mod`, a lockfile (`package-lock.json`,
    `yarn.lock`, `pnpm-lock.yaml`, `bun.lockb`, `bun.lock`, `Cargo.lock`,
    `uv.lock`), or a workspace file (`pnpm-workspace.yaml`, `turbo.json`,
    `nx.json`, `lerna.json`)
  - the home directory itself and anything at or above it is never a root
    (a dotfiles `~/.git` must not swallow every service)
  - no cwd or no marker found -> service stays ungrouped
- Grouping in the adapter: cwd -> root resolved once per distinct cwd per
  snapshot; one `ProjectGroup` per distinct root; `project_id` set on member
  services; groups sorted by name; `service_ids` in service (port) order
- Project identity: `id` = `proj-{slug(basename)}`; if two distinct roots
  share a slug, all colliding projects use `proj-{slug(parent)}-{slug(basename)}`;
  `name` = raw folder basename; `root` displayed `~`-shortened when under
  home (matches the prototype look), absolute otherwise
- Tests: pure walk logic against fake directory trees (repo beats nearer
  package marker, package-only project, home boundary, no markers, marker in
  cwd itself); one real-fs self-test (the test binary's cwd is this repo, so
  detection from `src/` must land on the portdoc root); adapter grouping
  tests (shared root, id collision, ungrouped, sorted output)

## Out of scope

- `package_manager` and `git_branch` on `ProjectGroup` (feature 8) - both
  stay absent; the UI already renders them conditionally
- Framework labels (feature 8), exposure/URL refinement (feature 9),
  conflicts/stale (feature 10), Docker grouping (feature 14)
- Any UI change - `ProjectGroups.tsx` already renders groups + Ungrouped
- Watching or caching across requests - detection stays per-snapshot
- Canonicalizing symlinked cwds (`/proc` already resolves them)

## Build steps

- [x] **Step 1 - root detection** - `src/project.rs` with the walk-up rule
  as pure logic over an injected marker lookup, the real fs lookup, and the
  home boundary. Unit tests on fake trees plus the real-fs self-test.
  *Done when:* `cargo test` green.
- [x] **Step 2 - adapter grouping** - resolve roots for all service cwds,
  build sorted `ProjectGroup`s with slug ids (collision rule), `~`-shortened
  root, and set `project_id` on services. Unit tests for grouping, ids,
  collisions, and ungrouped passthrough. *Done when:* `cargo test`,
  `cargo clippy`, `cargo fmt --check` clean and `portdoc --json` shows real
  projects with member services on this machine.

## Files / areas

- `src/project.rs` - new module + tests
- `src/adapter.rs` - grouping stage + tests; `projects` stops being empty
- `src/main.rs` - `mod project;` declaration only

## Data / contracts

- `DevSnapshot` shape unchanged (locked): `ProjectGroup { id, name, root,
  service_ids }` with `package_manager`/`git_branch` omitted until feature 8
- `Service.project_id` points at the owning group; both sides stay in sync
  (every `project_id` appears in exactly one group's `service_ids`)
- IDs predictable across refreshes: same root -> same id (slug of basename,
  parent-qualified only on collision)
- `Service.cwd` stays the raw absolute path (feature 6 decision); only
  `ProjectGroup.root` gets the `~` display shortening

## Testing

No declared test command (gate off); `cargo test` runs as evidence anyway.
Root detection and grouping are pure logic with real edge cases (monorepo
subdir, home boundary, slug collision) - unit tested against fake trees, no
tempdir fixtures needed. The real-fs self-test proves the fs lookup against
this repo. End-to-end proof is `--json` projects plus the dashboard grouping
this machine's real dev servers.

## Notes for the AI

- No `unwrap`/`expect` outside tests; unreadable directories during marker
  checks degrade to "no marker", never a probe failure
- Keep the walk bounded: stop before home or filesystem root, whichever
  comes first; cap is implicit (path components), no depth constant needed
- Resolve HOME once per snapshot (std::env::var_os), not per service
- The slug helper already exists in `adapter.rs`; reuse it rather than
  duplicating (move it if project.rs needs it too)

## Completion notes

- Shipped as spec'd; no contract changes, no UI changes needed
- Acceptance evidence: 30 tests green (13 new), clippy/fmt clean; 6 real
  projects on trav-dev (certificreate, startdev, ai-skills-directory,
  bradtraversy.dev, portdoc, pylance extension) with 17 of 31 services
  grouped; membership cross-references verified; project ids and membership
  stable across API refreshes; dashboard browser-verified with zero console
  errors ("32 across 6 projects")
- Known quirk (by design): VS Code extension folders with package.json group
  as projects (ms-python.vscode-pylance-...); editor noise belongs to
  feature 11 filters or feature 13 ignore, not a path denylist here
