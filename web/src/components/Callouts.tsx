import { Clock, TriangleAlert } from 'lucide-react'
import type { DevSnapshot } from '../lib/types'
import { canStop, staleUnconflicted, stopBlockedReason } from '../lib/derive'
import { useRequestStop } from '../lib/stop'
import type { TabId } from './TabBar'
import { Button } from './ui/button'

interface CalloutsProps {
  snapshot: DevSnapshot
  onNavigate: (tab: TabId) => void
}

export function Callouts({ snapshot, onNavigate }: CalloutsProps) {
  const requestStop = useRequestStop()
  return (
    <>
      {snapshot.conflicts.map((conflict) => (
        <div
          key={conflict.port}
          className="flex items-center gap-3 rounded-lg border border-danger/30 bg-danger/10 px-3.5 py-2.5 text-sm"
        >
          <TriangleAlert className="size-3.5 flex-none text-danger" />
          <span>
            <strong className="font-semibold">Port {conflict.port} conflict.</strong>{' '}
            {conflict.hint}
          </span>
          <span className="ml-auto flex flex-none gap-2">
            <Button size="sm" disabled title="Inspect lands with feature 13">
              Inspect
            </Button>
            <Button size="sm" variant="accent" onClick={() => onNavigate('conflicts')}>
              Resolve
            </Button>
          </span>
        </div>
      ))}
      {staleUnconflicted(snapshot).map((service) => (
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
