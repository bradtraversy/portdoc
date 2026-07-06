# Feature: Linux listening-port probe

**From build-plan:** feature 5
**Status:** complete

## Goal

Make `LinuxProbe` real: enumerate listening TCP sockets from `/proc`, join
them to owning PIDs where permissions allow, and attach process name,
command, cwd, user, and age. The probe still feeds nothing user-visible -
feature 6 adapts it into `DevSnapshot`. Correctness evidence comes from a
self-listener integration test plus pure-helper unit tests.

## In scope

- `procfs` crate as a Linux-only target dependency (named in the project
  plan's tech stack) for `/proc/net/tcp{,6}` and `/proc/<pid>/*` reading
- Socket enumeration: TCP v4 + v6 entries in LISTEN state -> `ListeningSocket`
- PID join: scan process fd tables for `socket:[inode]` targets; sockets
  whose owner can't be read stay `pid: None` (unknown owner is a feature,
  not an error)
- Process metadata: name (comm), command (joined cmdline), cwd, user
  (uid resolved via a pure `/etc/passwd` parser), `started_secs_ago`
  (uptime minus starttime/ticks, pure math helper)
- Tests: pure helpers unit-tested; one integration-style test that binds an
  ephemeral local listener, runs the probe, and finds its own socket with
  its own PID and metadata

## Out of scope

- Wiring probe output into `/api/snapshot` or `--json` (feature 6) - the app
  keeps serving mock data unchanged
- UDP, IPv6-specific handling beyond reading `tcp6` entries as-is
- Deduplicating dual-stack (v4+v6) listeners - the probe reports raw truth;
  merging is the adapter's call in feature 6
- macOS/Windows probes
- Declaring the `test` command in AGENTS.md (gate switch stays with `/tests`)

## Build steps

- [x] **Step 1 - socket enumeration** - add the `procfs` target dependency;
  `LinuxProbe::probe()` returns all LISTEN TCP v4/v6 sockets with address and
  port (`pid: None` for now). Test: bind `127.0.0.1:0` in-test, probe, assert
  the ephemeral port appears. *Done when:* `cargo test` green including the
  self-listener test.
- [x] **Step 2 - PID join** - build the socket-inode -> PID map from process
  fd tables, skipping unreadable processes; attach `pid`. Self-listener test
  extends to assert `pid == std::process::id()`. *Done when:* `cargo test`
  green.
- [x] **Step 3 - process metadata** - attach `ProcessInfo` (name, command,
  cwd, user, started_secs_ago) for joined PIDs; pure helpers for passwd
  parsing and age math with their own unit tests; self-listener test asserts
  its own name/command appear. *Done when:* `cargo test` green, `cargo
  clippy` and `cargo fmt --check` clean, and a probe run on this machine
  reports a plausible socket count (printed via the test).

## Files / areas

- `Cargo.toml` - `[target.'cfg(target_os = "linux")'.dependencies] procfs`
- `src/probe/linux.rs` - the real implementation + tests
- `src/probe/mod.rs` - only if a shared helper belongs at the boundary

## Data / contracts

- Fills the existing feature 4 shapes exactly; no shape changes and no
  `DevSnapshot` involvement
- Error posture: whole-probe `Err(ProbeError::Io)` only when `/proc/net`
  itself is unreadable; per-process read failures (permissions, races with
  exiting processes) degrade to `pid: None` / missing metadata, never a
  probe failure
- v4 and v6 entries on the same port are both reported (raw truth)

## Testing

No declared test command (gate off); `cargo test` runs as evidence anyway.
In-scope logic with real edge cases gets unit tests: passwd parsing (missing
uid, malformed lines), age math (zero ticks, future starttime clamps to 0).
The self-listener test is the end-to-end proof on a real kernel. Manual
signal for the packet: test output showing the machine's actual listener
count.

## Notes for the AI

- No `unwrap`/`expect` in probe code; per-entry failures are skipped or
  degrade to `None`
- Keep `linux.rs` functions small; pure helpers take plain inputs (string
  contents, numbers), not file paths, so they test without fixtures
- procfs API details verified against the compiled version, not memory
- Sorting: order output by port ascending so results are stable across runs
