# Feature: Windows listening-port probe

**From build-plan:** feature 15b
**Status:** complete 2026-07-09. All four steps built via autopilot and
CI-proven on the ubuntu + macos + windows matrix; `probe_sees_own_listener`
passed on the windows-latest runner (91 tests green there, clippy
`-D warnings` clean). The local cross-check bet paid off: unlike the macOS
bindgen wall, `cargo clippy --target x86_64-pc-windows-msvc --all-targets`
works from Linux and caught a real Windows-only dead-code lint before push.
Live Windows verification deferred like 15a's step 4: checklist in the vault
Inbox (`PortDoc Windows Test Checklist.md`); Brad's first from-source attempt
hit the GNU-toolchain `dlltool.exe` error (fix documented there: switch to
MSVC). Must pass before 15c ships a release.
Known gaps flagged for 15c spec time: Windows stop returns a typed
"not supported", and VS Code extension sub-labels don't match backslash
marker paths.

## Goal

A `probe/windows.rs` implementation behind the feature 4 boundary so a Windows
binary shows a real dashboard: listening TCP ports joined to owning PIDs with
process name, command, cwd, user, and start age, degrading to unknown owners
exactly like Linux. With 15a this closes the platform half of feature 15; only
the 15c release pipeline remains.

## Decision (route)

Decided with 15a on 2026-07-08: **netstat2** for the socket table
(GetExtendedTcpTable under the hood, owning pid comes straight from the kernel
table) plus **sysinfo** for process metadata - the exact pair the macOS probe
already uses, which is why netstat2 was picked over lsof parsing in the first
place. Both deps move from macOS-only to a shared
`[target.'cfg(any(target_os = "macos", target_os = "windows"))'.dependencies]`
scope; Linux gains nothing.

**Recorded fallback:** if netstat2's Windows side proves broken in practice
(validated at step 2), shell out to `netstat -ano` through the existing
`exec.rs` deadline runner and join pids via sysinfo.

## In scope

- `windows-latest` added to the CI test matrix (the enforcement bed, mirroring
  15a's step-1-first rule).
- Making the existing test suite pass on Windows: the four signal-delivery
  tests in `action.rs` are unix-only and need gating (plus a Windows twin
  asserting `terminate` returns `Unsupported`); the `exec.rs` tests shell out
  to `echo`/`false`/`sleep`/`pwd` and need per-OS command fixtures.
- `probe/windows.rs` implementing the `Probe` trait, probe name
  `"windows-netstat"`.
- Listening TCP sockets (v4 + v6), sorted by port, pid joined from the kernel
  table; `ProcessInfo` (name, command, cwd, user, `started_secs_ago`) via
  sysinfo; anything unreadable degrades to `None`, never an error. Socket
  `uid` stays honestly `None` (no unix uid on Windows; user comes from the
  process join).
- Label token cleaning that understands Windows names: `clean_token` in
  `label.rs` strips `.exe` and splits basenames on `\` as well as `/`, so
  `node.exe` and `C:\Program Files\nodejs\node.exe` label like their unix
  twins.

## Out of scope

- The release pipeline and installers (15c).
- Windows stop support: `action.rs` already returns a typed `Unsupported`
  error on non-unix and the endpoint surfaces it; a real Windows terminate
  (taskkill/TerminateProcess, no graceful/force distinction exists there) is a
  design decision for 15c spec time or its own item. Flag it, don't build it.
- UDP (the boundary is TCP-only everywhere).
- Any `DevSnapshot` or `ProbeOutput` shape change - none is needed.
- Live Windows verification: no Windows hardware or VM is reachable from this
  session. CI's `probe_sees_own_listener` run is the automated evidence; the
  manual dashboard check happens on the Microsoft dev VM before 15c ships
  (carried in the vault, same deferral shape as 15a's step 4).

## Build loop

Build one step at a time, never the whole feature at once.

1. Plan mode lays out the step before any code.
2. The AI implements just that step.
3. It shows the diff (not full files); you read it and understand it.
4. You approve, then choose whether to commit a checkpoint or roll straight on.

Never accept a step you haven't read. If a diff is too big to review, the step
was too big, so split it.

## Build steps

- [x] **Step 1 - windows CI matrix + portable test suite** - add
  `windows-latest` to `.github/workflows/ci.yml`; gate the four signal tests
  in `action.rs` behind `#[cfg(unix)]` and add a `#[cfg(not(unix))]` test that
  `terminate` returns `Unsupported`; give `exec.rs` tests per-OS command
  fixtures (`cmd /C ...` on Windows, `std::env::temp_dir()` instead of
  `/tmp`). *Done when:* `cargo test` and clippy stay green on Linux locally,
  and the windows CI job is green at push time with zero probe (the
  "platform without a probe" path exercised, exactly like macOS in 15a
  step 1).
- [x] **Step 2 - probe skeleton on netstat2** - widen the `netstat2`/`sysinfo`
  target scope to macOS+Windows in `Cargo.toml`; `WindowsProbe` in
  `probe/windows.rs` returning listening TCP sockets (local addr, port, pid)
  sorted by port; `platform_probe()` returns it under
  `cfg(target_os = "windows")`; extend the `cfg_attr` dead-code allows in
  `mod.rs` to include windows; mod.rs platform tests gain Windows twins.
  *Done when:* Linux suite green locally; `cargo check --target
  x86_64-pc-windows-msvc` if the toolchain allows it (attempt it - unlike the
  macOS SDK bindgen wall, the Windows side may cross-check), otherwise the
  windows CI job at push time is the verifier.
- [x] **Step 3 - process metadata and owner mapping** - sysinfo enrichment
  mirroring `macos.rs` minus the mac quirks: no env-tail trim (KERN_PROCARGS2
  is a mac artifact) and no p_comm truncation-expansion (Windows names are not
  truncated); name, command, cwd, user, `started_secs_ago`; a windows-gated
  `probe_sees_own_listener` integration test mirroring the macOS one; pure
  helpers unit-tested. *Done when:* pure-helper tests pass in the normal
  suite; the integration test passes on the windows CI runner at push time.
- [x] **Step 4 - label token cleaning for Windows names** - `clean_token`
  strips a trailing `.exe` (before the script-extension pass) and takes the
  basename across both `/` and `\`; unit tests cover `node.exe`,
  `C:\Program Files\nodejs\node.exe server.js`, and that unix behavior is
  unchanged. *Done when:* new tests pass; existing label tests untouched and
  green.

## Files / areas

- `.github/workflows/ci.yml` - windows-latest matrix entry
- `src/action.rs`, `src/exec.rs` - test portability only, no source changes
  expected beyond cfg gates and fixtures
- `Cargo.toml` - netstat2 + sysinfo target scope widened
- `src/probe/windows.rs` - new
- `src/probe/mod.rs` - module wiring, platform_probe arm, cfg_attr updates,
  tests
- `src/label.rs` - `clean_token` only
- No frontend changes

## Data / contracts

- None changed. Every field the Windows probe may miss is already optional.
- New probe name string `"windows-netstat"` surfaces in `/api/sockets`
  (cosmetic, not contract-locked).

## Testing

- Test gate is on (`cargo test` in AGENTS.md). In-scope logic ships tests in
  the same diff: the exec fixtures keep all four exec tests running on every
  OS, the probe's pure helpers get unit tests, `clean_token` gets Windows
  cases, and `probe_sees_own_listener` gets a windows twin.
- cfg-gated Windows tests cannot run on trav-dev - the CI matrix is the
  enforcement mechanism, which is why it is step 1. Local `cargo check
  --target x86_64-pc-windows-msvc` is attempted as a cheaper compile-level
  check.
- Pushing the branch (to get windows CI evidence) needs Brad's yes; until
  then the Windows-side done-whens are "deferred to CI at push".

## Notes for the AI

- Mirror `probe/macos.rs` structure and honesty rules; skip `trim_env_tail`
  and `expand_name` - they encode mac-specific kernel behavior. Windows
  process names keep their `.exe`; the probe reports facts, `clean_token`
  handles labeling.
- netstat2's Windows table always carries the owning pid (even for system
  processes), but sysinfo may not be able to open the process (access
  denied on services); that is the unknown-owner degrade path - pid set,
  process `None`.
- Report v4 and v6 listeners separately like Linux and macOS; downstream
  merges twins.
- Keep clippy clean; edition 2024; thiserror; no `anyhow`.
