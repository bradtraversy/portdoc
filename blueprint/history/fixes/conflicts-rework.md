# Current Feature

> **Generated file.** Holds the one feature or fix being built right now. Run
> `/feature <number-or-name>` to spec a build-plan feature, or `/fix "<bug>"` for
> an ad-hoc fix. Build one thing at a time; `/complete` archives it (to
> `blueprint/history/features/` or `blueprint/history/fixes/`) and resets this file.

## Conflicts rework: drop bump inference, remove the Conflicts tab

**Type:** Fix

### The problem

Feature 10's conflict detection over-claims, and the "conflict" concept itself
doesn't survive scrutiny:

1. **Bump inference is guesswork presented as fact.** `bumped_port_conflicts` in
   `src/hint.rs` accused a 12-day-old Next.js server of bumping a 3-hour-old
   Vite - causally impossible from one snapshot. The conflict developers
   actually hit (EADDRINUSE) is invisible to a probe: the loser dies without
   ever listening, so causation inferred from port adjacency will cry wolf.
2. **The observable remainder is not a conflict.** After the adapter merges an
   app's own v4/v6 listeners, "two processes on one port" means SO_REUSEPORT
   worker pools and similar - deliberate design, not a problem. Styled today as
   a red alarm (danger triangle, red stat card, dashboard callout).
3. A Conflicts tab fed only by that remainder would sit empty almost always. It
   is a footnote, not a top-level surface.

The real EADDRINUSE workflow ("hit the error -> open portdoc -> look up the
port") is already served by the 13a port lookup and inspect drawer.

### The fix

Decision (Brad, 2026-07-07, superseding the earlier "quiet tab" call):
**remove the Conflicts tab; keep shared-port facts as quiet metadata.**

- Backend: delete bump inference entirely - `bumped_port_conflicts` and its
  helpers (`DEFAULT_PORTS`, `BUMP_RANGE`, `default_port`,
  `command_mentions_own_port`, `holder_label`) plus their tests. Shared-port
  detection stays and keeps feeding the contract-locked `DevSnapshot.conflicts`
  field (feature 1 locks the name; contents become shared-port groups only).
- Frontend: remove the Conflicts tab, the dashboard conflict callout, and the
  "Port conflicts" stat card. Shared-port facts survive as a neutral
  "shared port" badge on service rows and in the inspect drawer, plus the
  filter chip (relabeled "Shared port").
- Deeper socket detail belongs to feature 14's Advanced tab, not this fix.
- Plan/overview mentions of the Conflicts tab get synced in the already-planned
  `/overview` re-run (config-storage decision), not here.

Must not break: port lookup / inspect drawer behavior, filter chip mechanics
(id `conflict` stays), stale hints and callouts, safe stop.

### Build steps

- [x] **Step 1 - backend: shared-port only.** In `src/hint.rs`, delete
  `bumped_port_conflicts` and helpers; `detect_conflicts` returns only
  `shared_port_conflicts` output. Keep the hint copy factual ("N processes are
  listening on :port (...)"), update the module doc comment, delete
  bump-specific tests, keep shared-port tests green, add a regression test:
  the old false-positive shape (framework service on default+1 while another
  service holds the default) returns no conflict.
  Done when: `cargo test` passes with no bump tests remaining and the
  regression test proves the false positive is gone.
- [x] **Step 2 - frontend: remove the tab, keep the badge.**
  - `TabBar.tsx`: drop the `conflicts` tab and its count badge; `TabId` loses
    `'conflicts'`.
  - `App.tsx`: remove `ConflictsView` routing; delete `ConflictsView.tsx`.
  - `Callouts.tsx`: remove the per-conflict dashboard alarm; stale callouts
    stay. Simplify `staleUnconflicted` in `derive.ts` to plain stale services.
  - `StatCards.tsx`: remove the "Port conflicts" card (three cards remain).
  - `ServiceRow.tsx` / `ServicesTable.tsx` / `InspectDrawer.tsx`: badge text
    "shared port" with neutral styling instead of danger "conflict".
  - `filter.ts`: chip label "Shared port" (id stays `conflict`).
  Done when: `npm run build` and `npm run lint` pass, no Conflicts tab or red
  conflict styling remains anywhere, and a service sharing a port still shows
  the neutral badge in the table and drawer.

### Verify

- `cargo test` green (step 1 carries the logic; the test gate applies).
- Run the app next to a Vite server on 5174 with something on 3000/5173: no
  bump conflict appears anywhere.
- Dashboard has no conflict callout and no conflicts card; tab bar has five
  tabs; the "Shared port" filter chip and row/drawer badges still work for a
  genuinely shared port.
