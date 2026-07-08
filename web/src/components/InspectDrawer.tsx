import { useEffect, useState } from 'react'
import { Copy, ExternalLink, EyeOff, FolderOpen, Square, X } from 'lucide-react'
import type { DevSnapshot, DockerHint, ProjectGroup, Service } from '../lib/types'
import type { InspectTarget } from '../lib/inspect'
import { canStop, conflictedIds, displayName, stopBlockedReason } from '../lib/derive'
import { useRequestStop } from '../lib/stop'
import { useConfig } from '../lib/config'
import { Badge } from './ui/badge'
import { Button } from './ui/button'

interface InspectDrawerProps {
  target: InspectTarget
  snapshot: DevSnapshot
  onClose: () => void
}

export function InspectDrawer({ target, snapshot, onClose }: InspectDrawerProps) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && onClose()
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [onClose])

  const projectById = new Map(snapshot.projects.map((p) => [p.id, p]))
  const conflicted = conflictedIds(snapshot)
  const { services, port } = target
  const heading =
    services.length > 1
      ? `Port :${port} - ${services.length} listeners`
      : services.length === 1
        ? `${displayName(services[0])} on :${services[0].port}`
        : `Port :${port}`

  return (
    <div
      className="fixed inset-0 z-40 flex justify-end bg-black/40"
      onClick={(e) => e.target === e.currentTarget && onClose()}
    >
      <aside
        role="dialog"
        aria-modal="true"
        className="flex h-full w-full max-w-md flex-col overflow-y-auto border-l border-border bg-surface shadow-xl"
      >
        <div className="sticky top-0 flex items-center gap-2 border-b border-border bg-surface px-4 py-3">
          <h2 className="text-sm font-semibold">{heading}</h2>
          <button className="ml-auto text-faint hover:text-text" onClick={onClose} title="Close">
            <X className="size-4" />
          </button>
        </div>

        {services.length > 1 && (
          <p className="border-b border-border px-4 py-2 text-xs text-muted">
            Multiple processes hold this port (usually an IPv4/IPv6 split or a worker pool).
            localhost may reach either one - check the bind address.
          </p>
        )}
        {services.length === 0 ? (
          <div className="px-4 py-10 text-center text-sm text-faint">
            Nothing is listening on :{port}. The port is free.
          </div>
        ) : (
          services.map((service) => (
            <ServiceDetail
              key={service.id}
              service={service}
              project={service.project_id ? projectById.get(service.project_id) : undefined}
              conflicted={conflicted.has(service.id)}
              dockerHint={dockerHintFor(service, snapshot.docker_hints)}
            />
          ))
        )}
      </aside>
    </div>
  )
}

// Joined hints match by service id; a port fallback still names the
// container when the join could not resolve an owner.
function dockerHintFor(service: Service, hints: DockerHint[]): DockerHint | undefined {
  return (
    hints.find((h) => h.service_id === service.id) ?? hints.find((h) => h.port === service.port)
  )
}

function ServiceDetail({
  service,
  project,
  conflicted,
  dockerHint,
}: {
  service: Service
  project?: ProjectGroup
  conflicted: boolean
  dockerHint?: DockerHint
}) {
  const requestStop = useRequestStop()
  const stoppable = canStop(service)
  const { ignored, setIgnored } = useConfig()
  const isIgnored = ignored.has(service.id)
  const [ignoreError, setIgnoreError] = useState(false)
  const toggleIgnored = async () => {
    try {
      await setIgnored(service.id, !isIgnored)
    } catch {
      setIgnoreError(true)
      setTimeout(() => setIgnoreError(false), 1600)
    }
  }

  return (
    <div className="border-b border-border px-4 py-3.5 last:border-b-0">
      <div className="mb-3 flex flex-wrap gap-1.5">
        <Badge variant={service.exposure === 'lan' ? 'warn' : 'default'} dot>
          {service.exposure === 'lan' ? 'LAN' : service.exposure}
        </Badge>
        {service.framework && <Badge>{service.framework}</Badge>}
        {isIgnored && <Badge title="Hidden from the dashboard">ignored</Badge>}
        {conflicted && (
          <Badge dot title="Another process is also listening on this port">
            shared port
          </Badge>
        )}
        {service.stale && (
          <Badge variant="warn" dot>
            stale
          </Badge>
        )}
      </div>

      <dl className="space-y-1.5 text-xs">
        <Field label="Port" value={`:${service.port}`} />
        <Field label="PID" value={service.pid?.toString()} />
        <Field label="Process" value={service.process_name} />
        <Field label="Command" value={service.command} />
        <Field label="Path" value={service.cwd} />
        <Field label="User" value={service.user} />
        <Field label="Project" value={project ? `${project.name}  ${project.root}` : undefined} />
        <Field label="Container" value={dockerHint?.container} />
        <Field label="Image" value={dockerHint?.image} />
        <Field label="Compose" value={dockerHint?.compose_project} />
        <Field label="URL" value={service.url} />
        <Field label="Age" value={service.started_age} />
        <Field label="Stale" value={service.stale?.reason} plain />
      </dl>

      <div className="mt-3.5 flex flex-wrap gap-1.5">
        <CopyButton
          label="Open"
          icon={<ExternalLink className="size-3" />}
          disabled={!service.url}
          disabledReason="No local URL detected"
          onClick={() => service.url && window.open(service.url)}
        />
        <CopyButton
          label="Copy URL"
          icon={<Copy className="size-3" />}
          disabled={!service.url}
          disabledReason="No local URL detected"
          copyText={service.url}
        />
        <CopyButton
          label="Copy kill cmd"
          icon={<Copy className="size-3" />}
          disabled={service.pid === undefined}
          disabledReason="No owner pid"
          copyText={service.pid !== undefined ? `kill ${service.pid}` : undefined}
        />
        <RevealButton path={project?.root ?? service.cwd} />
        <Button
          size="sm"
          variant="ghost"
          title={isIgnored ? 'Show this service again' : 'Hide this service from the dashboard'}
          onClick={() => void toggleIgnored()}
        >
          <EyeOff className="size-3" />
          {ignoreError ? 'Failed' : isIgnored ? 'Unignore' : 'Ignore'}
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
      </div>
    </div>
  )
}

function Field({ label, value, plain }: { label: string; value?: string; plain?: boolean }) {
  if (!value) return null
  return (
    <div className="flex gap-2">
      <dt className="w-16 flex-none text-faint">{label}</dt>
      <dd className={`min-w-0 break-words ${plain ? 'text-muted' : 'font-mono text-text'}`}>
        {value}
      </dd>
    </div>
  )
}

function CopyButton({
  label,
  icon,
  disabled,
  disabledReason,
  copyText,
  onClick,
}: {
  label: string
  icon: React.ReactNode
  disabled: boolean
  disabledReason: string
  copyText?: string
  onClick?: () => void
}) {
  const [copied, setCopied] = useState(false)
  const handle = async () => {
    if (onClick) return onClick()
    if (copyText && navigator.clipboard) {
      await navigator.clipboard.writeText(copyText)
      setCopied(true)
      setTimeout(() => setCopied(false), 1200)
    }
  }
  return (
    <Button
      size="sm"
      variant="ghost"
      disabled={disabled}
      title={disabled ? disabledReason : undefined}
      onClick={() => void handle()}
    >
      {icon}
      {copied ? 'Copied' : label}
    </Button>
  )
}

function RevealButton({ path }: { path?: string }) {
  const [error, setError] = useState(false)
  const reveal = async () => {
    setError(false)
    try {
      const res = await fetch('/api/reveal', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ path }),
      })
      if (!res.ok) throw new Error()
    } catch {
      setError(true)
      setTimeout(() => setError(false), 1600)
    }
  }
  return (
    <Button
      size="sm"
      variant="ghost"
      disabled={!path}
      title={path ? undefined : 'No folder detected'}
      onClick={() => void reveal()}
    >
      <FolderOpen className="size-3" />
      {error ? 'Failed' : 'Reveal folder'}
    </Button>
  )
}
