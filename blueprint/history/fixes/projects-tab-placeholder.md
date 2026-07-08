# Current Feature

> **Generated file.** Holds the one feature or fix being built right now. Run
> `/feature <number-or-name>` to spec a build-plan feature, or `/fix "<bug>"` for
> an ad-hoc fix. Build one thing at a time; `/complete` archives it (to
> `blueprint/history/features/` or `blueprint/history/fixes/`) and resets this file.

## Fix: Projects tab shows a stale placeholder

**Type:** Fix
**Branch**: `fix/projects-tab-placeholder`

### The problem

The Projects tab still renders the hardcoded placeholder "Project-grouped view
lands with real project detection (feature 7)" (`web/src/App.tsx`), even though
feature 7 shipped: the `ProjectGroups` component exists and renders the grouped
view on the Dashboard. The tab even shows a live project count badge next to a
"coming soon" note, which reads as broken.

### The fix

Render `ProjectGroups` for the `projects` tab in `App.tsx` and remove the
`projects` entry from the placeholder record (leaving only `advanced`), the
same wiring pattern the Docker tab got in 14b. Reuse `ProjectGroups` as-is -
projects, Docker, and Ungrouped sections are all "services grouped by owner",
which matches the overview's Projects tab description. Must not change the
Dashboard, which keeps rendering the same component.

### Build steps

- [x] 1. Wire `ProjectGroups` into the `projects` tab branch of `App.tsx` and
  drop the stale placeholder entry. Done when the Projects tab shows the same
  grouped sections as the Dashboard's Projects block and `npm run build` is
  green.

### Verify

Run the server, open the UI, click the Projects tab: project sections (name,
root, branch, package manager, framework badges, service rows) render instead
of the placeholder text. Dashboard unchanged. UI-only change, so evidence is
the browser plus the frontend build; no Rust code is touched (test gate not
in play).
