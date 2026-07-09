# Feature: Project actions

**From build-plan:** feature 16c
**Status:** complete 2026-07-09. Built by autopilot in 4 steps, all gates green (98 Rust tests, clippy, oxlint, build). Live-verified in the browser: open-in-editor, copy-cd clipboard, and the full stop-all contract (graceful, escalate, force, self-refusal inline). Includes two bug fixes found during verification: tilde-path expansion for /api/open and /api/reveal, and the stop-all dialog lifted to App level so it survives its trigger unmounting.

## Goal

Act on a whole project instead of one service at a time: open the project root
in an editor, copy a `cd` command, and stop every service the project owns -
the "I'm done with this project" button - on the feature 12 safety contract.

## Decisions

- **Editor command (decided 2026-07-09, Brad):** a new `editor` key in the 13b
  config file, defaulting to `"code"`. The value is split on whitespace
  (program + args) and the project root is appended, so `"code"`, `"cursor"`,
  or `"code -n"` all work by editing one config line.
- **Stop-all is frontend orchestration.** No new batch endpoint: the dialog
  loops the existing `/api/stop` per service, so every feature 12 guard
  (self-refusal, graceful-then-verify, force behind second confirm) applies
  per service unchanged.

## In scope

- `POST /api/open` - validate the path is an existing directory, spawn the
  configured editor detached (reaped, no zombies).
- Config `editor` key with serde default `"code"`; old config files load
  unchanged.
- Project action buttons in two places: the Projects tab group headers and the
  project drawer - Open in editor, Copy cd, Stop all.
- `StopAllDialog`: lists every stoppable service (port, PID, name, command)
  before anything is signaled; graceful stop for all; per-service outcomes;
  force kill only for survivors behind a second explicit confirmation.
  Services without a PID are listed as not stoppable, and per-service errors
  (e.g. the self-stop refusal, other-user processes) surface inline.

## Out of scope

- Running project scripts (dev/build) from the UI - display-only since 16b.
- Editor picking UI; config is edited by hand (a settings screen is post-v1).
- Batch stop endpoint on the backend.
- Windows probe work (15b) and release (15c).

## Build steps

- [x] **Step 1 - backend: editor config + /api/open** - `editor` key on
  `Config` (serde default `"code"`, unknown keys still tolerated);
  `editor_command()` parser (whitespace split, root appended) unit-tested,
  including empty/whitespace-only values degrading to the default; `/api/open`
  endpoint validating the path like `/api/reveal` and spawning detached with a
  reaper thread. *Done when:* `cargo test` green with new tests; `curl -X POST
  /api/open` with a bad path returns 400 with an error body, with a real repo
  path returns 200 (and launches the editor when run interactively).
- [x] **Step 2 - frontend: open in editor + copy cd** - a `ProjectActions`
  row rendered in the Projects tab group headers and the project drawer
  header: Open in editor (POST `/api/open` with the project root, surface
  errors), Copy cd (clipboard `cd <root>`, transient copied state, same
  pattern as the inspect drawer's copy actions). *Done when:* build green;
  buttons visible in both spots; clicking Copy cd puts `cd <root>` on the
  clipboard (Playwright-verifiable); open-in-editor errors render instead of
  failing silently.
- [x] **Step 3 - frontend: stop-all dialog** - `StopAllDialog` on the feature
  12 contract: pre-signal list of stoppable services with PID/command,
  services without PID marked unstoppable, sequential graceful stops with
  per-service outcome badges, escalate phase listing only survivors with a
  single explicit force confirmation, per-service errors inline; wired from
  both action rows. A project with zero stoppable services (no PIDs) shows an
  explanatory empty state instead of a confirm button. *Done when:* build green; against the running app with
  two throwaway `python3 -m http.server` listeners started from a repo under
  `~/Code`, stop-all lists both, stops both gracefully, and the ports verify
  released; the self-stop refusal renders as an inline error, not a crash.
- [x] **Step 4 - acceptance check** - `/check` behavior against the running
  dev build: Playwright walk of Projects tab, drawer, both new actions, and
  the stop-all happy path + escalate rendering; console clean. *Done when:*
  evidence captured (screenshots + outcomes) and all done-whens above hold.

## Files / areas

- `src/config.rs` - `editor` key + default + tests
- `src/main.rs` - `/api/open` route + handler (+ any shared path validation)
- `web/src/components/ProjectGroups.tsx`, `ProjectDrawer.tsx` - action rows
- `web/src/components/StopAllDialog.tsx` - new
- `web/src/lib/stop.ts` - reuse `postStop`; no contract changes

## Data / contracts

- `DevSnapshot` untouched. New request shape `{path}` for `/api/open`
  (mirrors `/api/reveal`). Config gains `editor: string` - additive, old
  files load via serde default.

## Testing

- Test gate on: config default/parsing logic and the editor command splitting
  get unit tests in the same diff (step 1). Steps 2-4 are UI/integration:
  Playwright + build evidence per the standards; no frontend runner exists.

## Notes for the AI

- Follow the feature 12 phase vocabulary (confirm/working/escalate/error) and
  the StopDialog visual language; stop-all must never signal anything before
  the user has seen the full list.
- The backend already refuses stopping its own pid; treat that error as an
  expected inline outcome in the dialog.
- Reap the spawned editor process (thread + `wait()`) so the server never
  accumulates zombies.
- No em dashes in any generated text; hyphens only.
