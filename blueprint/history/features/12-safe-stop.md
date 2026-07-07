# Feature: Safe stop action

**From build-plan:** feature 12
**Status:** complete

## Goal

Stop a service from the UI, safely: a confirmation dialog shows exactly
what will be signaled (name, PID, command, cwd), graceful SIGTERM comes
first, the server verifies the port actually releases, and SIGKILL sits
behind a second, separate confirmation. First mutating endpoint in the
app - the guards are the feature.

## In scope

- **Action module** (`src/action.rs`, unix-gated like the probe split):
  - `terminate(pid, force)` - SIGTERM, or SIGKILL when `force`; thin
    safe wrapper over `libc::kill` (new Linux/unix target dependency
    `libc`, no shelling out)
  - `wait_released(check, attempts, interval)` - polls a closure until
    the listener is gone or attempts run out (pure control flow,
    testable without processes)
  - typed `StopError` via thiserror: `NotPermitted` (EPERM - root-owned),
    `NoSuchProcess` (already gone counts as success upstream), `Io`
- **Endpoint** `POST /api/stop`, body `{ service_id, pid, force }`:
  - re-probes before signaling and verifies the (service_id, pid, port)
    triple still matches a live listener - a stale UI or reused pid gets
    409 "service changed, refresh and retry", never a signal
  - refuses PortDoc itself (`pid == std::process::id()`) with 403 -
    server-side, not just the UI badge
  - refuses services without a pid (unknown owner) with 400
  - after signaling, polls the probe (6 x 500ms) for the (port, pid)
    listener to disappear; responds `{ outcome: "released" }` or
    `{ outcome: "still_listening" }` (200 - not an error, it is the
    signal for the force path)
  - EPERM surfaces as 403 with "permission denied - root-owned process"
  - non-unix platforms: 501
- **UI - one shared stop dialog** rendered at `App` level:
  - opens from three entry points: dashboard row Stop, Services table
    row Stop, and the stale callout's "Stop safely"
  - confirmation shows process name, PID, full command, cwd, port -
    the exact facts, mono type
  - Cancel / "Stop service" (graceful). On `released`: close + refresh.
    On `still_listening`: the dialog escalates to a distinct second
    confirmation - danger-styled "Force kill" - nothing automatic
  - errors (403/409) display in the dialog with a refresh suggestion
  - Stop stays disabled (with the existing tooltips) for self, docker
    rows, and unknown-owner rows

## Out of scope

- Stop-by-port without a known pid (unknown owners are 400 by design;
  feature 14's diagnostics own that story)
- Killing process groups/children (signal the one pid only)
- Docker container stop (feature 14; docker rows keep Stop disabled)
- Batch/multi-stop, conflict-tab "stop the holder" shortcuts (can ride
  the inspect drawer later)
- Any persistence or audit log

## Build steps

- [x] **Step 1 - action module** - `src/action.rs` with terminate,
  wait_released, StopError + tests: guards as pure logic, wait_released
  with counting closures, and integration tests that spawn own child
  processes (`sleep`) and verify TERM and KILL delivery. *Done when:*
  `cargo test` green.
- [x] **Step 2 - /api/stop endpoint** - verify handshake, self/no-pid
  guards, signal + release polling, status codes as spec'd. *Done when:*
  `cargo test`, `cargo clippy`, `cargo fmt --check` clean and curl
  evidence: stopping an own throwaway listener returns `released` and
  the port vanishes from the next snapshot; a self-stop attempt returns
  403; a stale-pid attempt returns 409.
- [x] **Step 3 - stop dialog UI** - shared dialog at App level, three
  entry points wired, graceful -> force escalation, refresh on success.
  *Done when:* `npm run lint` + `npm run build` pass and browser
  evidence shows the full flow on a throwaway listener.

## Files / areas

- `Cargo.toml` - `libc` as a unix target dependency
- `src/action.rs` - new module + tests
- `src/main.rs` - POST route, request/response types, handler
- `web/src/components/StopDialog.tsx` - new
- `web/src/App.tsx` - dialog state + refresh wiring
- `web/src/components/ServiceRow.tsx`, `ServicesTable.tsx`,
  `Callouts.tsx` - enable Stop entry points (prop-drilled callback)

## Data / contracts

- `DevSnapshot` untouched. `/api/stop` is new surface: request
  `{ service_id: string, pid: number, force: bool }`, response
  `{ outcome: "released" | "still_listening" }` or an error body
  `{ error: string }` with 400/403/409/500/501
- The verify handshake is the core safety contract: no signal is ever
  sent to a pid that does not currently own the claimed service

## Testing

No declared test command (gate off); `cargo test` runs as evidence.
Signal delivery is tested against child processes the test itself spawns
(sleep), never external ones. Acceptance stops ONLY a throwaway listener
started for the purpose (python http.server) - no real service on the
machine gets signaled. Browser evidence for the dialog flow; curl
evidence for the guard matrix.

## Notes for the AI

- The unsafe `libc::kill` call gets a one-line SAFETY comment; nothing
  else unsafe
- ESRCH after SIGTERM means the process died between probe and signal -
  that is success (released), not an error
- Keep the handler honest about timing: the poll holds the request for
  up to ~3s; that is acceptable for a local tool and simpler than a job
  queue
- The dialog is hand-built in the existing ui/ style (no Radix, no new
  deps); trap focus minimally, Escape cancels
- Do not enable Stop on conflict-tab rows this pass - one dialog, three
  entry points, no more

## Completion notes

- First mutating endpoint. Shipped as spec'd: action module, verify-handshake
  /api/stop, shared stop dialog with graceful->force escalation, three entry
  points (dashboard row, services table, stale callout)
- Acceptance evidence: 63 tests green (5 new incl. a spawned TERM-ignoring
  child escalated to KILL); guard matrix curl-verified (self 403, stale 409,
  unknown 409, no-owner 400); escalation contract curl-verified
  (still_listening -> force -> released); dialog visually confirmed by Brad
  (SIGTERM delivered, row cleared)
- Playwright MCP backend was wedged the whole session, so the dialog's visual
  flow rests on Brad's manual confirmation; endpoint fully automated-proven
- Conservative UI quirk (intended): any portdoc-named row is greyed by the
  UI, while the server only refuses the true std::process::id(); a stray
  second portdoc instance is un-stoppable from the UI by that safe default
- Added a `danger` Button variant and a `libc` unix target dependency
