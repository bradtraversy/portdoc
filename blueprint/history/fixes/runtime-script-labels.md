# Current Feature

> **Generated file.** Holds the one feature or fix being built right now. Run
> `/feature <number-or-name>` to spec a build-plan feature, or `/fix "<bug>"` for
> an ad-hoc fix. Build one thing at a time; `/complete` archives it (to
> `blueprint/history/features/` or `blueprint/history/fixes/`) and resets this file.

## Fix: Label Paperclip and other runtime-run CLI tools

**Type:** Fix
**Branch**: `fix/runtime-script-labels`

### The problem

Paperclip (`node .../bin/paperclipai run -i default`, listening on :3100) shows
as a bare `node` row: it is not in the label vocabulary and its cwd maps to no
project root. The general class: any globally installed CLI or agent run by a
generic runtime (`node`, `python3`, `deno`) is invisible - only the script
path in the command carries its identity, and nothing reads it.

### The fix

Two layers in `src/label.rs`:

1. **Paperclip joins the vocabulary** - `("Paperclip", ["paperclipai",
   "paperclip"])` in the FRAMEWORKS table, so it gets a proper name.
2. **Runtime-script fallback** (the systemic part) - after frameworks, VS Code
   extensions, and desktop apps all miss: when the command's first token is a
   generic runtime (`node`, `deno`, `python`, `python3`), label with the
   cleaned basename of the first non-flag argument (the script), raw
   ("express-server", "http.server"). Skipped when the basename is itself
   meaningless (`index`, `main`, `server`, `app`, `cli`, `run`, `start`,
   `dev`, `script`) or another runtime. `clean_token` learns `.ts` and `.py`
   extensions.

Must not break: existing vocabulary precedence (frameworks > extensions >
desktop apps > fallback), stale accusations (fallback labels are never dev
servers), URL logic, and the filter chips (they match explicit allowlists in
`filter.ts`, so raw fallback labels do not join any chip). `bun` rows keep the
existing "Bun" label - the fallback only rescues otherwise-unlabeled rows.
Two existing tests assert `None` for commands like `node express-server.js`;
they change deliberately: an honest script-basename label is the new intended
behavior there.

### Build steps

- [x] 1. Vocabulary entry + runtime-script fallback in `label.rs` with unit
  tests (gate on); update the two tests whose expectation changes. Done when
  `portdoc --json` labels the live :3100 service "Paperclip" and `cargo test`
  is green.

### Verify

`portdoc --json` shows `"framework": "Paperclip"` on :3100; the UI row and
drawer show the label. A generic-runtime script row (e.g. any plain
`node something.js`) picks up its script basename. Rust-only change: tests
plus the JSON/UI check.
