# Feature: Signed Windows releases

**From build-plan:** feature 17
**Status:** merged; steps 5-7 (rc tag, v0.1.1 tag, Windows VM retest) run from main - progress tracked here

## Goal

Authenticode-sign `portdoc.exe` in the release pipeline via Azure Artifact
Signing so Smart App Control and SmartScreen accept installs on fresh Windows 11
without overrides. Ships as v0.1.1. The v0.1.0 finding: SAC hard-blocks the
unsigned exe with no "run anyway" path, so unsigned means uninstallable for a
growing slice of Windows users.

## In scope

- Azure side: Public Trust certificate profile on the existing
  `bradtraversy-signing` account (identity validation Completed 2026-07-10,
  valid to 2027-07-10), a GitHub OIDC federated identity, and the
  Certificate Profile Signer role for it.
- Pipeline side: `allow-dirty = ["ci"]` in `dist-workspace.toml`, a hand-patched
  signing block on the Windows leg of `release.yml` (unzip, sign with
  `azure/artifact-signing-action@v2` + RFC3161 timestamp, re-zip, fix checksums),
  `id-token: write` permission, GitHub secrets.
- Release: version 0.1.1, README known-limitations update, `v0.1.1-rc.1`
  prerelease to prove signing, then public `v0.1.1`.
- Verification: signature check from Linux (`osslsigncode`), then the manual
  Windows VM retest of the full install path on the signed build.

## Out of scope

- macOS signing/notarization - curl and brew paths dodge Gatekeeper; needs the
  $99/yr Apple Developer membership. Decide later.
- cargo-dist's native `azure-windows-sign` - swap in when PR #2396 merges
  (project is community-maintained post-axo; don't wait on it).
- Windows stop support, crates.io publishing, Scoop - all separate items.
- Signing the installer scripts themselves (`install.ps1` is fetched over TLS
  and run in-memory; SAC checks the exe, not the script).

## Build loop

Build one step at a time, never the whole feature at once.

1. Plan mode lays out the step before any code.
2. The AI implements just that step.
3. It shows the diff (not full files); you read it and understand it.
4. You approve, then choose whether to commit a checkpoint or roll straight on.
   Checkpoints are optional; `/complete` makes the real feature-level commit at the end.

Never accept a step you haven't read. If a diff is too big to review, the step was too big, so split it.

## Build steps

Steps 1-4 happen on the feature branch. Steps 5-7 run from main after
`/complete` merges, mirroring how 15c handled its release steps (progress notes
go on the archived spec).

- [x] **Step 1 - Azure certificate profile and OIDC identity** - guided setup,
  mostly Brad in the portal / `az` CLI with the AI driving. Create a Public
  Trust certificate profile on `bradtraversy-signing` (East US, resource group
  `portdoc-signing`); common name will be "Brad Traversy" from the completed
  validation. Create the GitHub OIDC identity (app registration or user-assigned
  managed identity with a federated credential) - subject bound to a GitHub
  Actions **environment** (`release`) rather than a tag ref, because federated
  credential subjects are exact-match and tag refs change every release. Assign
  it the Artifact Signing Certificate Profile Signer role on the account. Add
  repo secrets `AZURE_TENANT_ID` and `AZURE_CLIENT_ID`; endpoint
  (`https://eus.codesigning.azure.net`), account, and profile names are not
  secrets and go straight in the workflow. *Done when:* the cert profile shows
  Active in the portal, and `az` (or the portal) shows the federated credential
  with subject `repo:bradtraversy/portdoc:environment:release` and the role
  assignment on the signing account.
- [x] **Step 2 - workflow signing patch** - add `allow-dirty = ["ci"]` to
  `dist-workspace.toml` so dist tolerates the hand-edited workflow. Patch
  `release.yml`: add `id-token: write` to permissions, set
  `environment: release` on `build-local-artifacts` (create the environment on
  GitHub, no protection rules), and insert steps between `Build artifacts` and
  `Post-build`, gated on `contains(matrix.targets, 'x86_64-pc-windows-msvc')`:
  unzip the built archive from `target/distrib/`, sign `portdoc.exe` with
  `azure/artifact-signing-action@v2` (RFC3161 timestamp server), re-zip
  preserving the archive layout, recompute the zip's sha256, and patch the new
  hash into the local `dist-manifest.json` (and any `.sha256` sidecar) with
  `jq`/PowerShell so the global job's checksum files and installers match the
  signed artifact. Check the action's current input names against its README
  during implement, not from memory. *Done when:* `dist plan` runs clean
  locally with the dirty workflow, YAML parses, and the diff shows checksum
  patching before the upload step.
- [x] **Step 3 - version bump** - `Cargo.toml` to 0.1.1 (Cargo.lock follows),
  since the tag must equal the Cargo version. *Done when:* `cargo check` passes
  and `dist plan` reports 0.1.1 across all five targets.
- [x] **Step 4 - README honesty update** - replace the known-limitations note
  about Windows SAC blocking the unsigned exe with "Windows binaries are
  Authenticode-signed (as Brad Traversy) since v0.1.1". *Done when:* README has
  no stale unsigned-Windows warning.
- [ ] **Step 5 - rc proves signing (post-merge, from main)** - Brad pushes tag
  `v0.1.1-rc.1`; pipeline runs. Download the Windows zip from the prerelease,
  verify from Linux: `osslsigncode verify` on the extracted exe shows signer
  CN "Brad Traversy" and a valid timestamp, and the published `.sha256` matches
  the signed zip. Iterate here until green. *Done when:* signature and checksum
  both verify on the rc assets.
- [ ] **Step 6 - public v0.1.1** - Brad pushes tag `v0.1.1`. *Done when:* the
  release wears the Latest badge, the Homebrew formula bumps to 0.1.1,
  `portdoc.dev/install.sh` still installs (latest-URL now resolves to v0.1.1),
  and the signature verifies on the final Windows asset.
- [ ] **Step 7 - Windows VM retest (manual)** - fresh Win11 VM with Smart App
  Control on: `irm portdoc.dev/install.ps1 | iex`, then run `portdoc`. *Done
  when:* no SAC block, the dashboard loads at `127.0.0.1:7788` with real
  services, and stopping is correctly absent (typed "not supported"). The old
  vault checklist note was pruned; this step is the checklist now.

## Files / areas

- `dist-workspace.toml` - `allow-dirty = ["ci"]`
- `.github/workflows/release.yml` - permissions, environment, Windows signing block
- `Cargo.toml` / `Cargo.lock` - 0.1.1
- `README.md` - known-limitations update
- Azure portal / `az` CLI and GitHub repo settings (environment + secrets) - config, not repo files

## Data / contracts

- No `DevSnapshot` or API changes.
- Load-bearing pipeline contract: the sha256 that reaches the global job must be
  computed from the **signed** zip. Signing after checksum consumption ships
  installers and `.sha256` files that reject the real artifact.
- GitHub OIDC subject contract: `repo:bradtraversy/portdoc:environment:release`.
  Renaming the environment or repo breaks signing auth.

## Testing

- No new Rust logic, so no new unit tests; `cargo test` must stay green (the
  gate applies to logic-bearing code, and this feature is pipeline config).
- Evidence per step: `dist plan` output (steps 2-3), `osslsigncode verify`
  output and checksum comparison (step 5), release page + formula + installer
  run (step 6), VM screenshot (step 7).

## Notes for the AI

- `dist generate`/`dist init` will overwrite the hand-patched `release.yml`.
  `allow-dirty = ["ci"]` only stops dist from failing on the drift; any future
  regeneration must re-apply the signing block. Re-check this whenever the
  cargo-dist version bumps, and drop the patch entirely if PR #2396's native
  `azure-windows-sign` lands.
- Signing failures should fail the release job hard - never fall through to
  publishing an unsigned exe as v0.1.1+.
- Brad runs outward actions himself (tag pushes, secret creation, portal
  clicks); the permission classifier blocks them in auto mode. Prepare exact
  commands for him to paste.
- The uncommitted `blueprint/build-plan.md` hunk (the feature 17 entry) rides on
  this feature's branch.
- Verify current `azure/artifact-signing-action@v2` inputs and the artifact
  signing endpoint format from the live README/docs during implement - the
  service was renamed from Trusted Signing in Jan 2026 and details may have
  moved again.
- `environment: release` applies to all five matrix legs (GitHub can't scope an
  environment to one matrix entry); harmless deployment records on the other
  four legs are expected.
