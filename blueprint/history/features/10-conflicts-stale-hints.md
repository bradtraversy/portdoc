# Feature: Conflicts and stale hints

**From build-plan:** feature 10
**Status:** complete

## Goal

The snapshot starts carrying real `conflicts` and `stale` data, and the
Conflicts tab becomes a real view. Two honest, per-snapshot detections:
same-port multi-listener conflicts and default-port bump inference; plus an
age-based stale rule for known dev servers. The dashboard callouts, badges,
stat card, and tab count are already wired - they light up on their own.

## In scope

- **Conflict detection** (new flat module `src/hint.rs`), two provable
  classes:
  - *Same-port multi-owner*: two or more merged services on one port
    (SO_REUSEPORT, v4/v6 split ownership). Hint names the listeners:
    "2 processes are listening on :3000 (next-server, node)."
  - *Default-port bump inference*: a framework service running above its
    default port while another service holds the default. Defaults table:
    Next.js/Remix/Nuxt/React scripts 3000, Vite 5173, Astro 4321. Rules:
    the bumped service sits within default+1..=default+10, the default
    port is held by a different service, and - the false-positive guard -
    the bumped service's command does NOT mention its own port (an
    explicit `--port 4322` means intentional, not bumped). Hint: "Vite
    defaults to :5173, held by node (startdev); this instance moved to
    :5174." Multiple bumped instances of one default fold into a single
    conflict with all contender ids.
  - `Conflict.port` is the contested (default/shared) port;
    `service_ids` lists holder first, then bumped/co-listeners
- **Stale hints** (in the adapter, where raw `started_secs_ago` exists):
  a service whose framework is a known dev server (Next.js, Vite, Astro,
  Remix, Nuxt, React scripts) running >= 3 days gets
  `stale: { reason: "dev server running for {age}" }`. Databases, Docker,
  editors, and unlabeled processes never get flagged - honest restraint
  over noisy guessing
- **Conflicts tab UI** (`web/src/components/ConflictsView.tsx`): replaces
  the placeholder; one card per conflict - "Port :5173 conflict." headline,
  the hint, contender rows reusing `ServiceRow`; empty state "No port
  conflicts detected." when clean
- Tests: conflict matrix (same-port pair, bump with holder, explicit-port
  guard suppresses, multiple bumps fold, no false conflict on distinct
  ports), stale rule (dev server 3d+ flagged, 2d not, postgres 8d never,
  unlabeled never), hint copy shape

## Out of scope

- **Expected-but-missing apps** - requires persisted expectations, and
  local config storage is explicitly undecided until feature 13 (overview
  open question). Deferred with that dependency, not skipped silently
- **Conflict actions** (stop the holder, etc.) - actions are feature 12's
  safe-stop; the tab ships informational this pass
- Request-traffic observation ("no requests observed" claims) - we cannot
  see traffic; stale reasons only state what is provable (age)
- Cross-refresh state or history - detection stays per-snapshot

## Build steps

- [x] **Step 1 - conflict detection** - `src/hint.rs` with both conflict
  classes + defaults table + explicit-port guard; wired after grouping in
  `from_probe` (hints can name the holder's project). Unit tests.
  *Done when:* `cargo test` green.
- [x] **Step 2 - stale hints** - `is_dev_server` in `src/label.rs`, 3-day
  rule in `service_from`. Unit tests. *Done when:* `cargo test`,
  `cargo clippy`, `cargo fmt --check` clean and `--json` on this machine
  shows stale on the multi-day astro/next dev servers and nothing else.
- [x] **Step 3 - Conflicts tab** - `ConflictsView` replaces the
  placeholder in `App.tsx`; conflict cards with contender rows + empty
  state. *Done when:* `npm run lint` + `npm run build` pass and the tab
  renders a real conflict in the browser (manufactured: two listeners on
  one port), zero console errors.

## Files / areas

- `src/hint.rs` - new module: conflict detection + defaults table + tests
- `src/adapter.rs` - stale rule in `service_from`, conflicts wired in
  `from_probe`
- `src/label.rs` - `is_dev_server`
- `src/main.rs` - `mod hint;`
- `web/src/components/ConflictsView.tsx` - new; `App.tsx` - route the tab

## Data / contracts

- No shape changes: `Conflict { port, service_ids, hint }` and
  `StaleHint { reason }` are locked contract types, filled for the first
  time; `docker_hints` stays empty (feature 14)
- Hint strings are plain copy (per the prototype-polish decision: "Port
  3000 conflict." style, no clever phrasing); the UI prepends its own
  "Port {n} conflict." headline, so hints must not duplicate it
- Stale reason states only provable facts (age), never traffic claims

## Testing

No declared test command (gate off); `cargo test` runs as evidence anyway.
Detection is pure logic over `&[Service]` - the scope rule's territory.
The UI card is integration surface: browser evidence with a manufactured
same-port conflict (two loopback listeners on one port), plus the live
stale badges on this machine's multi-day dev servers.

## Notes for the AI

- Bump inference is deliberately conservative: framework known, within 10
  ports of the default, holder present, no explicit port in the command.
  When unsure, no conflict - false positives poison trust
- Holder-first ordering in `service_ids` matters: the UI's first contender
  reads as "who has it"
- The `staleUnconflicted` derive already dedupes stale-vs-conflict
  callouts; do not re-implement that in Rust
- Keep hint.rs pure: `&[Service]`, `&[ProjectGroup]` in, `Vec<Conflict>`
  out; no fs, no probing

## Completion notes

- Shipped as spec'd: same-port + bump-inference conflicts in src/hint.rs,
  3-day dev-server stale rule in the adapter, real Conflicts tab
- Expected-but-missing apps deferred to the feature 13 config decision
  (documented in Out of scope); Brad also expanded build-plan item 14 with
  well-known-port hints and desktop-app labels for the Ungrouped mystery
  rows during this feature's review
- Acceptance evidence: 58 tests green (9 new), clippy/fmt/oxlint/build
  clean; live stale flag on the 11d certificreate next-server only;
  manufactured v4+v6 same-port conflict rendered in the tab (contenders,
  red badges, pid-suffixed ids) and cleared to the empty state hands-free
  via polling when the listeners died; zero console errors
- Known inert affordances: the stale callout's Ignore / Stop safely buttons
  come alive at features 13 and 12 respectively
