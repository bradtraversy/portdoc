# Feature: Mock snapshot contract

**From build-plan:** feature 1
**Status:** complete

## Goal

Lock the shared `DevSnapshot` JSON contract and expose it three ways from one
code path: `/api/health`, `/api/snapshot` with mocked services, and
`portdoc --json` printing the same snapshot to stdout. Every later feature (UI,
probes, adapters) consumes this shape, so it changes here or not at all.

## In scope

- `serde` + `serde_json` dependencies (planned for this feature in the overview)
- `DevSnapshot`, `Service`, `ProjectGroup`, `Conflict`, `DockerHint` types in a
  new `snapshot` module, with the serialization rules below
- A `mock` module producing one realistic mocked snapshot (single source of
  truth used by both the API and `--json`)
- `GET /api/health` returning status + version
- `GET /api/snapshot` returning the mocked `DevSnapshot` as JSON
- `--json` CLI flag: print the same snapshot (pretty JSON) to stdout and exit
  without starting the server

## Out of scope

- Real probing (features 4-6), project detection (7), labels (8-9)
- Serving the web UI or opening a browser (feature 2)
- Any `web/` changes; TS types for the contract land with the UI (feature 3)
- `portdoc ui` subcommand - CLI surface decision deferred to feature 2
- Conflict/stale computation - mock data hardcodes examples only

## Build steps

- [x] **Step 1 - contract types** - add `serde`/`serde_json` to `Cargo.toml`
  and create `src/snapshot.rs` with the full type tree and serialization rules.
  *Done when:* `cargo check` passes with the new module compiled in.
- [x] **Step 2 - mock snapshot** - create `src/mock.rs` building a
  representative `DevSnapshot`: 3 projects, 8 services covering every exposure
  variant, an unknown-owner service, 2 stale services, 2 conflicts (each with
  two contenders), 2 docker hints. *Done when:* `cargo check` passes and the
  mock exercises every field of the contract at least once.
- [x] **Step 3 - API routes** - wire `GET /api/health` and `GET /api/snapshot`
  into the axum router. *Done when:* with the server running,
  `curl 127.0.0.1:7788/api/health` returns `{"status":"ok","version":...}` and
  `curl 127.0.0.1:7788/api/snapshot` returns the full mock snapshot with
  `content-type: application/json`.
- [x] **Step 4 - `--json` flag** - add `--json` to the clap CLI; it prints the
  pretty-printed snapshot to stdout and exits before binding the listener.
  Serialization failure exits 1 with a message on stderr (no panic). *Done
  when:* `portdoc --json` parses with `jq` and, ignoring `generated_at`, equals
  the `/api/snapshot` body.

## Files / areas

- `Cargo.toml` - add `serde` (derive) and `serde_json`
- `src/snapshot.rs` - new: contract types (serialize-only)
- `src/mock.rs` - new: mock snapshot builder
- `src/main.rs` - routes, `--json` flag

## Data / contracts

Locked JSON decisions (the contract this feature exists to pin down):

- `DevSnapshot`: `generated_at`, `services[]`, `projects[]`, `conflicts[]`,
  `docker_hints[]` - arrays always present, empty allowed
- `generated_at`: unix epoch **milliseconds** (number). No chrono dependency;
  the JS client does `new Date(ms)` directly
- Optional fields are **omitted** when absent (`skip_serializing_if`), never
  `null`
- `Service`: `id`, `port`, then optional `pid`, `process_name`, `command`,
  `cwd`, `user`, `project_id`, `framework`, `url`, `started_age`, `stale`;
  `exposure` required
- `exposure`: string enum `"local" | "lan" | "docker" | "unknown"` (lowercase)
- `stale`: optional object `{ "reason": string }` - present means stale, absent
  means not stale
- `started_age`: human string (`"2h"`, `"6d"`), matching the prototype display
- `ProjectGroup`: `id`, `name`, `root`, optional `package_manager`,
  `git_branch`, and `service_ids[]`
- `Conflict`: `port`, `service_ids[]`, `hint`
- `DockerHint`: `port`, `container`, optional `service_id`, `image`,
  `compose_project`
- `/api/health`: `{ "status": "ok", "version": "<CARGO_PKG_VERSION>" }`
- Service `id`s are stable strings in the mock (e.g. `"svc-3001-next"`);
  the predictable-ID scheme for real data is feature 6's job

## Testing

No test command is declared in `AGENTS.md`, so the test gate is off. Evidence
per step: `cargo check` (steps 1-2), running server + `curl` output (step 3),
`portdoc --json | jq` plus a `jq 'del(.generated_at)'` diff against the API
body (step 4). `cargo clippy` and `cargo fmt` clean before the feature is done.

## Notes for the AI

- Mock data should echo the prototype's dummy world (startdev, portdoc,
  react-crash-2026, Docker postgres/redis) so feature 3 can drive the mock
  dashboard straight from this payload
- Keep the `/` placeholder route as is; feature 2 replaces it with the UI
- `unwrap`/`expect` only at startup paths; the `--json` serialization error
  path must not panic
- Handlers return data infallibly (mock is static); no error middleware yet
