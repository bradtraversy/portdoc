# Current Feature

> **Generated file.** Holds the one feature or fix being built right now. Run
> `/feature <number-or-name>` to spec a build-plan feature, or `/fix "<bug>"` for
> an ad-hoc fix. Build one thing at a time; `/complete` archives it (to
> `blueprint/history/features/` or `blueprint/history/fixes/`) and resets this file.

# Feature: Ignore service

**From build-plan:** feature 13b
**Status:** not started

## Goal

Let a service be hidden from the dashboard and remembered across restarts: the
first persisted local config. Forgotten-but-fine listeners (a printer daemon, a
long-running tool) stop cluttering the everyday view, and the ignore action the
UI has promised since the stale callout shipped ("lands with feature 13")
finally works.

## In scope

- `src/config.rs`: load/save `config.json` from the platform config dir via the
  `dirs` crate (`~/.config/portdoc/config.json` on Linux). Missing file means
  default (empty) config; malformed JSON is treated as default rather than a
  crash; saves create the directory and write atomically (temp file + rename).
- Config shape (load-bearing, additive later): `{ "ignored_services": ["svc-3000-node"] }`.
- API: `GET /api/config` returns the config; `POST /api/ignore`
  `{ "service_id": "...", "ignored": true|false }` mutates the list and returns
  the updated config. Mutations serialize behind a `tokio::sync::Mutex` so
  concurrent read-modify-writes cannot drop each other.
- Frontend: fetch config alongside the snapshot; ignored services disappear
  from the dashboard (project groups, callouts, stat counts) and from the
  default Services table.
- A "N ignored - show" toggle on the Services tab reveals ignored rows with a
  neutral "ignored" badge.
- Actions: "Ignore" enabled in the stale callout; "Ignore" / "Unignore" button
  in the inspect drawer. Failures surface briefly on the button (the
  `RevealButton` error pattern), never silently.
- Port lookup still finds an ignored service (an explicit port question gets an
  honest answer), shown with the "ignored" badge.

## Out of scope

- Any other config keys (UI preferences, trusted project roots) - the shape
  allows them later, nothing writes them now.
- Ignore-by-port or ignore-by-project.
- Filtering ignored services out of `/api/snapshot` or `portdoc --json` - the
  snapshot stays the unfiltered truth; hiding is a UI concern.
- A dedicated "Ignored" filter chip (the show-ignored toggle covers it).
- Garbage-collecting ignored ids whose service never reappears - the config
  just keeps them; a future config feature can prune.
- Docker-tab behavior (feature 14).

## Build loop

Build one step at a time, never the whole feature at once.

1. Plan mode lays out the step before any code.
2. The AI implements just that step.
3. It shows the diff (not full files); you read it and understand it.
4. You approve, then choose whether to commit a checkpoint or roll straight on.

Never accept a step you haven't read. If a diff is too big to review, the step
was too big, so split it.

## Build steps

- [x] **Step 1 - config module.** Add the `dirs` dependency and `src/config.rs`:
  `Config { ignored_services: Vec<String> }` with serde defaults, `config_path()`,
  `load(path)` (missing or malformed -> default), `save(path)` (create dir,
  temp-file + rename), and a `set_ignored(id, bool)` mutation that dedupes.
  *Done when:* `cargo test` covers load-missing, load-malformed, round-trip,
  and dedupe/unignore mutation, all green; `cargo clippy` clean.
- [x] **Step 2 - API endpoints.** `GET /api/config` and `POST /api/ignore` in
  `main.rs`, mutations behind a `tokio::sync::Mutex`. Bad request bodies get a
  400, save failures a 500 with a message; handlers never panic.
  *Done when:* `curl /api/config` shows the empty default, a `POST /api/ignore`
  round-trips into the file on disk (visible in `cat` and a fresh GET), and
  unignoring removes it.
- [x] **Step 3 - hide ignored services.** `useConfig` hook (fetch + mutate +
  refetch), an `ignored` set threaded through `derive.ts`; dashboard groups,
  callouts, stat counts, and the default Services table exclude ignored
  services; the Services tab gains the "N ignored - show" toggle and the
  neutral badge; port lookup still surfaces ignored listeners with the badge.
  *Done when:* `npm run build` and `npm run lint` pass; with one service
  ignored (via curl), the dashboard and table hide it, the toggle reveals it
  with the badge, and lookup on its port finds it.
- [x] **Step 4 - ignore/unignore actions.** Enable the stale-callout "Ignore"
  button; add "Ignore"/"Unignore" to the inspect drawer actions; both call
  `POST /api/ignore` through the hook and re-render without a manual refresh;
  errors flash on the button.
  *Done when:* in the browser, ignoring from the drawer hides the row
  immediately, "show ignored" + Unignore restores it, and the stale callout's
  Ignore works end to end.

## Files / areas

- `Cargo.toml` - add `dirs`
- `src/config.rs` (new), `src/main.rs` (routes, shared state)
- `web/src/lib/config.ts` or `useConfig` hook (new), `web/src/lib/derive.ts`
- `web/src/components/`: `Callouts.tsx`, `InspectDrawer.tsx`, `ServicesTable.tsx`,
  `StatCards.tsx`, `ProjectGroups.tsx`, `PortLookup.tsx` (whichever of these
  consume the visible-services derivation)

## Data / contracts

- **Load-bearing config file shape**: `{ "ignored_services": string[] }` -
  future keys are additive; unknown keys in the file must not break loading
  (serde default + allow unknown fields).
- **API**: `GET /api/config` -> the config JSON; `POST /api/ignore`
  `{ service_id: string, ignored: boolean }` -> the updated config JSON.
- The `DevSnapshot` contract is untouched - no new fields, no filtering.

## Testing

- `cargo test` gate applies to steps 1-2 (config load/save/mutation logic and
  any extracted handler mutation function). Inject the path; never touch the
  real `~/.config` in tests (tempdir).
- Steps 3-4 are UI: browser evidence plus `npm run build` / `npm run lint`.
- Manual verify: ignore a real service, restart the binary, confirm it stays
  hidden (persistence proven), then unignore.

## Notes for the AI

- Service IDs are the stable handle (feature 6 made them predictable across
  refreshes); ignoring stores ids, nothing else.
- Keep `unwrap`/`expect` out of runtime paths; config errors return defaults or
  proper HTTP status codes per `coding-standards.md`.
- The snapshot polling loop already exists (`useSnapshot`); config fetching
  should follow its patterns, not invent a new data layer.
- Ignoring the portdoc row itself is allowed - it only hides a row; the safe-stop
  guard is a separate concern and stays untouched.
