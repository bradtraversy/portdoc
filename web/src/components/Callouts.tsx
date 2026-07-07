import { useState } from 'react'
import { Clock } from 'lucide-react'
import type { DevSnapshot } from '../lib/types'
import { canStop, staleServices, stopBlockedReason } from '../lib/derive'
import { useRequestStop } from '../lib/stop'
import { useConfig } from '../lib/config'
import { Button } from './ui/button'

export function Callouts({ snapshot }: { snapshot: DevSnapshot }) {
  const requestStop = useRequestStop()
  const { ignored } = useConfig()
  return (
    <>
      {staleServices(snapshot)
        .filter((service) => !ignored.has(service.id))
        .map((service) => (
        <div
          key={service.id}
          className="flex items-center gap-3 rounded-lg border border-warn/30 bg-warn/10 px-3.5 py-2.5 text-sm"
        >
          <Clock className="size-3.5 flex-none text-warn" />
          <span>
            <strong className="font-semibold">Possibly stale:</strong>{' '}
            {service.process_name ?? 'unknown process'} on :{service.port},{' '}
            {service.stale?.reason}
          </span>
          <span className="ml-auto flex flex-none gap-2">
            <IgnoreButton serviceId={service.id} />
            <Button
              size="sm"
              disabled={!canStop(service)}
              title={canStop(service) ? undefined : stopBlockedReason(service)}
              onClick={() => requestStop(service)}
            >
              Stop safely
            </Button>
          </span>
        </div>
      ))}
    </>
  )
}

function IgnoreButton({ serviceId }: { serviceId: string }) {
  const { setIgnored } = useConfig()
  const [error, setError] = useState(false)
  const handle = async () => {
    try {
      await setIgnored(serviceId, true)
    } catch {
      setError(true)
      setTimeout(() => setError(false), 1600)
    }
  }
  return (
    <Button size="sm" title="Hide this service from the dashboard" onClick={() => void handle()}>
      {error ? 'Failed' : 'Ignore'}
    </Button>
  )
}
