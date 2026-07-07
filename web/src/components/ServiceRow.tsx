import { EllipsisVertical, ExternalLink, Shield, Square } from 'lucide-react'
import type { DockerHint, Service } from '../lib/types'
import { canStop, displayName, isSelf, stopBlockedReason } from '../lib/derive'
import { useRequestStop } from '../lib/stop'
import { useInspect } from '../lib/inspect'
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
  // when the headline is the framework, keep the raw process name here so
  // the honest executable ("node") is never lost
  const proc = service.framework ? service.process_name : undefined
  const parts = [
    proc,
    service.pid !== undefined ? `PID ${service.pid}` : undefined,
    service.command,
  ].filter(Boolean)
  return parts.length ? parts.join(' · ') : undefined
}

export function ServiceRow({ service, conflicted, dockerHint }: ServiceRowProps) {
  const self = isSelf(service)
  const sub = subLine(service, dockerHint)
  const requestStop = useRequestStop()
  const inspect = useInspect()
  const stoppable = canStop(service)

  return (
    <div
      className="group grid cursor-pointer grid-cols-[64px_1fr_auto] items-center gap-3.5 border-b border-border px-4 py-2 last:border-b-0 hover:bg-surface-2"
      onClick={() => inspect({ services: [service] })}
    >
      <span className="text-right font-mono text-sm font-semibold tabular-nums">
        :{service.port}
      </span>
      <div className="min-w-0">
        <div className="flex flex-wrap items-baseline gap-2">
          <span className="text-sm font-semibold">{displayName(service)}</span>
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
      <div className="flex items-center gap-1.5" onClick={(e) => e.stopPropagation()}>
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
          <Badge dot title="Another process is also listening on this port">
            shared port
          </Badge>
        )}
        {service.exposure === 'lan' && (
          <Badge variant="warn" dot>
            LAN visible
          </Badge>
        )}
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
