# Feature: Release pipeline and installers

**From build-plan:** feature 15c
**Status:** pipeline built (steps 1-4, merged 2026-07-10); steps 5-6 (rc
prerelease + live installer checks on all three OSes, then the public v0.1.0
tag) run from main - check them off here as they land

## Goal

Ship v0.1 as installable binaries: a cargo-dist release pipeline that builds
PortDoc for Linux, macOS, and Windows on a version tag, publishes binaries and
checksums to GitHub Releases, and generates the standard install trio -
`curl | sh`, `irm | iex`, and `brew install bradtraversy/tap/portdoc` - plus
README install docs. After this, nobody needs Rust or Node to run PortDoc.

## In scope

- cargo-dist (`dist`) config: `dist-workspace.toml`, `[profile.dist]`, and the
  generated tag-triggered `.github/workflows/release.yml`
- Targets: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
  `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`
- Installers: `shell`, `powershell`, `homebrew` with tap
  `bradtraversy/homebrew-tap`
- Frontend embed guarantee: a `github-build-setup` fragment that runs
  `npm ci && npm run build` in `web/` before `dist build`, plus a hard guard
  that fails the job if `web/dist/index.html` is missing (build.rs silently
  creates an empty `web/dist`, so a skipped npm build would ship a blank
  dashboard - this guard is the difference between a broken release and a
  failed workflow)
- Tap repo `bradtraversy/homebrew-tap` + the token secret the generated
  workflow references
- README install section (trio + from-source fallback) with a dashboard
  screenshot, and a known-limitations note (Windows stop unsupported in v0.1)
- An `-rc.1` prerelease tag to prove the whole pipeline and run the deferred
  Windows live check before anything public

## Out of scope

- `portdoc.dev` website and `/install.sh` / `/install.ps1` redirects (later
  project; GitHub URLs are the v0.1 install path)
- crates.io publish, winget, Scoop, npm
- Apple codesigning/notarization (decided: skip for v0.1 - curl and brew paths
  bypass Gatekeeper quarantine)
- Implementing Windows stop (decided at spec time: stays typed "not supported
  on this platform" for v0.1, documented in the README; a real Windows kill is
  post-v0.1 work)

## Build loop

Build one step at a time, never the whole feature at once.

1. Plan mode lays out the step before any code.
2. The AI implements just that step.
3. It shows the diff (not full files); you read it and understand it.
4. You approve, then choose whether to commit a checkpoint or roll straight on.

Never accept a step you haven't read. If a diff is too big to review, the step
was too big, so split it.

## Build steps

- [x] **Step 1 - dist init and config** - install the `dist` CLI locally, run
  `dist init`, and shape the config: the five targets, the three installers,
  `tap = "bradtraversy/homebrew-tap"`, `publish-jobs = ["homebrew"]`, GitHub CI.
  Accept the `[profile.dist]` release profile it adds. *Done when:* `dist plan`
  runs clean locally and lists all five artifacts plus the three installers,
  and the generated `.github/workflows/release.yml` is in the diff.
- [x] **Step 2 - frontend build hook** - add the `github-build-setup` fragment
  (setup-node with npm cache, `npm ci && npm run build` in `web/`, then a
  `shell: bash` guard step failing unless `web/dist/index.html` exists) and
  regenerate the workflow so the fragment is injected. *Done when:* the
  regenerated `release.yml` shows the node/npm/guard steps inside the build
  job before `dist build`.
- [x] **Step 3 - tap repo and secret** - create the `bradtraversy/homebrew-tap`
  repo (public, README only) and add the token secret under whatever name the
  generated workflow's homebrew publish job references (expected
  `HOMEBREW_TAP_TOKEN`; read the workflow, don't assume). Token needs write
  access to the tap repo only. *Done when:* `gh secret list` on `portdoc`
  shows the secret and the tap repo exists.
- [x] **Step 4 - README install docs** - install section with the trio (curl,
  PowerShell, brew) pointing at GitHub Release URLs, from-source as the
  fallback, a dashboard screenshot, and the v0.1 known-limitations note
  (Windows stop returns "not supported"). *Done when:* README renders with all
  three install commands and the screenshot.
- [ ] **Step 5 - rc prerelease and live checks (post-merge)** *(in progress
  2026-07-10: v0.1.0-rc.1 released, all jobs green, homebrew correctly
  skipped; Linux passed - checksum verified, 25 services/4 projects probed,
  embedded UI real, installer.sh installs and runs. Remaining: macOS
  installer + dashboard, Windows install.ps1 + vault checklist (the 15b
  live check), brew install after v0.1.0 since prereleases skip the tap.)* - after
  `/complete` merges to main: bump version to `0.1.0-rc.1` (dist requires the
  tag to match the Cargo version), tag `v0.1.0-rc.1`, push the tag (explicit
  yes), watch the release workflow, then verify: binaries + sha256 checksums
  on the GitHub prerelease, `install.sh` on Linux and macOS, `install.ps1` on
  the Windows VM (this doubles as the deferred 15b live check - run the vault
  checklist), `brew install bradtraversy/tap/portdoc`. *Done when:* each
  installer puts a working `portdoc` on PATH and the dashboard populates with
  real services on all three OSes.
- [ ] **Step 6 - v0.1.0 public release** - only after every step 5 check
  passes: version back to `0.1.0`, tag `v0.1.0`, push (explicit yes), confirm
  the release publishes and the brew formula updates. *Done when:* the v0.1.0
  GitHub Release is live with all artifacts and `brew upgrade` sees it.

## Files / areas

- `dist-workspace.toml` (new) - dist config
- `Cargo.toml` - `[profile.dist]`; version bumps in steps 5/6
- `.github/workflows/release.yml` (generated) - tag-triggered release
- `.github/workflows/build-setup.yml` or similar (new) - the injected npm steps
- `README.md` - install section, screenshot, known limitations
- `blueprint/project-plan.md` - carries the uncommitted domain note already
  sitting on main; it rides along in this feature's commit
- External: `bradtraversy/homebrew-tap` repo, one repo secret on `portdoc`

## Data / contracts

- Tag format `vX.Y.Z` must equal the Cargo package version or dist fails the
  plan step - this is the pipeline's contract with git tags.
- Install URLs in the README point at
  `github.com/bradtraversy/portdoc/releases/latest/download/...`; when
  `portdoc.dev` later adds redirects they alias these, nothing here changes.
- No `DevSnapshot` or API changes.

## Testing

- No logic-bearing Rust code, so no new `cargo test` coverage; the existing
  suite and clippy must stay green (CI matrix already proves this per push).
- Step-level evidence: `dist plan` output (steps 1-2), `gh secret list` and
  the tap repo (step 3), rendered README (step 4).
- The real test is step 5: a live `-rc.1` release exercising every artifact
  and all three installers, including the deferred Windows live checklist from
  the vault (`Inbox/PortDoc Windows Test Checklist.md`).
- An empty or blank-dashboard binary is the defining failure; the step 2 guard
  exists so it can only manifest as a red workflow, never a shipped artifact.

## Notes for the AI

- `dist init` may rewrite `release.yml` wholesale on regeneration - always
  regenerate via the CLI, never hand-edit the generated workflow (edits die on
  the next `dist generate`). The build-setup fragment is the supported
  customization point.
- Existing `ci.yml` triggers on every push including tags; if the rc tag shows
  a redundant CI run, scoping `ci.yml` to branch pushes is an allowed one-line
  tidy, not scope creep.
- Tag pushes and repo creation are outward-facing: explicit yes from Brad
  before each (`/complete`'s pre-authorized push covers main only, not tags).
- Step 3 can use `gh repo create` and `gh secret set`; if a fine-grained PAT
  is needed for the tap, Brad creates the token, the session wires it in.
- Windows runner shells default to PowerShell; the guard step must declare
  `shell: bash` to be portable.
- `aarch64-unknown-linux-gnu` is the one target that may need cross-compile or
  arm-runner setup; if it fights back, drop the target for v0.1 instead of
  growing step 1 - four artifacts ship, arm Linux waits.
- The rc flow means Cargo.toml briefly reads `0.1.0-rc.1` on main between
  steps 5 and 6 - intentional, matches dist's tag/version contract.
