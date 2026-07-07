import { createContext, useCallback, useContext, useEffect, useState } from 'react'

export interface PortdocConfig {
  ignored_services: string[]
}

export interface ConfigState {
  ignored: ReadonlySet<string>
  setIgnored: (serviceId: string, ignored: boolean) => Promise<void>
}

// Same no-prop-drilling pattern as StopContext/InspectContext.
export const ConfigContext = createContext<ConfigState>({
  ignored: new Set(),
  setIgnored: async () => {},
})

export function useConfig() {
  return useContext(ConfigContext)
}

export function useConfigState(): ConfigState {
  const [ignored, setIgnoredSet] = useState<ReadonlySet<string>>(new Set())

  useEffect(() => {
    let cancelled = false
    void (async () => {
      try {
        const res = await fetch('/api/config')
        if (!res.ok) return
        const config = (await res.json()) as PortdocConfig
        if (!cancelled) setIgnoredSet(new Set(config.ignored_services))
      } catch {
        // config is a nicety; a failed load just means nothing is hidden
      }
    })()
    return () => {
      cancelled = true
    }
  }, [])

  const setIgnored = useCallback(async (serviceId: string, nowIgnored: boolean) => {
    const res = await fetch('/api/ignore', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ service_id: serviceId, ignored: nowIgnored }),
    })
    const body = (await res.json()) as PortdocConfig & { error?: string }
    if (!res.ok) throw new Error(body.error ?? `ignore failed (${res.status})`)
    setIgnoredSet(new Set(body.ignored_services))
  }, [])

  return { ignored, setIgnored }
}
