# Feature: Developer labels

**From build-plan:** feature 8
**Status:** complete

## Goal

Rows and project groups get developer-meaningful labels: `framework` on
services (Next.js, Vite, Astro, Convex, and friends), `package_manager` and
`git_branch` on project groups, and honest process names that say `node` or
`next-server (v16.2.9)` instead of `MainThread` and `next-server (v1`. The
UI already renders all three fields; this feature only fills them.

## In scope

- **Better process names (probe)**: pure `best_name(comm, argv0_basename)`
  policy in `probe/linux.rs`:
  - comm at the kernel's 15-char cap whose expansion prefix-matches the
    process title (cmdline first arg) expands to the full title
    (`next-server (v1` -> `next-server (v16.2.9)`)
  - comm that shares no prefix with the argv0 basename in either direction
    is a renamed thread label; use the argv0 basename (`MainThread` ->
    `node`)
  - otherwise keep comm; empty cmdline (kernel threads) keeps comm
- **Framework labels (new flat module `src/label.rs`)**: pure
  `detect_framework(process_name, command)` over tokenized command
  basenames (extensions `.js`/`.mjs`/`.cjs` stripped), first match wins,
  specific tools before runtimes:
  - Next.js (`next`, `next-server*`), Vite (`vite`), Astro (`astro`),
    Remix (`remix`), Nuxt (`nuxt`, `nuxi`), React scripts
    (`react-scripts`), Convex (`convex`, `convex-local-backend`),
    Prisma Studio (`prisma` followed by `studio`), Express (`express`,
    best-effort - a library rarely shows in argv), Bun (`bun`, `bunx`),
    Postgres (`postgres`, `postmaster`), Redis (`redis-server`)
  - no generic "Node" label - only meaningful tools get badges; anything
    else stays unlabeled
- **Project labels (in `src/label.rs`)**, detected once per project root in
  the adapter's grouping stage:
  - `package_manager` from marker files at the root, most specific first:
    `bun.lockb`/`bun.lock` -> bun, `pnpm-lock.yaml`/`pnpm-workspace.yaml`
    -> pnpm, `yarn.lock` -> yarn, `package-lock.json`/`package.json` ->
    npm, `Cargo.toml`/`Cargo.lock` -> cargo
  - `git_branch` by reading `.git/HEAD` directly (no git subprocess):
    `ref: refs/heads/<branch>` -> branch; detached HEAD -> first 7 hex
    chars; a `.git` *file* (worktree, `gitdir: <path>`) follows one level;
    unreadable/absent -> absent
- Wiring: adapter sets `Service.framework` in `service_from` and both
  project fields in `group_projects`
- Tests: pure fns unit-tested (name policy, every framework rule plus a
  non-match, HEAD parsing incl. detached and malformed, package-manager
  precedence); real-fs self-test: this repo's root labels as cargo with a
  non-empty branch

## Out of scope

- UI changes of any kind - badges, columns, and header slots already exist
- URL/exposure refinement (feature 9), conflicts/stale (feature 10)
- Custom renames (config storage lands with feature 13; project names
  already cover the folder-name ask from chat)
- Running `git` or any subprocess; shelling out is banned here (per-request
  probe must stay fast)
- Framework detection from package.json contents (cmdline-only this pass)

## Build steps

- [x] **Step 1 - honest process names** - `best_name` policy + tests in
  `probe/linux.rs`; `process_info` uses it. *Done when:* `cargo test` green
  including the existing self-listener test.
- [x] **Step 2 - framework detection** - `src/label.rs` with
  `detect_framework` + table-driven tests; adapter sets `framework`.
  *Done when:* `cargo test` green.
- [x] **Step 3 - project labels** - `package_manager` + `git_branch` in
  `label.rs` (pure parsers + fs wrappers); adapter fills both in
  `group_projects`. *Done when:* `cargo test`, `cargo clippy`,
  `cargo fmt --check` clean; `portdoc --json` shows framework labels,
  package managers, and branches for this machine's real projects.

## Files / areas

- `src/probe/linux.rs` - name policy + tests
- `src/label.rs` - new module: framework + package manager + git branch
- `src/adapter.rs` - wire labels into services and groups
- `src/main.rs` - `mod label;`

## Data / contracts

- No shape changes: `Service.framework`, `ProjectGroup.package_manager`,
  `ProjectGroup.git_branch` are locked optionals, filled for the first time
- `process_name` policy change is a probe accuracy fix, not a contract
  change; the UI's protected-row check (`process_name === 'portdoc'`) still
  holds (comm and argv0 agree for portdoc)
- Labels are display vocabulary, not enums - new tools can be added to the
  table later without contract impact

## Testing

No declared test command (gate off); `cargo test` runs as evidence anyway.
Everything new is pure logic with real edge cases (truncation boundary,
detached HEAD, precedence ordering) plus fs wrappers proven by the repo
self-test. End-to-end proof is `--json` and badges appearing in the
browser.

## Notes for the AI

- No `unwrap`/`expect` outside tests; unreadable `.git`/lockfiles degrade
  to absent labels, never errors
- Match on token basenames, not raw substrings (`next` must not match
  `nextcloud`)
- Keep the framework table data-driven (patterns -> label) so feature
  additions are one-line diffs
- `git_branch` reads at most two small files; no caching needed beyond the
  per-snapshot group pass

## Completion notes

- Shipped as spec'd: best_name policy in the probe, data-driven framework
  table in src/label.rs, package_manager + git_branch on project groups
- Acceptance evidence: 44 tests green (14 new), clippy/fmt/oxlint clean;
  real branches (incl. slashed names like course/express-api-fundamentals),
  pm labels pnpm/bun/cargo/npm, framework badges for Next.js/Convex/Astro/
  Postgres; no MainThread anywhere; browser-verified, zero console errors
- Express detection is best-effort by design: the library rarely appears in
  argv, so only an explicit express token labels (the wrong test case that
  assumed otherwise became the non-match regression test)
- Post-build audit (Opus 4.8 subagent): zero findings; the two considered
  non-findings (PACKAGE_MANAGERS/PACKAGE_MARKERS overlap, commonRoot edge)
  were deliberately left alone as over-engineering
