# Current Feature

> **Generated file.** Holds the one feature or fix being built right now. Run
> `/feature <number-or-name>` to spec a build-plan feature, or `/fix "<bug>"` for
> an ad-hoc fix. Build one thing at a time; `/complete` archives it (to
> `blueprint/history/features/` or `blueprint/history/fixes/`) and resets this file.

## Feature 14b - Docker tab

**Branch**: `feature/docker-tab`

### What

Docker and Compose hints tied to services and ports, plus the Docker tab view
replacing its placeholder. Mechanism decided 2026-07-08: shell out to the docker
CLI (`docker ps --format {{json .}}`), no socket client. Missing or unreachable
Docker always degrades to empty hints, never an error - the snapshot API must
stay healthy on machines without Docker.

### Why

The `docker_hints` field has been an empty contract-locked array since feature 1.
Docker-exposed services currently show as bare `docker-proxy` rows; this maps
them back to the container, image, and Compose project a developer actually
recognizes.

### How

- New `src/docker.rs`: run `docker ps --format {{json .}}` with a hard timeout
  (spawn + `try_wait` polling, kill on expiry) so a hung daemon can never stall
  `/api/snapshot`. Pure parsers turn stdout lines into containers: JSON line ->
  name (first of `Names`), image, compose project (`com.docker.compose.project=`
  label), published host TCP ports parsed from the `Ports` string
  (`0.0.0.0:5432->5432/tcp, [::]:5432->5432/tcp` -> 5432, deduped; unpublished
  and udp entries skipped). Malformed lines are skipped, never fatal.
- Hint mapping: one `DockerHint { port, container, service_id?, image?,
  compose_project? }` per published port. `service_id` joins by port against the
  snapshot's services, preferring a `docker-proxy`-owned listener when several
  share the port.
- `adapter.rs` fills `docker_hints` from the docker module inside the existing
  `spawn_blocking` probe path.
- New `web/src/components/DockerView.tsx` replaces the docker placeholder:
  containers grouped by Compose project (ungrouped last), each row shows
  container, image, and its published ports; clicking a port opens the existing
  inspect drawer for that port. Honest empty state: no containers found, and
  Docker itself may be absent or stopped (the snapshot cannot tell these apart,
  so the copy says so).
- `InspectDrawer` gains Container / Image / Compose fields when a hint matches
  the service (by id, falling back to port).
- Contract untouched: `DockerHint` shape already locked at feature 1.

### Out of scope

- Container lifecycle actions (start/stop/restart containers) - stop stays
  process-based (feature 12).
- `docker inspect` enrichment, container stats, logs.
- A snapshot-level "docker status" field (contract stays as-is).
- Advanced tab work (14c).

### Build steps

- [x] 1. `src/docker.rs` pure parsing: container JSON-line parsing, `Ports`
  string parsing to deduped host TCP ports, compose-project label extraction,
  with unit tests (gate on).
- [x] 2. Wire into the snapshot: timeout-guarded `docker ps` runner, hint
  mapping with the service-id join (docker-proxy preferred), `adapter.rs`
  fills `docker_hints`; unit tests for the join and mapping.
- [x] 3. Docker tab UI: `DockerView.tsx` grouped by Compose project with
  port -> inspect-drawer links and the honest empty state; remove the docker
  placeholder entry. Frontend build green.
- [x] 4. Inspect drawer docker fields (Container / Image / Compose) for
  services with a matching hint. Frontend build green.

### Done when

- `portdoc --json` on a machine without Docker prints a healthy snapshot with
  `docker_hints: []` and no error output from the docker path.
- With Docker running containers that publish TCP ports, `docker_hints` carries
  one entry per published port with container, image, compose project when
  present, and a `service_id` joined to the matching listener (unit-tested;
  this machine has no Docker, so live evidence is the graceful path plus a
  stubbed-snapshot UI check).
- The Docker tab renders grouped containers with clickable ports, and shows the
  empty state instead of a placeholder when there are no hints.
- The inspect drawer shows Container / Image / Compose for a docker-hinted
  service.
- `cargo test`, `cargo clippy`, `cargo fmt --check`, and `npm run build` are all
  green.

### Testing

Rust gate is on. In scope: `Ports` string parsing (published v4/v6 dedupe,
unpublished skip, udp skip, ranges/garbage tolerated), JSON line parsing
(malformed lines skipped, first name wins, missing labels), compose label
extraction, and hint mapping (per-port hints, docker-proxy-preferred join,
no-match -> `service_id` absent). The CLI runner itself (process spawn, timeout)
and all UI stay on build + screenshot evidence.
