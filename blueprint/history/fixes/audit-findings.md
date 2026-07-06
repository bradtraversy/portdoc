# Fix: Audit findings cleanup

**Type:** Fix
**Status:** complete

## Goal

Close the three findings from the post-feature-2 `/audit`: the bare `/api`
route-precedence edge, free-floating mock ID literals, and the runtime panic
on server-loop errors.

## Build steps

- [x] **Step 1 - bare `/api` stays in the API namespace** - extend the
  fallback guard so `GET /api` (no trailing slash) is 404, matching the
  feature 2 contract. *Done when:* `curl /api` returns 404 and `/api/health`,
  `/`, `/dashboard` behave as before.
- [x] **Step 2 - mock ID constants** - hoist every repeated service and
  project ID in `src/mock.rs` to a `const`, referenced everywhere. *Done
  when:* `portdoc --json` output is byte-identical to before the change
  (ignoring `generated_at`) and every `project.service_ids` /
  `conflict.service_ids` entry matches a real service ID.
- [x] **Step 3 - graceful server-error exit** - replace
  `serve(...).expect("server error")` with an error message on stderr and
  exit 1. *Done when:* `cargo clippy` is clean and normal startup/shutdown
  still works.

## Files / areas

- `src/main.rs` - fallback guard, serve error handling
- `src/mock.rs` - ID constants

## Testing

No test gate. Evidence: `cargo check`/`clippy`/`fmt`, curl matrix for the
route edge, `--json` before/after diff for the mock refactor.
