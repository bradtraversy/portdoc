# Current Feature

> **Generated file.** Holds the one feature or fix being built right now. Run
> `/feature <number-or-name>` to spec a build-plan feature, or `/fix "<bug>"` for
> an ad-hoc fix. Build one thing at a time; `/complete` archives it (to
> `blueprint/history/features/` or `blueprint/history/fixes/`) and resets this file.

## Feature 16a - Project rollups

**Branch**: `feature/project-rollups`

### Goal

Make the Projects tab earn its existence: each project group's header answers
"how much is running here, is any of it unhealthy, and on which ports" at a
glance, from data already in the snapshot. Frontend-only; no contract change,
no new probing.

### In scope

- Per-project header rollups on the **Projects tab**: service count, stale
  count, LAN-visible count, and the group's ports as clickable chips that open
  the inspect drawer (same interaction as the Docker tab's port chips).
- The same rollup treatment on the Projects tab's Docker and Ungrouped
  sections, where the counts apply.
- The Dashboard's grouped block stays exactly as it is (compact); the shared
  `ProjectGroups` component grows a `detailed` flag the Projects tab sets.

### Out of scope

- Anything needing new data: package.json facts, git dirty/commit age (16b),
  project actions (16c).
- Rollups on the Dashboard - it stays calm; the stat cards already summarize
  globally there.
- Sorting/filtering projects.

### Build steps

- [x] 1. Rollup helpers in `derive.ts` (counts computed from visible member
  services - ignored services excluded, consistent with every other count) and
  the detailed header rendering in `ProjectGroups.tsx` behind a `detailed`
  prop: count text, stale/LAN badges only when nonzero, port chips opening the
  inspect drawer. Projects tab passes `detailed`; Dashboard does not. Done
  when the Projects tab shows rollups on this machine's real groups, the
  Dashboard block is visually unchanged, and lint + build are green.

### Files / areas

- `web/src/lib/derive.ts` - rollup computation (pure)
- `web/src/components/ProjectGroups.tsx` - detailed header variant
- `web/src/App.tsx` - Projects tab passes the flag

### Data / contracts

None. Reads the existing `DevSnapshot` shapes only.

### Testing

Frontend has no runner (see AGENTS.md); UI-only change rides on browser
evidence plus the build, per the testing gate. The rollup math is trivial
filter/count over already-typed data; if 16b/16c grow real frontend logic,
Vitest gets revisited then.

### Notes for the AI

- Stale/LAN badges reuse the existing `Badge` variants (warn + dot), matching
  the row badges so the vocabulary stays consistent.
- Port chips: same visual as `DockerView`'s chips; clicking opens the drawer
  via `useInspect` with `servicesOnPort`.
- Counts must respect the ignored set from `useConfig` - a hidden service is
  hidden everywhere.
