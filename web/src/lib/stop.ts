import { createContext, useContext } from 'react'
import type { Service } from './types'

// Lets any row open the single App-level stop dialog without prop drilling.
export const StopContext = createContext<(service: Service) => void>(() => {})

export function useRequestStop() {
  return useContext(StopContext)
}

export interface StopResult {
  outcome: 'released' | 'still_listening'
}

export async function postStop(service: Service, force: boolean): Promise<StopResult> {
  const res = await fetch('/api/stop', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ service_id: service.id, pid: service.pid, force }),
  })
  const body = (await res.json()) as StopResult & { error?: string }
  if (!res.ok) throw new Error(body.error ?? `stop failed (${res.status})`)
  return body
}
