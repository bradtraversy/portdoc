# Current Feature

> **Generated file.** Holds the one feature or fix being built right now. Run
> `/feature <number-or-name>` to spec a build-plan feature, or `/fix "<bug>"` for
> an ad-hoc fix. Build one thing at a time; `/complete` archives it (to
> `blueprint/history/features/` or `blueprint/history/fixes/`) and resets this file.

## Feature 14c - Advanced tab

**Branch**: `feature/advanced-tab`

### What

The Advanced tab: raw socket details, JSON export, and unknown-owner
diagnostics (well-known-port hints like ":22 - usually SSH" plus why an owner
is unreadable). Advanced data stays out of the main dashboard, per the
overview's UI section.

### Why

The dashboard deliberately hides raw networking facts; this is the escape
hatch for the cases where the calm view isn't enough - plus honest answers for
the "unknown" rows instead of leaving them mysterious.

### How

- **Raw sockets are not in the locked `DevSnapshot`** - they ship on a new
  `/api/sockets` endpoint (additive, same pattern as 13b's `/api/config`; the
  locked contract stays untouched). Response: probe backend name
  (`Probe::name()`, "linux-proc"), the pre-merge socket list (protocol, bind
  address, port, pid, process name, socket owner uid/user), and unknown-owner
  diagnostics.
- **Probe gains socket ownership**: `/proc/net/tcp{,6}` carries the socket
  uid; add `uid: Option<u32>` and `user: Option<String>` (resolved via the
  existing passwd lookup) to `ListeningSocket`. This is what lets diagnostics
  say "owned by root" for sockets whose pid is unresolvable. The
  `#[allow(dead_code)]` markers on `protocol` and `Probe::name` (parked "until
  the Advanced tab") come off now that they are consumed.
- **New `src/advanced.rs`**: a focused well-known-port table (ssh 22, DNS 53,
  SMTP 25/587, HTTP 80/443, rpcbind 111, IPP/printing 631, mDNS 5353, plus
  common dev infra: MySQL 3306, Postgres 5432, Redis 6379, MongoDB 27017) with
  `well_known(port)`, and `unknown_owner_reason(...)`: root-owned vs
  other-user vs no-uid phrasing, naming the user portdoc runs as. Diagnostics
  assemble one entry per unknown-owner port (deduped across v4/v6 binds).
- **`AdvancedView.tsx`** replaces the last placeholder, fetching
  `/api/sockets` on mount (loading and error states per coding standards):
  1. Unknown owners - port, well-known hint when there is one, and the
     unreadable-owner reason.
  2. Raw sockets - dense mono table: proto, bind address, port, pid, process,
     user; probe backend name in the section header.
  3. JSON export - download `/api/snapshot` as a timestamped `.json` file,
     copy-to-clipboard, and a note that `portdoc --json` prints the same
     snapshot.
- The now-unused `Placeholder` component and its record in `App.tsx` are
  deleted (no dead code).

### Out of scope

- Filters/search on the raw socket table (the Services tab owns search).
- UDP sockets (the probe is TCP-only; a "TCP only" note in the header is the
  honest label).
- Elevating privileges or reading other users' /proc - diagnostics explain
  the limit, they don't work around it.

### Build steps

- [x] 1. Probe: `uid`/`user` on `ListeningSocket` from the /proc entries,
  allow-markers removed from the now-consumed fields; unit tests (gate on).
- [x] 2. `src/advanced.rs`: well-known-port lookup, unknown-owner reason
  phrasing, diagnostics assembly with per-port dedupe; unit tests (gate on).
- [x] 3. `/api/sockets` endpoint: spawn_blocking probe, JSON shape (probe
  name, sockets, diagnostics), 500 JSON error on probe failure like
  `/api/snapshot`.
- [x] 4. `AdvancedView.tsx` with the three sections, loading/error states,
  JSON download/copy; placeholder machinery deleted. Frontend build green.

### Done when

- `/api/sockets` returns the probe name, every pre-merge listening socket with
  bind address and owner uid/user when readable, and one diagnostic per
  unknown-owner port with a well-known hint (when the port has one) and an
  honest reason.
- The Advanced tab renders the three sections against the live API; unknown
  root-owned ports on this machine (e.g. :22, :53, :631) show hints and
  reasons.
- JSON export downloads a valid snapshot file and copy puts the same JSON on
  the clipboard.
- `cargo test`, `cargo clippy`, `cargo fmt --check`, `npm run lint`, and
  `npm run build` all green.

### Testing

Rust gate is on. In scope: well-known-port lookup, unknown-owner reason
phrasing (root vs other user vs unknown uid), diagnostics assembly (only
unknown-owner sockets, deduped per port, well-known hint attached), and the
uid/user passthrough in the probe's socket mapping where unit-testable. The
endpoint handler is thin wiring; UI rides on build + browser evidence.
