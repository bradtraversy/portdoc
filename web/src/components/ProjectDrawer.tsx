import { useEffect } from 'react'
import { GitBranch, X } from 'lucide-react'
import type { DevSnapshot, ProjectGroup, Service } from '../lib/types'
import { displayName } from '../lib/derive'
import { useInspect } from '../lib/inspect'
import { Badge } from './ui/badge'
import { ProjectActions } from './ProjectActions'

interface ProjectDrawerProps {
  project: ProjectGroup
  snapshot: DevSnapshot
  onClose: () => void
}

export function ProjectDrawer({ project, snapshot, onClose }: ProjectDrawerProps) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && onClose()
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [onClose])

  const byId = new Map(snapshot.services.map((s) => [s.id, s]))
  const services = project.service_ids
    .map((id) => byId.get(id))
    .filter((s): s is Service => Boolean(s))

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
          <h2 className="text-sm font-semibold">{project.name}</h2>
          <span className="font-mono text-xs text-faint">{project.root}</span>
          <button className="ml-auto text-faint hover:text-text" onClick={onClose} title="Close">
            <X className="size-4" />
          </button>
        </div>

        <div className="border-b border-border px-4 py-3.5">
          <div className="flex flex-wrap gap-1.5">
            {project.git_branch && (
              <Badge>
                <GitBranch className="size-[11px]" />
                {project.git_branch}
              </Badge>
            )}
            {project.dirty !== undefined &&
              (project.dirty ? (
                <Badge variant="warn" dot title="git status reports uncommitted tracked changes">
                  uncommitted changes
                </Badge>
              ) : (
                <Badge variant="ok" dot>
                  clean
                </Badge>
              ))}
            {project.package_manager && <Badge>{project.package_manager}</Badge>}
            {project.node_version && <Badge title="from .nvmrc / engines">node {project.node_version}</Badge>}
          </div>
          {project.description && (
            <p className="mt-2.5 text-xs text-muted">{project.description}</p>
          )}
          <dl className="mt-2.5 space-y-1.5 text-xs">
            <Field label="Last commit" value={project.last_commit_age && `${project.last_commit_age} ago`} plain />
            <Field label="Workspaces" value={project.workspaces?.join('  ')} />
          </dl>
          <div className="mt-2.5">
            <ProjectActions project={project} services={services} />
          </div>
        </div>

        {project.scripts && project.scripts.length > 0 && (
          <Section title="Scripts">
            <dl className="space-y-1.5 text-xs">
              {project.scripts.map((s) => (
                <Field key={s.name} label={s.name} value={s.command} />
              ))}
            </dl>
          </Section>
        )}

        {project.key_deps && project.key_deps.length > 0 && (
          <Section title="Stack">
            <div className="flex flex-wrap gap-1.5">
              {project.key_deps.map((d) => (
                <Badge key={d.name}>
                  {d.name}
                  {d.version && <span className="text-faint">{d.version}</span>}
                </Badge>
              ))}
            </div>
          </Section>
        )}

        <Section title={`Services (${services.length})`}>
          <div className="-mx-4">
            {services.map((service) => (
              <ServiceLine key={service.id} service={service} />
            ))}
          </div>
        </Section>
      </aside>
    </div>
  )
}

function ServiceLine({ service }: { service: Service }) {
  const inspect = useInspect()
  return (
    <button
      className="grid w-full cursor-pointer grid-cols-[56px_1fr] items-baseline gap-3 px-4 py-1.5 text-left hover:bg-surface-2"
      onClick={() => inspect({ services: [service] })}
    >
      <span className="text-right font-mono text-xs font-semibold tabular-nums">
        :{service.port}
      </span>
      <span className="truncate text-xs">
        {displayName(service)}
        {service.started_age && <span className="text-faint"> · running {service.started_age}</span>}
      </span>
    </button>
  )
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="border-b border-border px-4 py-3.5 last:border-b-0">
      <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-muted">{title}</h3>
      {children}
    </div>
  )
}

function Field({ label, value, plain }: { label: string; value?: string; plain?: boolean }) {
  if (!value) return null
  return (
    <div className="flex gap-2">
      <dt className="w-20 flex-none text-faint">{label}</dt>
      <dd className={`min-w-0 break-words ${plain ? 'text-muted' : 'font-mono text-text'}`}>
        {value}
      </dd>
    </div>
  )
}
