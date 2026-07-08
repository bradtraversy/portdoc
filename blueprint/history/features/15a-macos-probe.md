# Feature: macOS listening-port probe

**From build-plan:** feature 15a
**Status:** complete 2026-07-08. Steps 1-3 built and CI-proven (ubuntu +
macos matrix, `probe_sees_own_listener` passed on the macos-latest runner).
Step 4 (live trav-studio verification) deferred by decision at /complete:
merged on CI evidence; the live check must happen before 15c ships a release.

## Goal

A `probe/macos.rs` implementation behind the feature 4 boundary so a macOS
binary shows a real dashboard: listening TCP ports joined to owning PIDs with
process name, command, cwd, user, and start age, degrading to unknown owners
exactly like Linux. This unblocks the macOS half of the feature 15 release.

## Decision (route)

Native crates, not shell-out: **netstat2** for the socket table + pid
association (sysctl `pcblist` under the hood, sees all users' sockets without
root) and **sysinfo** for process metadata. Chosen over `lsof`/`netstat`
parsing because non-root `lsof` cannot see other users' sockets at all (a
regression vs Linux, where the socket always appears even when the owner is
unreadable), and because netstat2 also covers Windows, so 15b reuses it.

**Recorded fallback:** if netstat2 proves broken or unmaintained in practice
(validated at step 2), shell out to `netstat -anv -p tcp` plus `ps` through
the existing `exec.rs` deadline runner - the docker/git precedent.

Both deps are target-scoped (`[target.'cfg(target_os = "macos")'.dependencies]`)
so the Linux binary gains nothing.

## In scope

- GitHub Actions CI test matrix (ubuntu + macos) - the macOS test bed every
  later step relies on; 15b adds windows to the same matrix.
- `probe/macos.rs` implementing the `Probe` trait, probe name `"macos-netstat"`.
- Listening TCP sockets (v4 + v6), sorted by port (the boundary's stability
  guarantee), pid joined where readable.
- `ProcessInfo`: name, command, cwd, user, `started_secs_ago` via sysinfo.
- Unknown owners as a first-class case: pid join failure leaves the socket
  listed with `pid: None`; socket `uid`/`user` filled only if netstat2 exposes
  the owner, otherwise honestly `None`.
- Live verification on real hardware (trav-studio).

## Out of scope

- Windows probe (15b) and the release pipeline / installers (15c).
- UDP (the boundary is TCP-only everywhere).
- Elevated-privilege probing; root-owned process details stay unreadable, the
  Advanced tab diagnostics already explain that from uid facts.
- Any `DevSnapshot` or `ProbeOutput` shape change - none is needed; every
  field the macOS probe may miss is already optional.

## Build loop

Build one step at a time, never the whole feature at once.

1. Plan mode lays out the step before any code.
2. The AI implements just that step.
3. It shows the diff (not full files); you read it and understand it.
4. You approve, then choose whether to commit a checkpoint or roll straight on.

Never accept a step you haven't read. If a diff is too big to review, the step
was too big, so split it.

## Build steps

- [x] **Step 1 - CI test matrix** - `.github/workflows/ci.yml`: on push + PR,
  matrix `ubuntu-latest` / `macos-latest`; steps: checkout, Node setup,
  `npm ci && npm run build` in `web/` (rust-embed needs `web/dist` to exist at
  compile time), Rust toolchain, `cargo test`, `cargo clippy -- -D warnings`,
  `npm run lint` in `web/`. *Done when:* the workflow is green on both OSes for
  the feature branch push (macOS runs `cargo test` with zero probe: the
  "platform without a probe" path is itself exercised).
- [x] **Step 2 - probe skeleton on netstat2** - target-scoped deps; `MacProbe`
  in `probe/macos.rs` returning listening TCP sockets (local addr, port, pid
  where associated) sorted by port; `platform_probe()` returns it under
  `cfg(target_os = "macos")`; mod.rs platform tests gain macOS twins
  ("macos gets a probe", "probe runs and output is sorted"). *Done when:*
  macOS CI job passes with the new tests; Linux job unchanged. (Spec
  correction: local `cargo check --target aarch64-apple-darwin` is not
  possible - netstat2's build script runs bindgen against the macOS SDK's
  `libproc.h`, so the macOS CI job is the sole verifier.) Validated the
  netstat2 bet: `macos_probe_runs` exercised sysctl enumeration on the
  macos-latest runner and passed.
- [x] **Step 3 - process metadata and owner mapping** - sysinfo enrichment:
  name, command, cwd, user, `started_secs_ago` (from sysinfo start_time vs
  now); socket uid/user if available; pure mapping helpers (name selection,
  age computation, uid-to-user) unit-tested with constructed inputs; a
  macOS-gated `probe_sees_own_listener` integration test mirroring the Linux
  one (bind a listener, assert pid/name/command/cwd/user/age resolve for our
  own process). *Done when:* that test passes on the macOS CI runner and the
  pure-helper tests pass in the normal suite.
- [ ] **Step 4 (DEFERRED to before the 15c release) - live verification on trav-studio** - pull the branch on the
  Mac, `cargo run` with at least one real dev server up; dashboard shows it
  with port, pid, cwd-derived project grouping, and framework label; Advanced
  tab shows probe `"macos-netstat"` and honest unknown-owner reasons for
  other-user sockets. *Done when:* screenshot evidence from trav-studio,
  no panics, `/api/snapshot` refresh-stable.

## Files / areas

- `.github/workflows/ci.yml` - new
- `Cargo.toml` - macOS-target-scoped `netstat2`, `sysinfo`
- `src/probe/macos.rs` - new
- `src/probe/mod.rs` - `platform_probe()` macOS arm, module wiring, tests
- No frontend changes

## Data / contracts

- None changed. `ListeningSocket` / `ProcessInfo` / `ProbeOutput` are already
  optional-everywhere; `DevSnapshot` untouched.
- New probe name string `"macos-netstat"` surfaces in `/api/sockets` (`probe`
  field) - cosmetic, not contract-locked.

## Testing

- Test gate is on (`cargo test` in AGENTS.md). In-scope logic: the mapping
  helpers in `probe/macos.rs` (name selection, age computation, owner
  resolution) ship unit tests in the same diff; the macOS integration test
  (`probe_sees_own_listener` twin) runs on the macos CI runner.
- cfg-gated macOS tests cannot run on trav-dev - the CI matrix from step 1 is
  the enforcement mechanism, which is why it is step 1.
- Step 4 is screenshot + running-app evidence per the standards (no UI code
  changes, so no frontend tests).

## Notes for the AI

- Mirror `probe/linux.rs` structure and honesty rules: any unreadable piece
  degrades to `None`, never an error; a probe failure is a typed `ProbeError`.
- macOS has no 15-char comm truncation, so don't port `best_name`'s
  truncation-expansion logic blindly; prefer sysinfo's name with the
  executable basename as tiebreak, and keep the helper pure for testing.
- Report v4 and v6 listeners separately like Linux does; downstream already
  merges twins.
- `exec.rs` exists if the fallback route is needed; 2s deadline convention.
- Keep clippy clean; edition 2024; thiserror for errors; no `anyhow`.
- Project-plan §5 already names `sysinfo` in the stack (old drift) - this
  feature makes that line true; no doc fix needed here.
