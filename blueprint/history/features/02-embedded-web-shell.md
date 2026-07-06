# Feature: Embedded web shell

**From build-plan:** feature 2
**Status:** complete

## Goal

Serve the production Vite build from the Rust binary at `127.0.0.1:7788` and
open the browser by default, giving PortDoc its single-binary shape. The UI is
still the Vite starter page; feature 3 replaces its content. This feature also
settles the CLI surface: `portdoc` (default) and `portdoc ui` both launch the
server, `--no-open` suppresses the browser.

## In scope

- `rust-embed` dependency (the overview leaves rust-embed vs include_dir to
  this feature; the project plan's stack names `rust-embed`, so that's the pick)
- `build.rs` that creates `web/dist` if missing, so `cargo check` works on a
  fresh clone before the first web build
- Static serving from the embedded `web/dist`: `/` serves `index.html`, asset
  paths serve with correct content-types via a small extension map (no
  mime-guess dependency)
- SPA fallback: unknown non-`/api` GET paths return `index.html`; unknown
  `/api/*` paths stay 404
- Friendly 503 ("UI not built - run `npm run build` in web/") when the embed
  has no `index.html`
- `open` dependency + browser opening after the listener binds; `--no-open`
  flag; open failures log a warning, never crash the server
- `ui` subcommand as an explicit alias of the default launch behavior
- One line in `AGENTS.md` Commands noting the server embeds `web/dist`, so
  build the frontend first when the UI matters

## Out of scope

- Any change to `web/src` - the dashboard UI is feature 3
- Release builds, installers, cross-platform packaging (feature 15)
- Auto-rebuilding the frontend from cargo; dev flow stays `npm run build` then
  `cargo run`
- Changing `/api/*` behavior from feature 1

## Build steps

- [x] **Step 1 - embedded static serving** - add `rust-embed` + `build.rs`,
  replace the `/` placeholder with a fallback handler serving the embedded
  dist: `index.html` at `/`, assets by path with correct content-types, SPA
  fallback for non-API paths, 404 for unknown `/api/*`, 503 hint when the
  embed is empty. *Done when:* `curl -i /` returns the starter HTML as
  `text/html`; the hashed `/assets/*.js` and `*.css` return
  `text/javascript` / `text/css`; `/some/client/route` returns `index.html`;
  `/api/nope` is 404; `/api/snapshot` unchanged; Playwright screenshot of
  `127.0.0.1:7788` shows the Vite starter page with no console errors.
- [x] **Step 2 - browser open + CLI surface** - add the `open` crate,
  `--no-open` flag, and the `ui` subcommand alias; after binding, open
  `http://127.0.0.1:<port>` unless `--no-open`; log a warning if the open
  command fails. *Done when:* `portdoc --no-open` and `portdoc ui --no-open`
  both serve without opening a browser and print the URL; a default run
  opens the served page in the browser; `--json` still prints and exits
  without opening anything.

## Files / areas

- `Cargo.toml` - add `rust-embed`, `open`
- `build.rs` - new: ensure `web/dist` exists at compile time
- `src/main.rs` - embed struct, static/fallback handlers, mime map, CLI
  changes (`--no-open`, `ui` subcommand), browser open
- `AGENTS.md` - one-line Commands note about the embedded frontend build

## Data / contracts

- No `DevSnapshot` changes; `/api/health` and `/api/snapshot` byte-identical
  to feature 1
- Route precedence contract: `/api/*` never falls back to HTML; everything
  else falls back to `index.html` (SPA)
- Debug builds read `web/dist` from disk at runtime (rust-embed default), so
  UI edits don't need a cargo rebuild; release builds embed the files

## Testing

No test command declared in `AGENTS.md`; test gate off. Evidence: `cargo
check`/`clippy`/`fmt`, `npm run build`, curl content-type checks per route
class, Playwright screenshot of the served starter page plus console check,
and CLI runs for `--no-open`, `ui`, `--json`. The browser-open path is proven
by one real default-run open.

## Notes for the AI

- Resolves the overview's `portdoc ui` open question: `ui` is an alias; note
  it for the next `/overview` regeneration
- Keep handlers panic-free; startup bind may still panic per standards
- Content-type map needs only what Vite emits: html, js, css, svg, png, ico,
  json, map, txt, woff2
- Bind stays `127.0.0.1` only - no LAN exposure of the dashboard itself
