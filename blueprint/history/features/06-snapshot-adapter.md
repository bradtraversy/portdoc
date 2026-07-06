# Feature: Snapshot adapter

**From build-plan:** feature 6
**Status:** complete

## Goal

Real probe output becomes the `DevSnapshot` served by `/api/snapshot` and
printed by `--json`. The mock module retires; the dashboard renders what is
actually listening on this machine. Unknown owners stay first-class, IDs are
predictable across refreshes, and every service gets a reachable localhost
URL. Grouping, framework labels, exposure classification, conflicts, and
stale hints stay stubbed (features 7-10).

## In scope

- New flat module `src/adapter.rs`: pure conversion from probe sockets to
  `DevSnapshot`
- Dual-stack merge: sockets sharing `(port, pid)` collapse into one service
  (the probe reports v4 and v6 raw; merging is this adapter's job)
- Predictable IDs: `svc-{port}-{slug}` where slug is the sanitized process
  name (lowercase, `[a-z0-9]` runs joined by `-`), `unknown` when there is
  no owner or name; same-snapshot collisions (SO_REUSEPORT) append `-{pid}`
- Field mapping: pid, process_name, command, cwd (absolute path string),
  user pass through; absent stays absent (omitted in JSON, never null)
- `started_age` humanized from `started_secs_ago`: `45s`, `4m`, `2h`, `6d`
- URL: `http://localhost:{port}` when any bind address is loopback or
  unspecified; otherwise the literal bind address (IPv6 bracketed). Every
  service gets a URL in this feature; feature 9 owns HTTP-looking
  classification and refines this
- Exposure: `local` when all of a service's bind addresses are loopback,
  otherwise `unknown` (feature 9 adds lan/docker)
- Wiring: `/api/snapshot` and `--json` build from `platform_probe()`;
  the probe runs in `spawn_blocking` per request (refresh-safe, re-probed
  every time). Probe error -> 500 with a JSON error body (API) / stderr +
  exit 1 (`--json`). No probe on this platform -> empty snapshot
- Retire `src/mock.rs` (delete) and drop the probe module's
  `#![allow(dead_code)]` now that it has a consumer

## Out of scope

- Project grouping (feature 7): `project_id` absent, `projects` empty
- Framework labels (feature 8): `framework` absent
- Real exposure classification and HTTP-looking URL filtering (feature 9)
- Conflicts and stale hints (feature 10): both empty, `stale` absent
- Docker hints (feature 14): empty
- Any UI change - the dashboard is already data-driven and renders empty
  sections gracefully
- Declaring the `test` command in AGENTS.md (gate switch stays with `/tests`)

## Build steps

- [x] **Step 1 - adapter core** - `src/adapter.rs` with the socket merge
  (group by port + pid), predictable ID generation with slug + collision
  rules, exposure rule, and direct field mapping into `Service`. Unit tests:
  v4+v6 merge, unknown-owner merge, slug edge cases (empty name, symbols),
  ID collision suffix, all-loopback vs mixed exposure. *Done when:*
  `cargo test` green.
- [x] **Step 2 - derived labels** - `started_age` humanizer and URL builder
  (localhost vs literal address, IPv6 brackets) attached to services. Unit
  tests: age boundaries (59s/60s, 59m/60m, 23h/24h), URL for loopback,
  wildcard, LAN-only v4, LAN-only v6. *Done when:* `cargo test` green.
- [x] **Step 3 - wiring** - `/api/snapshot` and `--json` serve the adapted
  real snapshot via `spawn_blocking`; probe error paths (500 JSON / exit 1);
  empty snapshot on unsupported platforms; delete `src/mock.rs`; remove the
  probe module's `#![allow(dead_code)]`. *Done when:* `cargo test`,
  `cargo clippy`, `cargo fmt --check` all clean and `portdoc --json` prints
  real services from this machine.

## Files / areas

- `src/adapter.rs` - new module + tests
- `src/main.rs` - handler and `--json` wiring, `mod` declarations
- `src/mock.rs` - deleted
- `src/probe/mod.rs` - remove the module-wide dead_code allow (a targeted
  allow may remain on `Probe::name` until feature 14 uses it)

## Data / contracts

- `DevSnapshot` shape is locked (feature 1); this feature only changes where
  the data comes from. Empty arrays serialize as `[]`; absent optionals are
  omitted
- `generated_at` epoch-milliseconds helper moves from `mock.rs` into the
  adapter
- Refresh-safe means: every request re-probes, concurrent requests are fine
  (blocking probe work off the async runtime), and a service that stays up
  keeps the same ID across refreshes by construction (port + name)
- The UI's protected-row check is `process_name === 'portdoc'`; the real
  probe reports comm `portdoc`, so the served binary keeps its badge

## Testing

No declared test command (gate off); `cargo test` runs as evidence anyway.
The adapter is exactly the kind of pure logic the scope rule wants tested:
merge, ID, slug, age, and URL helpers take plain inputs and get unit tests
with real edge cases. The end-to-end proof is `--json` output and the
dashboard rendering this machine's real listeners.

## Notes for the AI

- No `unwrap`/`expect` in adapter or handler code; degrade or return errors
- Keep the adapter pure: `ProbeOutput` in, `DevSnapshot` out; `main.rs` owns
  the async/spawn_blocking edge
- Mock IDs and consts disappear with `mock.rs`; nothing else references them
- Sort services by port (the probe already sorts; keep the guarantee after
  merging)

## Completion notes

- Shipped as spec'd; one deviation: `src/snapshot.rs` gained targeted
  `#[allow(dead_code)]` on `Exposure::Lan`/`Docker` (locked contract values
  whose only constructor was the deleted mock; features 9/14 construct them)
- Acceptance evidence: 18 tests green, clippy/fmt clean; 29-30 real services
  on trav-dev via `--json` and `/api/snapshot`; ID sets identical across
  consecutive probes; bare `/api` still 404; dashboard browser-verified with
  zero console errors, portdoc's own row keeps the protected badge
- Real-data findings for later features: kernel comm truncates to 15 chars
  ("next-server (v1") and node reports "MainThread" - feature 8 derives
  friendly labels from the full command (consider /proc/pid/exe basename in
  the probe); root-owned listeners surface as unknown-owner as designed
