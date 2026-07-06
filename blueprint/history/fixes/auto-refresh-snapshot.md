# Fix: Auto-refresh snapshot

**Type:** Fix (not a build-plan item)
**Status:** complete

## Problem

The backend re-probes on every request, but the UI fetches once on page load
and then only when Refresh is clicked. Close or start a dev server and the
dashboard silently shows stale rows until a manual refresh - wrong posture
for a tool whose pitch is "what is running right now".

## Fix

Poll `/api/snapshot` from `useSnapshot` every 5 seconds:

- Background polls are silent: they must not set `loading`, so the Refresh
  button doesn't flicker disabled every tick; manual Refresh keeps the
  existing loading behavior
- Pause polling while the tab is hidden (`document.visibilityState`), and
  refresh immediately when the tab becomes visible again
- Guard against overlapping requests (skip a tick if one is in flight)
- Poll failures reuse the existing error state: the error banner appears
  automatically (with stale data still rendered) and clears on the next
  successful poll

## Out of scope

- WebSockets/SSE push - polling a ~50ms local endpoint is enough
- Configurable interval, backoff, or a pause toggle in the UI
- Backend changes of any kind

## Build steps

- [x] **Step 1 - polling in useSnapshot** - interval + visibility handling +
  silent background loads + in-flight guard, all inside the hook; no
  component API changes. *Done when:* `npm run lint` and `npm run build`
  pass, and in the browser a listener started after page load appears
  within ~5s without clicking, disappears within ~5s of being stopped, with
  zero console errors and no Refresh-button flicker.

## Files / areas

- `web/src/lib/useSnapshot.ts` - the only file

## Testing

UI/integration behavior: browser evidence per coding-standards (start and
stop a throwaway listener, watch rows change hands-free), plus lint and the
production build. No unit test - there is no frontend runner and this is
exactly the integration surface the standards say not to unit test.
