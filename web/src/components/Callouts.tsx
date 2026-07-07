import { Clock } from 'lucide-react'
import type { DevSnapshot } from '../lib/types'
import { canStop, staleServices, stopBlockedReason } from '../lib/derive'
import { useRequestStop } from '../lib/stop'
import { Button } from './ui/button'

export function Callouts({ snapshot }: { snapshot: DevSnapshot }) {
  const requestStop = useRequestStop()
  return (
    <>
      {staleServices(snapshot).map((service) => (
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
            <Button size="sm" disabled title="Ignore lands with feature 13">
              Ignore
            </Button>
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
