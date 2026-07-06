import { useCallback, useEffect, useRef, useState } from 'react'
import type { DevSnapshot } from './types'

const POLL_MS = 5000

interface SnapshotState {
  snapshot: DevSnapshot | null
  error: string | null
  loading: boolean
  fetchedAt: number | null
}

export function useSnapshot() {
  const [state, setState] = useState<SnapshotState>({
    snapshot: null,
    error: null,
    loading: true,
    fetchedAt: null,
  })
  const inFlight = useRef(false)

  // Background polls stay silent (no loading state) so the Refresh button
  // doesn't flicker disabled on every tick.
  const load = useCallback(async (background: boolean) => {
    if (inFlight.current) return
    inFlight.current = true
    if (!background) setState((s) => ({ ...s, loading: true }))
    try {
      const res = await fetch('/api/snapshot')
      if (!res.ok) throw new Error(`API returned ${res.status}`)
      const snapshot = (await res.json()) as DevSnapshot
      setState({ snapshot, error: null, loading: false, fetchedAt: Date.now() })
    } catch (err) {
      setState((s) => ({
        ...s,
        loading: false,
        error: err instanceof Error ? err.message : String(err),
      }))
    } finally {
      inFlight.current = false
    }
  }, [])

  const refresh = useCallback(() => load(false), [load])

  useEffect(() => {
    void load(false)
    const id = setInterval(() => {
      if (document.visibilityState === 'visible') void load(true)
    }, POLL_MS)
    const onVisibility = () => {
      if (document.visibilityState === 'visible') void load(true)
    }
    document.addEventListener('visibilitychange', onVisibility)
    return () => {
      clearInterval(id)
      document.removeEventListener('visibilitychange', onVisibility)
    }
  }, [load])

  return { ...state, refresh }
}
