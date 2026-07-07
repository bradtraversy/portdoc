import { EllipsisVertical, ExternalLink, Shield, Square } from 'lucide-react'
import type { DockerHint, Service } from '../lib/types'
import { canStop, isSelf, stopBlockedReason } from '../lib/derive'
import { useRequestStop } from '../lib/stop'
import { Badge } from './ui/badge'
import { Button } from './ui/button'

interface ServiceRowProps {
  service: Service
  conflicted: boolean
  dockerHint?: DockerHint
}

function subLine(service: Service, dockerHint?: DockerHint): string | undefined {
  if (dockerHint) {
    return `container ${dockerHint.container}${dockerHint.image ? ` · ${dockerHint.image}` : ''}`
  }
  if (service.pid !== undefined) {
    return `PID ${service.pid}${service.command ? ` · ${service.command}` : ''}`
  }
  return service.command
}

export function ServiceRow({ service, conflicted, dockerHint }: ServiceRowProps) {
  const self = isSelf(service)
  const sub = subLine(service, dockerHint)
  const requestStop = useRequestStop()
  const stoppable = canStop(service)

  return (
    <div className="group grid grid-cols-[64px_1fr_auto] items-center gap-3.5 border-b border-border px-4 py-2 last:border-b-0 hover:bg-surface-2">
      <span className="text-right font-mono text-sm font-semibold tabular-nums">
        :{service.port}
      </span>
      <div className="min-w-0">
        <div className="flex flex-wrap items-baseline gap-2">
          <span className="text-sm font-semibold">{service.process_name ?? 'unknown'}</span>
          {service.url && (
            <a
              className="font-mono text-xs text-accent hover:underline"
              href={service.url}
              target="_blank"
              rel="noreferrer"
            >
              {service.url}
            </a>
          )}
          {self && <span className="text-xs text-faint">this app</span>}
        </div>
        {sub && <span className="block truncate font-mono text-xs text-faint">{sub}</span>}
      </div>
      <div className="flex items-center gap-1.5">
        <span className="hidden gap-1 group-hover:inline-flex">
          <Button
            size="sm"
            variant="ghost"
            disabled={!service.url}
            title={service.url ? undefined : 'No local URL detected'}
            onClick={() => service.url && window.open(service.url)}
          >
            <ExternalLink className="size-3" />
            Open
          </Button>
          <Button
            size="sm"
            variant="ghost"
            disabled={!stoppable}
            title={stoppable ? undefined : stopBlockedReason(service)}
            onClick={() => requestStop(service)}
          >
            <Square className="size-3" />
            Stop
          </Button>
        </span>
        {self && (
          <Badge title="PortDoc will not stop itself">
            <Shield className="size-[11px]" />
            protected
          </Badge>
        )}
        {conflicted && (
          <Badge variant="danger" dot>
            conflict
          </Badge>
        )}
        {service.exposure === 'lan' && (
          <Badge variant="warn" dot>
            LAN visible
          </Badge>
        )}
        {service.framework && <Badge>{service.framework}</Badge>}
        {service.stale ? (
          <Badge variant="warn" dot title={service.stale.reason}>
            stale{service.started_age ? ` · ${service.started_age}` : ''}
          </Badge>
        ) : (
          service.started_age && (
            <Badge variant="ok" dot>
              running {service.started_age}
            </Badge>
          )
        )}
        <Button size="sm" variant="ghost" title="More actions">
          <EllipsisVertical className="size-3" />
        </Button>
      </div>
    </div>
  )
}
