import { Box, GitBranch } from 'lucide-react'
import type { DevSnapshot, Service } from '../lib/types'
import { conflictedIds, dockerServices, ungroupedServices } from '../lib/derive'
import { useConfig } from '../lib/config'
import { Badge } from './ui/badge'
import { ServiceRow } from './ServiceRow'

function uniqueFrameworks(services: Service[]): string[] {
  return [...new Set(services.map((s) => s.framework).filter((f): f is string => Boolean(f)))]
}

export function ProjectGroups({ snapshot }: { snapshot: DevSnapshot }) {
  const conflicted = conflictedIds(snapshot)
  const { ignored } = useConfig()
  const byId = new Map(snapshot.services.map((s) => [s.id, s]))
  const hintFor = new Map(
    snapshot.docker_hints.filter((h) => h.service_id).map((h) => [h.service_id, h]),
  )
  const docker = dockerServices(snapshot).filter((s) => !ignored.has(s.id))
  const ungrouped = ungroupedServices(snapshot).filter((s) => !ignored.has(s.id))

  const row = (service: Service) => (
    <ServiceRow
      key={service.id}
      service={service}
      conflicted={conflicted.has(service.id)}
      dockerHint={hintFor.get(service.id)}
    />
  )

  return (
    <>
      <h2 className="text-xs font-semibold uppercase tracking-wider text-muted">Projects</h2>
      {snapshot.projects.map((project) => {
        const services = project.service_ids
          .map((id) => byId.get(id))
          .filter((s): s is Service => Boolean(s))
          .filter((s) => !ignored.has(s.id))
        if (services.length === 0) return null
        return (
          <section key={project.id} className="overflow-hidden rounded-lg border border-border bg-surface">
            <div className="flex items-center gap-2.5 border-b border-border px-4 py-2.5">
              <span className="font-semibold">{project.name}</span>
              <span className="font-mono text-xs text-faint">{project.root}</span>
              <div className="ml-auto flex items-center gap-1.5">
                {project.git_branch && (
                  <span className="inline-flex items-center gap-1 font-mono text-xs text-muted">
                    <GitBranch className="size-[11px] text-faint" />
                    {project.git_branch}
                  </span>
                )}
                {project.package_manager && <Badge>{project.package_manager}</Badge>}
                {uniqueFrameworks(services).map((f) => (
                  <Badge key={f}>{f}</Badge>
                ))}
              </div>
            </div>
            {services.map(row)}
          </section>
        )
      })}
      {docker.length > 0 && (
        <section className="overflow-hidden rounded-lg border border-border bg-surface">
          <div className="flex items-center gap-2.5 border-b border-border px-4 py-2.5">
            <span className="font-semibold">Docker</span>
            <span className="font-mono text-xs text-faint">
              {docker.length} container{docker.length === 1 ? '' : 's'} publishing ports
            </span>
            <div className="ml-auto">
              <Badge variant="info" dot>
                <Box className="size-[11px]" />
                docker
              </Badge>
            </div>
          </div>
          {docker.map(row)}
        </section>
      )}
      {ungrouped.length > 0 && (
        <section className="overflow-hidden rounded-lg border border-border bg-surface">
          <div className="flex items-center gap-2.5 border-b border-border px-4 py-2.5">
            <span className="font-semibold">Ungrouped</span>
            <span className="font-mono text-xs text-faint">no project detected</span>
          </div>
          {ungrouped.map(row)}
        </section>
      )}
    </>
  )
}
