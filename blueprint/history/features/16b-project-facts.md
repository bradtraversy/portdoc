# Current Feature

> **Generated file.** Holds the one feature or fix being built right now. Run
> `/feature <number-or-name>` to spec a build-plan feature, or `/fix "<bug>"` for
> an ad-hoc fix. Build one thing at a time; `/complete` archives it (to
> `blueprint/history/features/` or `blueprint/history/fixes/`) and resets this file.

## Feature 16b - Project facts probe

**Branch**: `feature/project-facts`

### Goal

Each project answers "what is this, what's it built with, how do I run it, and
what state is the repo in" - description inline in the Projects tab header, the
full facts in a new project drawer opened by clicking the header.

### Contract decision (additive extension of the locked ProjectGroup)

First deliberate extension of the feature-1 contract, decided 2026-07-08. All
fields optional and omitted when absent, so `--json` consumers and the mock-era
shape keep working:

- `description: string?` - package.json `description`, else Cargo.toml
  `[package] description`, else the README's first content line (truncated)
- `scripts: {name, command}[]?` - the `dev`/`start`/`build` scripts, that order
- `key_deps: {name, version?}[]?` - curated stack-defining deps (next, react,
  vite, astro, express, prisma, tailwindcss, typescript, convex... plus a Rust
  list: axum, tokio, serde, clap...) with versions as written
- `workspaces: string[]?` - workspace globs (package.json `workspaces`, or
  pnpm-workspace.yaml `packages` entries)
- `node_version: string?` - `.nvmrc`, else `engines.node`
- `last_commit_age: string?` - humanized, same units as `started_age`
- `dirty: bool?` - uncommitted tracked changes; absent when git state unknown

### How

- **`src/exec.rs`** - shared deadline-guarded command runner extracted from
  `docker.rs`'s `ps_output` (same spawn/try_wait/kill + reader-thread shape);
  docker and the git facts both use it so no probe can stall a snapshot.
- **`src/facts.rs`** - pure parsers plus one fs entry point
  `project_facts(root)`: package.json (description, scripts, deps, workspaces),
  Cargo.toml description + curated deps (line parse, no toml crate),
  pnpm-workspace.yaml packages (line parse), README first line, .nvmrc/engines.
  Git facts via `exec`: `git log -1 --format=%ct` -> age, `git status
  --porcelain -uno` -> dirty; either failing leaves the field absent.
- **`adapter.rs`** - fills the new fields per group (facts run once per
  distinct root, alongside the existing `project_labels`); `humanize_age`
  shared with the commit age.
- **`web/src/lib/types.ts`** - mirror the new optional fields.
- **UI**: description line in the detailed header (faint, truncated);
  clicking a project header (Projects tab only) opens **`ProjectDrawer.tsx`** -
  same slide-over pattern as the inspect drawer: description, root, branch +
  last commit + dirty badge, node version, workspaces, scripts (mono), key dep
  badges, member services as clickable rows into the inspect drawer. Dashboard
  behavior unchanged.

### Out of scope

- Project actions (open in editor, copy cd, stop all) - 16c, the drawer just
  makes room for them.
- Glob-expanding workspaces to count real packages; the globs themselves are
  shown.
- Parsing lockfiles for resolved versions - versions render as written
  ("^15.1.0").
- Any dashboard change.

### Build steps

- [x] 1. `src/exec.rs` deadline runner extracted; `docker.rs` refactored onto
  it. Done when `cargo test` (incl. existing docker tests) is green and
  clippy is clean.
- [x] 2. `src/facts.rs` pure parsers + `project_facts` fs/git entry point,
  unit tests for every parser (gate on): package.json shapes (missing fields,
  malformed json, workspaces as array and object), Cargo.toml description +
  deps, README fallback (skips headings/badges), .nvmrc over engines, curated
  dep matching.
- [x] 3. Contract wiring: new optional `ProjectGroup` fields (serde skips when
  absent), adapter fills them per root, `types.ts` mirrored, overview +
  project-plan data sections updated with the dated extension. Done when
  `portdoc --json` shows real facts for this repo (description from Cargo.toml,
  cargo deps, branch age, dirty flag) and tests are green.
- [x] 4. UI: header description line + `ProjectDrawer` with click wiring on
  the Projects tab. Done when the drawer renders this machine's real projects
  (portdoc, startdev) with facts and clickable member services, the Dashboard
  is unchanged, and lint + build are green.

### Testing

Rust gate on: every parser in `facts.rs` (the bulk of the feature's logic) and
the adapter fill. The exec runner and git shell-outs are integration-shaped -
covered by the live `--json` check. UI rides on browser + build.

### Notes for the AI

- Fields absent, never null or empty-string; `skip_serializing_if` everywhere.
- Git shell-outs run inside the existing `spawn_blocking` probe path with the
  exec deadline so a hung git (network fs) cannot stall `/api/snapshot`.
- Curated dep lists are allowlists of exact package names - no substring
  matching (same lesson as the label tables).
- The project drawer and inspect drawer must not stack; opening one closes the
  other.
