# Feature: URL and exposure labels

**From build-plan:** feature 9
**Status:** complete

## Goal

Exposure becomes a real classification instead of feature 6's placeholder
(loopback -> local, everything else -> unknown), and URLs stop being
generated for services that clearly are not HTTP (`http://localhost:22` and
database links disappear). The LAN VISIBLE stat card and lan/docker badges
start telling the truth.

## In scope

- **Exposure classification** in the adapter, per merged service:
  - owning process is `docker-proxy` -> `docker` (the userland proxy that
    publishes container ports; provable ownership only - rootful Docker
    with unreadable fd tables stays bind-based)
  - all bind addresses loopback -> `local`
  - any bind address non-loopback (wildcard or a specific interface
    address) -> `lan` - the bind address answers reachability regardless
    of whether the owner is known, so unknown-owner wildcard listeners
    (sshd as non-root observer) honestly show as LAN visible
  - `unknown` remains in the vocabulary as the empty-information fallback
    (no addresses); real Linux snapshots may never produce it
- **HTTP-looking URL rule**: keep feature 6's URL generation (localhost
  when loopback/wildcard reachable, literal address otherwise) but only
  for services that look like HTTP. Deny-list refinement, any signal kills
  the URL:
  - framework label in {Postgres, Redis} (catches Brad's embedded
    postgres on :54329 where the port list can't)
  - well-known non-HTTP ports: 22, 25, 53, 110, 143, 465, 587, 993, 995,
    1433, 2049, 3306, 5432, 5672, 6379, 9092, 11211, 27017
  - process name in {sshd, dnsmasq, systemd-resolved} (non-framework
    daemons that roam ports)
  - everything else keeps its URL - an unknown wildcard :8080 is more
    likely a dev server than not
- Tests: exposure matrix (loopback-only, wildcard, specific LAN address,
  mixed loopback+wildcard merged service, docker-proxy owner beats lan,
  unknown-owner wildcard is lan), URL rule matrix (each deny signal, dev
  ports keep URLs, framework beats odd port, denied service still gets
  exposure)

## Out of scope

- Docker container identification, names, images, compose (feature 14's
  DockerHint - exposure `docker` here is only the provable proxy case)
- Active probing (sending HTTP requests to classify) - per-request
  snapshots must stay fast and passive
- LAN-shareable URLs (the machine's LAN IP for wildcard binds) - the
  localhost URL always works in Brad's browser; share/copy actions are
  feature 13
- Firewall awareness - `lan` means the bind address is LAN-reachable, not
  that a firewall permits it
- UI changes - lan/docker badges, the Docker group section, and the LAN
  stat card are already wired to these fields

## Build steps

- [x] **Step 1 - exposure classification** - docker-proxy rule + lan rule
  replace the feature 6 placeholder in `src/adapter.rs`; exposure test
  matrix. *Done when:* `cargo test` green.
- [x] **Step 2 - HTTP-looking URLs** - deny-list rule gates URL
  generation; URL test matrix. *Done when:* `cargo test`, `cargo clippy`,
  `cargo fmt --check` clean, and `portdoc --json` on this machine shows
  no URL on :22/:53/:631-class rows or postgres/redis rows, URLs intact
  on dev servers, and real lan exposure on wildcard binds.

## Files / areas

- `src/adapter.rs` - `exposure()` replacement, URL gating, tests
- `src/label.rs` - only if the deny lists read better there (they are
  label vocabulary); no other module changes expected

## Data / contracts

- No shape changes: `exposure` already carries all four values;
  `url` is already optional and omitted when absent
- `Exposure::Lan` and `Exposure::Docker` get constructed for the first
  time - remove their `#[allow(dead_code)]` in `src/snapshot.rs`
  (`Exposure::Unknown` stays constructible as the fallback)
- Deny lists are vocabulary tables like the framework table - one-line
  additions later, no contract impact

## Testing

No declared test command (gate off); `cargo test` runs as evidence anyway.
Classification and URL gating are pure decision logic over plain inputs -
exactly the scope rule's territory. End-to-end proof is `--json` plus the
dashboard: LAN stat card counts wildcard binds, ssh/database rows lose
their links, dev servers keep them.

## Notes for the AI

- `631` (CUPS) serves a real web UI; it is deliberately NOT on the port
  deny list - only flag things that are clearly not HTTP
- Exposure and URL are independent decisions: a denied-URL service still
  classifies (postgres -> local/lan), and a lan service still gets a
  localhost URL when it is HTTP-looking
- Keep the existing merged-addresses semantics: classification looks at
  ALL of a service's bind addresses, not the first
- The docker-proxy check matches the owning process name exactly
  ("docker-proxy"), not a substring of the command

## Completion notes

- Shipped as spec'd: real lan/docker exposure classification and
  HTTP-looking URL gating via deny tables in src/label.rs
- Acceptance evidence: 49 tests green, clippy/fmt clean; real machine shows
  7 lan / 20 local, URLs dropped on :22/:53/:6379 (port signal) and :54329
  (Postgres framework on an odd port), kept on :631 (CUPS) and dev servers;
  LAN VISIBLE stat card and badges browser-verified, zero console errors
- Docker exposure covered by unit test only (no docker-proxy running on
  trav-dev during acceptance); live visual arrives with feature 14
- Two feature 6 test expectations legitimately flipped (unknown -> lan) and
  were renamed to describe the new rule
