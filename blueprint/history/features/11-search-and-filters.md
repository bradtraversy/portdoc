# Feature: Search and filters

**From build-plan:** feature 11
**Status:** complete

## Goal

The Services tab gets a toolbar: quick text search plus category filter
chips (Framework, Runtime, API, Database, Docker, Unknown, LAN visible,
Stale, Conflict). This is the noise-control feature: two clicks to see
only stale dev servers, or type "astro" and see nothing else. Frontend
only - every signal already exists in the snapshot.

## In scope

- **Pure filter logic** in `web/src/lib/filter.ts`:
  - `matchesQuery`: case-insensitive substring over port (with and
    without `:`), process name, command, cwd, user, framework, and
    project name
  - chip predicates mapping the build plan's categories onto existing
    labels: Framework = dev-server labels (Next.js, Vite, Astro, Remix,
    Nuxt, React scripts); Runtime = Bun, Convex, Prisma Studio; API =
    Express; Database = Postgres, Redis; Docker / LAN visible = exposure;
    Unknown = no owning pid; Stale = stale hint present; Conflict = in a
    conflict's service_ids
  - semantics: selected chips OR together (pick "Stale" + "Conflict" to
    see anything needing attention); the text query ANDs on top; no
    selection means show everything
- **Toolbar UI** in `ServicesTable`: search input (Lucide Search icon,
  theme-styled), one toggle chip per category (amber accent when active -
  interaction color, per the theme rules), and a Clear button that
  appears only when something is active
- Footer count becomes "{shown} of {total} services" while filtered
- Empty state row ("No services match.") with the Clear affordance when
  filters eliminate everything
- Filter state is local to the Services tab and resets on reload -
  no persistence (config storage is feature 13's question)

## Out of scope

- Dashboard filtering - the dashboard stays a calm overview; the top-bar
  search jumps to Services instead of filtering project groups in place
- Persisted filter preferences (needs feature 13's config decision)
- Per-framework dropdowns or counts on chips - flat category toggles
  this pass
- Backend changes of any kind - zero Rust in this feature
- Ignore/hide-forever semantics (feature 13's ignore-service)

## Build steps

- [x] **Step 1 - search** - `filter.ts` with `matchesQuery`, search input
  wired into `ServicesTable`, footer shows filtered count. *Done when:*
  `npm run lint` + `npm run build` pass and typing in the browser narrows
  rows live.
- [x] **Step 2 - category chips** - chip predicates + toggle row + OR/AND
  semantics + Clear button + empty state. *Done when:* lint + build pass
  and in the browser each chip's row count matches the machine's known
  state (Stale = 1, LAN visible = wildcard binds, Unknown = ownerless
  rows), combined chip+query narrows further, Clear restores everything,
  zero console errors.
- [x] **Step 3 - top-bar jump-search** (added in review with Brad) - the
  search input moves to the TopBar as the app's single search box; query
  state lifts to `App`; typing from any tab jumps to Services with the
  query applied (the input stays mounted, so focus survives the jump);
  the Services toolbar keeps chips + Clear, and Clear resets both chips
  and the query. *Done when:* lint + build pass and typing from the
  dashboard lands you on Services, already narrowed, without losing
  keystrokes.

## Files / areas

- `web/src/lib/filter.ts` - new: query + chip predicates
- `web/src/components/ServicesTable.tsx` - toolbar, filtered rows,
  footer count, empty state
- No other files

## Data / contracts

- No contract involvement - reads existing `Service`, `ProjectGroup`,
  and `Conflict` fields only
- Category vocabularies live in `filter.ts` beside their use; they
  mirror (not import) the Rust label tables - if that drifts, the fix is
  feature 14's vocabulary work, not a shared-constants scheme across the
  language boundary

## Testing

Frontend has no test runner (gate off, and adding one is /tests' job,
never a silent mid-feature install). `filter.ts` is small predicate
logic; verification is browser evidence against known machine state plus
lint and the production build - exactly the integration surface the
standards route away from unit tests.

## Notes for the AI

- Amber accent marks the ACTIVE chip only - status colors stay reserved
  for status (theme rule from the prototypes)
- Keep the toolbar responsive: chips wrap (`flex-wrap`), the search input
  stays usable at narrow widths
- `rules-of-hooks` is an error in oxlint - hooks stay at the top of
  `ServicesTable`, filtering is plain derivation
- Filtering happens on the `Row[]` before `useReactTable` - do not adopt
  TanStack's column-filter machinery for this; it is more surface than
  the job needs

## Completion notes

- Shipped with one mid-run scope addition at Brad's direction: the search
  box became a top-bar global input that jumps any tab to Services with
  the query applied; the Services toolbar kept chips + Clear
- One real bug shipped and was caught during Brad's manual review: typing
  froze the browser. Root cause (confirmed via TanStack docs): unmemoized
  data arrays passed to useReactTable cause infinite re-render loops -
  latent since feature 3, detonated by search's rapid re-renders. Fixed by
  memoizing rows and filtered. The Playwright MCP hang during acceptance
  was the same bug freezing the shared renderer, not tooling
- Browser evidence was manual (Brad) after the Playwright backend wedged:
  search narrowing, chip counts vs known machine state, jump-search focus
  retention, and Clear all confirmed
- filter.ts vocabularies mirror (not share) the Rust label tables; drift
  is feature 14's vocabulary work
