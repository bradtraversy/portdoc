# Feature: Platform probe boundary

**From build-plan:** feature 4
**Status:** complete

## Goal

Create the probing abstraction that separates "ask the OS what's listening"
from everything else: raw probe data types, a `Probe` trait, a typed error,
and compile-time platform selection with a Linux implementation skeleton.
Feature 5 fills the Linux probe with real `/proc` work; feature 6 adapts probe
output into `DevSnapshot`. Nothing user-visible changes in this feature.

## In scope

- `src/probe/mod.rs`: raw data types (`ProbeOutput`, `ListeningSocket`,
  `ProcessInfo`, `Protocol`), the `Probe` trait, `ProbeError`, and
  `platform_probe()` selection (Linux -> `LinuxProbe`, other platforms ->
  `None`, so mac/windows builds compile without probing internals)
- `src/probe/linux.rs`: `LinuxProbe` skeleton returning an empty
  `ProbeOutput` (feature 5 replaces the body)
- `thiserror` dependency - this resolves the coding-standards TODO that
  deferred the error-crate choice to the first fallible module; typed enums
  beat hand-rolled `Display` impls as error types multiply (stop actions,
  config later)
- `#[cfg(test)]` unit tests exercising the platform selector and skeleton
  (the seam's only consumers until features 5-6, and the honest way to keep
  the module from being dead code)
- A module-level `#[allow(dead_code)]` with the one-line reason (boundary
  lands one feature before its consumers)
- `coding-standards.md`: resolve the error-handling TODO and refresh the
  module-layout TODO to reflect `src/probe/`

## Out of scope

- Real `/proc`/socket reading (feature 5)
- Adapting probe output to `DevSnapshot` or touching `/api/snapshot`
  (feature 6) - the API keeps serving the mock
- macOS/Windows probe implementations (unowned; overview open question)
- UDP - v0.1 scope is listening TCP ports per the plan
- Declaring a `test` command in AGENTS.md (the gate switch stays with
  `/tests`; the unit tests here are seam evidence, not the gate)

## Build steps

- [x] **Step 1 - probe types, trait, error** - add `thiserror`, create
  `src/probe/mod.rs` with the raw types, `Probe` trait, and `ProbeError`.
  *Done when:* `cargo check` passes with the module compiled in.
- [x] **Step 2 - Linux skeleton + platform selection + tests** - add
  `src/probe/linux.rs` (`LinuxProbe` returning empty output), cfg-gated
  `platform_probe()`, unit tests for selection and the skeleton, and the
  coding-standards doc updates. *Done when:* `cargo check`, `cargo clippy`,
  `cargo fmt --check` all clean and `cargo test` passes with the probe tests
  green.

## Files / areas

- `Cargo.toml` - add `thiserror`
- `src/probe/mod.rs` - new: the boundary
- `src/probe/linux.rs` - new: Linux skeleton
- `src/main.rs` - `mod probe;` declaration
- `blueprint/context/coding-standards.md` - error-handling decision recorded,
  module-layout TODO refreshed

## Data / contracts

Raw probe shapes (internal, pre-adapter - the `DevSnapshot` contract is
untouched):

- `ProbeOutput { sockets: Vec<ListeningSocket> }`
- `ListeningSocket { protocol, local_addr: IpAddr, port: u16, pid: Option<u32>, process: Option<ProcessInfo> }`
- `ProcessInfo { pid: u32, name, command, cwd, user, started_secs_ago }` -
  all metadata optional except `pid`, matching the unknown-owner reality
- `Protocol::Tcp` only for v0.1
- `Probe::probe() -> Result<ProbeOutput, ProbeError>`; `ProbeError` variants:
  `Io { path, source }` and `Unsupported`
- `platform_probe() -> Option<Box<dyn Probe>>` - `None` means "no probe for
  this platform", which feature 6 will surface as an empty-but-honest
  snapshot rather than a crash

## Testing

No test command declared in `AGENTS.md`; the gate stays off. Evidence:
`cargo check` + `clippy` + `fmt --check`, and `cargo test` run directly as
seam proof (selector returns a probe on Linux, skeleton returns empty output,
error type formats). UI untouched - no browser evidence needed.

## Notes for the AI

- Names per standards: modules `snake_case`, types `PascalCase`
- Keep the trait minimal - one `probe()` method plus a `name()` for
  diagnostics; resist speculative methods
- `unwrap`/`expect` stay out of probe code entirely - everything returns
  `Result`
- The `#[allow(dead_code)]` must carry its reason and should be removed by
  feature 6 when the adapter consumes the boundary
