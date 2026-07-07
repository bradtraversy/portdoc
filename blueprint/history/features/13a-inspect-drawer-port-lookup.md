# Feature: Inspect drawer and port lookup

**From build-plan:** feature 13a
**Status:** complete

## Goal

Answer "what exactly is on this port, and let me act on it" directly. A
dedicated "look up a port" field on the dashboard opens an inspect drawer
for that exact port; clicking any service row opens the same drawer. The
drawer shows full details and non-persistent quick actions. No config
storage - that is 13b (ignore service).

## In scope

- **Inspect drawer** (`web/src/components/InspectDrawer.tsx`): a right-side
  panel over a scrim, opened at App level via a context (same pattern as
  the stop dialog). Content per service:
  - full details: port, PID, process name, command, cwd, user, project
    (name + root), framework, exposure, url, started age, stale reason,
    conflict membership - each shown only when present, mono for data
  - quick actions: Open URL, Copy URL, Copy kill command (`kill <pid>`),
    Reveal folder, and Stop (reuses the feature 12 dialog + `canStop`
    guard). Actions render disabled with a reason when not applicable
    (no url, no pid, no cwd, unstoppable)
  - Escape and scrim-click close; the drawer traps focus minimally
- **Drawer keying** handles all three port cases: one service (row click
  or a clean port), a conflict port (multiple contenders stacked under a
  "Port :N - 2 listeners" header), and nothing ("Nothing is listening on
  :N" empty state with a hint)
- **Dashboard port lookup** (in `DashboardView`): a "look up a port" number
  field; Enter (or the arrow button) resolves the port against the current
  snapshot and opens the drawer with that port's services (0, 1, or many).
  Invalid/empty input is a no-op
- **Row-click entry point**: clicking a `ServiceRow` (dashboard/conflicts)
  or a `ServicesTable` row opens the drawer for that one service; the
  hover action buttons keep working via `stopPropagation`
- **Reveal-folder endpoint** `POST /api/reveal` body `{ path }`: opens the
  path in the OS file manager via the existing `open` crate, but only after
  verifying the path exists and is a directory (400 otherwise); non-unix or
  failure returns a JSON error the drawer surfaces. Server owns this because
  a browser cannot open a file manager

## Out of scope

- **Ignore service** (13b) - needs persisted config, still undecided
- "Advanced process details" beyond what the snapshot already carries
  (raw sockets, fd tables) - that is feature 14's Advanced tab
- Editing/renaming anything from the drawer (rename was folded into the
  feature 8 project-name discussion; not this pass)
- Port lookup history, autocomplete, or fuzzy matching - exact port only
- Opening arbitrary paths: `/api/reveal` only accepts a path that resolves
  to an existing directory

## Build steps

- [x] **Step 1 - inspect drawer + row click** - `InspectDrawer.tsx`,
  `inspect.ts` context, rendered at App; details + frontend actions (Open,
  Copy URL, Copy kill command, Stop); row-click entry points in
  `ServiceRow` and `ServicesTable` with action buttons still working.
  *Done when:* `npm run lint` + `npm run build` pass and clicking a row
  opens a populated drawer in the browser.
- [ ] **Step 2 - dashboard port lookup** - "look up a port" field in
  `DashboardView`; resolve against the snapshot; open the drawer for the
  one / conflict / nothing cases. *Done when:* lint + build pass and typing
  a live port, a conflict port, and an empty port each open the right
  drawer state.
- [x] **Step 3 - reveal folder** - `POST /api/reveal` (dir-validated,
  `open` crate) + the drawer's Reveal folder button. *Done when:*
  `cargo test`, `cargo clippy`, `cargo fmt --check` clean; curl to
  `/api/reveal` with a real dir returns 200 and with a bogus path returns
  400; the drawer button is wired.
- [x] **Step 4 - framework-first display name** (added in review with Brad)
  - a shared `displayName(service)` in `derive.ts`: the framework when
  known, else the process name, else "unknown". Used as the primary label
  in `ServiceRow`, the `ServicesTable` service cell, and the `InspectDrawer`
  heading; the raw process name stays in each sub-line. Rows like an Astro
  server headline "Astro" instead of "node"; an unlabeled node process
  still reads "node". *Done when:* `npm run lint` + `npm run build` pass
  and the dashboard headlines dev servers by framework with the process
  name demoted to the sub-line.

## Files / areas

- `web/src/components/InspectDrawer.tsx` - new
- `web/src/lib/inspect.ts` - new: drawer context + open helpers
- `web/src/App.tsx` - drawer state + provider (beside the stop provider)
- `web/src/components/DashboardView.tsx` - port lookup field
- `web/src/components/ServiceRow.tsx`, `ServicesTable.tsx` - row-click open
- `web/src/lib/derive.ts` - a `servicesOnPort(snapshot, port)` helper
- `src/main.rs` - `POST /api/reveal` handler + request type

## Data / contracts

- `DevSnapshot` untouched. New endpoint: `POST /api/reveal` body
  `{ path: string }`, response `{}` on success or `{ error }` with
  400/500/501. It is an OS side effect (launch file manager), not a data
  mutation, and touches nothing on disk
- The kill command copied is exactly `kill <pid>` (graceful); the drawer
  notes force is `kill -9 <pid>` in the copy tooltip but copies graceful
- Drawer reads only existing snapshot fields; no new service data

## Testing

No declared test command (gate off); `cargo test` runs as evidence. The
only new Rust logic is `/api/reveal`'s path validation - unit-tested with a
temp dir (ok), a nonexistent path (rejected), and a file rather than dir
(rejected), as a pure `validate_reveal_path` helper. The drawer and lookup
are integration surface: browser evidence for the three port cases plus the
row-click path, and clipboard/reveal actions confirmed manually.

## Notes for the AI

- Reuse the stop context/dialog untouched; the drawer's Stop button just
  calls `requestStop(service)` - do not duplicate stop logic
- Clipboard uses `navigator.clipboard.writeText`; guard for absence and
  show a brief "copied" affordance
- Keep the row-click from stealing action-button clicks: the buttons live
  in their own container with `onClick` stopPropagation
- `servicesOnPort` returns all services whose `port` matches - a conflict
  port yields more than one
- `validate_reveal_path` is pure (takes a `&Path`, checks existence + is_dir
  via injected metadata in tests) so it unit-tests without touching the real
  file manager
- No `unwrap`/`expect` in the reveal handler; a failed `open` returns 500
  with the error text

## Completion notes

- Shipped as spec'd plus a step 4 added in review: framework-first display
  name. Rows/table/drawer now headline by framework ("Astro") when known,
  process name demoted to the sub-line; unlabeled processes still read node/
  python3. Answers Brad's long-standing naming complaint.
- Acceptance: 64 tests green (1 new, validate_reveal_path), clippy/fmt/
  oxlint/build clean; /api/reveal curl matrix 200/400/400; framework-first
  labels verified against live data; drawer + port lookup (one/conflict/
  empty cases) + row-click visually confirmed by Brad
- Playwright MCP backend was wedged all session; drawer/lookup visuals rest
  on Brad's manual confirmation, endpoint + naming automated-proven
- Reveal folder is an OS side effect via the `open` crate, path-validated
  (existing dir only); no config storage touched
- 13b (ignore service) still pending the config-storage decision; feature 13
  parent stays partially open
