import { useCallback, useEffect, useState } from 'react'
import type { DevSnapshot } from './types'

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

  const refresh = useCallback(async () => {
    setState((s) => ({ ...s, loading: true }))
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
    }
  }, [])

  useEffect(() => {
    void refresh()
  }, [refresh])

  return { ...state, refresh }
}
