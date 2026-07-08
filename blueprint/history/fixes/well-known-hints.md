# Current Feature

> **Generated file.** Holds the one feature or fix being built right now. Run
> `/feature <number-or-name>` to spec a build-plan feature, or `/fix "<bug>"` for
> an ad-hoc fix. Build one thing at a time; `/complete` archives it (to
> `blueprint/history/features/` or `blueprint/history/fixes/`) and resets this file.

## Fix: Surface well-known-port hints on unknown rows

**Type:** Fix
**Branch**: `fix/well-known-hints`

### The problem

Unknown-owner rows (root-owned :22, :53, :631, :6379) read as bare "unknown"
in the Services table, project groups, and the inspect drawer, even though 14c
already ships "usually SSH"-style hints - stranded in the Advanced tab. The
hint should meet the user where they look, without ever claiming identity
(PortDoc cannot see the process; "SSH" as a name would be a guess dressed as
a fact).

### The fix

Frontend-only mirror of the 14c well-known-port table (no contract change;
same mirroring pattern `filter.ts` already uses for labels, with the same
"mirrors src/advanced.rs" comment):

- `derive.ts`: `wellKnownHint(service)` returns "usually SSH" etc. only when
  the service has no framework and no process name - a known identity always
  beats folklore.
- `ServiceRow` (dashboard/projects rows): unknown rows get the hint as their
  sub-line (currently empty for them).
- `ServicesTable` service cell: hint rendered muted next to "unknown".
- `InspectDrawer`: a "Usually" field, plus a short note that the owner is
  unreadable and the Advanced tab has the specific reason (the honest
  root-vs-other-user phrasing needs the socket uid, which stays off the
  locked `Service` shape).

Must not break: rows with real identities (hint must never appear beside a
framework or process name), and the display name stays "unknown" - the hint
is presentation, not identity.

### Build steps

- [x] 1. `wellKnownHint` in `derive.ts` + the three render sites. Done when
  :22/:53/:631/:6379 show "usually ..." hints in table, rows, and drawer,
  known rows are unchanged, and `npm run lint`/`npm run build` are green.

### Verify

Browser on the live app: Services tab shows "unknown - usually SSH" on :22;
the drawer for :22 shows the Usually field and the Advanced pointer; a labeled
row (e.g. Paperclip :3100) shows no hint. UI-only change - build plus browser
evidence, no Rust touched.
