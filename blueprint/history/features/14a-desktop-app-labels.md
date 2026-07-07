# Current Feature

> **Generated file.** Holds the one feature or fix being built right now. Run
> `/feature <number-or-name>` to spec a build-plan feature, or `/fix "<bug>"` for
> an ad-hoc fix. Build one thing at a time; `/complete` archives it (to
> `blueprint/history/features/` or `blueprint/history/fixes/`) and resets this file.

# Feature: Desktop app labels

**From build-plan:** feature 14a
**Status:** not started

## Goal

Editor and desktop-app internals stop reading as mystery rows: the VS Code
language servers, Discord's renderer, and browser helpers that hold local ports
get named ("VS Code", "Discord") instead of showing raw process names like
`code` and `exe`. Extends the feature 8 label vocabulary; no new UI.

## In scope

- A desktop-app vocabulary in `src/label.rs`, detected with the same
  token-basename mechanism `detect_framework` uses (never raw substrings):
  VS Code (`code`, `code-insiders`), Discord (`discord`), Chrome (`chrome`),
  Chromium (`chromium`, `chromium-browser`), Firefox (`firefox`), Brave
  (`brave`, `brave-browser`), Slack (`slack`), Spotify (`spotify`), and a
  generic Electron (`electron`).
- Checked after frameworks: a real framework match always wins.
- The label flows through the existing `framework` field, so `displayName`,
  rows, table, drawer, and project-group badges pick it up with zero frontend
  changes. (The contract-locked field name stays `framework`; it has been the
  general "detected label" since feature 1.)
- Path-derived tokens work for flag values: Discord's
  `--user-data-dir=/home/brad/.config/discord` yields the token `discord` via
  the existing basename cleaning.

## Out of scope

- Any UI change - no new filter chip, no "App" category; existing chips are
  unaffected (the `Framework` chip's explicit set does not include these).
- Well-known-port hints and why-unreadable diagnostics (14c).
- Docker hints and the Docker tab (14b).
- Stale accusations for desktop apps - `is_dev_server` keeps its explicit
  dev-server list, so labeling Discord never makes it stale-accusable.
- `http_looking` changes - desktop-app helper ports already get URLs today;
  tightening that is a separate decision.

## Build loop

Build one step at a time, never the whole feature at once. Autopilot runs the
steps without pausing; review happens at the final packet.

## Build steps

- [x] **Step 1 - detection vocabulary.** Add `DESKTOP_APPS` to `src/label.rs`
  and check it in `detect_framework` after the framework table; update the
  module doc. Tests use real command shapes from this machine: the snap
  VS Code binary + extension host, Discord's `exe` process with
  `--user-data-dir=.../.config/discord`, plus chrome/firefox/slack shapes,
  and a guard that framework matches still beat app matches
  (`node .bin/vite` inside anything stays Vite) and that basename matching
  holds (`/opt/discord-clone/server.js` is not Discord).
  *Done when:* `cargo test` green with the new cases; `cargo clippy` clean.
- [x] **Step 2 - live check.** Run the app against this machine's real
  processes: the pylance/VS Code helpers show "VS Code" (raw `code` demoted to
  the sub-line) and the Discord renderer shows "Discord" instead of `exe`.
  *Done when:* browser evidence shows the relabeled rows; `npm run build`
  untouched or still green.

## Files / areas

- `src/label.rs` - vocabulary, detection order, tests

## Data / contracts

- None new. Labels flow through the existing `Service.framework` field
  (contract-locked name, defined as the detected label since feature 1).

## Testing

- `cargo test` gate applies to step 1 (pure detection logic, real command
  shapes as cases).
- Step 2 is visual: browser evidence on the live snapshot.

## Notes for the AI

- Match command-token basenames only, per the existing `clean_token` mechanism;
  raw substring matching is exactly what feature 8's tests forbid.
- Framework beats app; apps are the fallback vocabulary.
- Keep the vocabulary small and provable - only apps with unambiguous
  process/path tokens.
