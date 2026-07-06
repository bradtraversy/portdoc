# Feature: Mock dashboard UI

**From build-plan:** feature 3
**Status:** complete

## Goal

Replace the Vite starter with the real PortDoc dashboard driven by
`/api/snapshot` mock data: summary cards, conflict/stale callouts, project
groups, and the services table, in the locked prototype look. After this
feature the served app at `127.0.0.1:7788` is PortDoc, not a placeholder.

## Design reference

`prototypes/dashboard.html`, `prototypes/services.html`, and
`prototypes/theme.css` (the current graphite/amber versions with PID
sub-lines, kebab actions, and the protected badge). Build against these, not
memory. `/complete` discards `prototypes/` once this feature consumes them.

## In scope

- UI stack adoption per coding-standards: Tailwind CSS (v4, Vite plugin),
  TanStack Table, Lucide (`lucide-react`); "shadcn/ui-style" means small
  hand-built components in `components/ui/`, not the shadcn CLI (which drags
  in Radix and its own theming - more than this needs)
- Port `prototypes/theme.css` tokens into the app stylesheet as a Tailwind
  `@theme` block
- TS types for the locked `DevSnapshot` contract (deferred here by feature 1)
- `useSnapshot` hook: fetch, loading, error, manual refetch, fetched-at age
- Vite dev proxy for `/api` -> `127.0.0.1:7788` so `npm run dev` works
  against a running backend
- App shell: topbar (logo, addr chip, snapshot age, Refresh) + tab bar with
  live counts; Dashboard and Services tabs functional, the other four
  (Projects, Conflicts, Docker, Advanced) as quiet placeholders
- Dashboard view: 4 stat cards, conflict + stale callouts from mock data,
  project groups with service rows (port, process, URL, note, PID/command
  sub-line, status badges, protected badge on the portdoc row)
- Services table (TanStack): Service, Port, PID, Command, Project, Exposure,
  Status, Age columns, conflict row accent, stale dimming, inert kebab column
- Starter cleanup: remove `App.css`, starter assets, `public/icons.svg`;
  PortDoc ECG-mark `favicon.svg`; `index.html` title "PortDoc"
- Update the Styling section of `coding-standards.md` (stack is no longer
  "planned, not installed")

## Out of scope

- Search, filter chips, text filter (feature 11)
- Inspect drawer and quick actions beyond an Open link (feature 13); Stop and
  kebab render disabled/inert
- Conflicts tab content and conflict actions (feature 10), Docker tab (14),
  Advanced tab (14)
- Any backend change; the API and mock data are frozen contract
- Auto-polling; Refresh is manual (refresh cadence is a later concern)

## Build steps

- [x] **Step 1 - UI stack + theme port** - install `tailwindcss`,
  `@tailwindcss/vite`, `@tanstack/react-table`, `lucide-react`; wire the Vite
  plugin and `/api` dev proxy; replace `index.css` with Tailwind + the ported
  `@theme` tokens; strip starter files; title/favicon; minimal themed
  placeholder App. *Done when:* `npm run build` and `npm run lint` pass and
  the served page shows the dark theme background/text (screenshot), no
  console errors.
- [x] **Step 2 - contract types + shell** - `DevSnapshot` TS types,
  `useSnapshot` hook, topbar (logo, `127.0.0.1:7788` chip, "snapshot Ns ago",
  working Refresh), tab bar with counts from the snapshot, placeholder
  panels, loading and error states. *Done when:* served app shows the shell
  with real mock counts (services 8, conflicts 2, docker 2); killing the
  backend and refreshing dev shows the error state, not a blank page.
- [x] **Step 3 - Dashboard view** - stat cards, danger/warn callouts,
  project groups with full service rows per the prototype (badges, sub-line,
  protected portdoc row, disabled Stop, inert kebab). *Done when:* Dashboard
  visually matches `prototypes/dashboard.html` against the same data shape
  (screenshot comparison), zero console errors.
- [x] **Step 4 - Services table** - TanStack Table rendering all mock
  services with the prototype's column set, conflict left-accent, stale
  dimming, exposure/status badges, age column, inert kebab. *Done when:* the
  Services tab lists all 8 mock services with correct cells and matches
  `prototypes/services.html` (screenshot), and the production path works:
  `npm run build` + `cargo run` serves the finished UI at `127.0.0.1:7788`.

## Files / areas

- `web/package.json` / lock - new deps
- `web/vite.config.ts` - Tailwind plugin, `/api` proxy
- `web/index.html`, `web/public/favicon.svg` - identity
- `web/src/index.css` - Tailwind + `@theme` tokens (from `prototypes/theme.css`)
- `web/src/lib/types.ts`, `web/src/lib/useSnapshot.ts` - contract + data hook
- `web/src/components/ui/*` - badge, button, card (shadcn-style, hand-built)
- `web/src/components/*` - TopBar, TabBar, StatCards, Callouts,
  ProjectGroups, ServicesTable, Placeholder
- `web/src/App.tsx` - shell + tab state; starter files removed
- `blueprint/context/coding-standards.md` - Styling section reality update

## Data / contracts

- Consumes the locked `DevSnapshot` exactly as feature 1 archived it; TS
  types must mirror it field-for-field (optionals omitted, lowercase
  exposure strings, `stale: { reason }`, epoch-ms `generated_at`)
- No API changes

## Testing

No test command declared; gate off. Evidence per step: `npm run build`,
`npm run lint`, Playwright screenshots against the dev server (steps 1-3)
and the embedded production server (step 4), console-error checks each time,
and a side-by-side against the prototype files for steps 3-4.

## Notes for the AI

- Status colors and dot+label badge rule come from the theme: never color
  alone; green running, red conflict, yellow caution, blue docker; amber is
  interaction-only
- Mono font for data (ports, PIDs, commands, paths), sans for UI chrome
- The tab bar scrolls within itself on narrow widths (prototype fix); keep it
- Fetch failures surface in the UI (coding-standards error rule)
- Keep components one-job; table cell renderers small
