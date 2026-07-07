# Current Feature

> **Generated file.** Holds the one feature or fix being built right now. Run
> `/feature <number-or-name>` to spec a build-plan feature, or `/fix "<bug>"` for
> an ad-hoc fix. Build one thing at a time; `/complete` archives it (to
> `blueprint/history/features/` or `blueprint/history/fixes/`) and resets this file.

## VS Code extension names in labels

**Type:** Fix

### The problem

Feature 14a labels every VS Code helper process plain "VS Code", so eight rows
read identically and the user still has to open the command sub-line to learn
that :38908 is Pylance. People should know exactly what something is from the
headline.

### The fix

When a command references a user-extension path
(`.vscode/extensions/<publisher>.<name>-<version>/`), refine the label to
"VS Code (<Extension>)":

- Parse the directory segment after the marker, strip the trailing version
  (`-2026.2.1`), drop the publisher (before the first `.`), strip a
  `vscode-` prefix or `-vscode` suffix, and humanize hyphens
  (`vscode-pylance` -> "Pylance", `rust-analyzer` -> "Rust Analyzer").
- The extension path is itself a signal: it labels even when the process runs
  under `node` instead of the `code` binary. Framework matches still win first.
- `.vscode-insiders/extensions/` counts too. Utility processes without an
  extension path stay plain "VS Code". Built-in extensions (under the app's
  own `resources/app/extensions`) are deliberately not parsed - they run as
  generic utility processes.

Must not break: framework precedence, the 14a vocabulary, basename-only
matching for everything else.

### Build steps

- [x] **Step 1 - extension-name refinement.** `vscode_extension(command)` in
  `src/label.rs`, checked between the framework table and the desktop-app
  table; tests for the real Pylance shape, a `node`-run extension server, a
  suffix-style id (`esbenp.prettier-vscode` -> "Prettier"), a hyphenated name
  (`rust-analyzer` -> "Rust Analyzer"), version stripping, and the plain
  "VS Code" fallback for utility processes.
  Done when: `cargo test` green with the new cases, `cargo clippy` clean, and
  `portdoc --json` on this machine shows "VS Code (Pylance)" for the pylance
  servers while the `--type=utility` rows stay "VS Code".

### Verify

- `cargo test`, `cargo clippy`, `portdoc --json` filtered to VS Code rows.
