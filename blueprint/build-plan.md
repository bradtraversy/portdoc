# Build Plan

> One of the two planning docs you provide. Write it yourself or with the AI's help.
>
> Keep this as a checkbox list. Run `/feature` with no number to spec the next
> unchecked item, or `/feature 3` / `/feature "linux probe"` to pick a specific
> one. Completed features get checked off here, so the build plan doubles as the
> progress tracker.

- [x] 1. **Mock snapshot contract** - define the shared `DevSnapshot` shape, add `/api/health`, add `/api/snapshot` with mocked services, and make `portdoc --json` print the same snapshot.
- [x] 2. **Embedded web shell** - build the Vite app for production, serve it from the Rust binary, keep the local server on `127.0.0.1:7788`, and open the browser by default.
- [x] 3. **Mock dashboard UI** - replace the Vite starter screen with dashboard summary cards, project groups, and a services table driven by mocked snapshot data.
- [x] 4. **Platform probe boundary** - create the platform probing abstraction and a Linux-first implementation path without committing to macOS or Windows internals yet.
- [x] 5. **Linux listening-port probe** - collect listening TCP ports on Linux, join sockets to owning PIDs where possible, and attach process name, command, cwd, and user.
- [x] 6. **Snapshot adapter** - convert real probe output into `DevSnapshot`, including unknown owners, predictable IDs, localhost URLs, and refresh-safe API responses.
- [x] 7. **Project grouping** - detect project roots from `package.json`, `.git`, workspace files, lockfiles, and cwd, then group services by project.
- [x] 8. **Developer labels** - detect package manager, git branch, common frameworks, runtimes, and tools such as Next.js, Vite, React scripts, Express, Bun, Convex, Prisma Studio, Astro, Remix, and Nuxt.
- [x] 9. **URL and exposure labels** - classify services as local, LAN visible, Docker, or unknown, and generate useful local URLs for HTTP-looking services.
- [ ] 10. **Conflicts and stale hints** - surface port conflicts, stale dev servers, old project processes, expected-but-missing apps, and conflict-focused actions.
- [ ] 11. **Search and filters** - add service filters for framework, runtime, API, database, Docker, unknown, LAN visible, stale, and conflict, plus quick text search.
- [ ] 12. **Safe stop action** - stop one service or port with confirmation, show exact PID/command/cwd before stopping, try graceful termination first, verify the port releases, and keep force kill behind a second explicit confirmation.
- [ ] 13. **Inspect drawer and quick actions** - add service details, open URL, copy URL, reveal project folder, copy kill command, ignore service, and advanced process details.
- [ ] 14. **Docker and Advanced tabs** - add Docker and Compose hints, a Docker tab, raw socket details, JSON export, unknown owner diagnostics, and Advanced data without crowding the dashboard.
- [ ] 15. **Release and install path** - add Linux, macOS, and Windows release builds, checksums, `install.sh`, `install.ps1`, README install docs, screenshots, and the path toward one-command installs from `portdoc.dev`.
