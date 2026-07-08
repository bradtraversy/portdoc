import type { DevSnapshot, Service } from './types'

export function conflictedIds(snapshot: DevSnapshot): Set<string> {
  return new Set(snapshot.conflicts.flatMap((c) => c.service_ids))
}

// The dashboard's own server row: never offered a Stop action
export function isSelf(service: Service): boolean {
  return service.process_name === 'portdoc'
}

// The primary label for a service: the framework is what a developer thinks
// of it as ("Astro"), so it wins over the raw executable ("node"). The
// process name stays in the sub-line for the honest detail.
export function displayName(service: Service): string {
  return service.framework ?? service.process_name ?? 'unknown'
}

// Stoppable only when we own a pid to signal, it isn't PortDoc, and it
// isn't a Docker row (killing the proxy would not stop the container).
export function canStop(service: Service): boolean {
  return service.pid !== undefined && !isSelf(service) && service.exposure !== 'docker'
}

export function stopBlockedReason(service: Service): string {
  if (isSelf(service)) return 'Protected - PortDoc will not stop itself'
  if (service.exposure === 'docker') return 'Use docker stop - killing the proxy does not stop the container'
  return 'No owner process to stop'
}

export function lanServices(snapshot: DevSnapshot): Service[] {
  return snapshot.services.filter((s) => s.exposure === 'lan')
}

export function dockerServices(snapshot: DevSnapshot): Service[] {
  return snapshot.services.filter((s) => s.exposure === 'docker')
}

export function ungroupedServices(snapshot: DevSnapshot): Service[] {
  return snapshot.services.filter((s) => !s.project_id && s.exposure !== 'docker')
}

export function staleServices(snapshot: DevSnapshot): Service[] {
  return snapshot.services.filter((s) => s.stale)
}

// Hiding ignored services is purely a UI concern; the snapshot stays complete.
export function visibleServices(snapshot: DevSnapshot, ignored: ReadonlySet<string>): Service[] {
  return snapshot.services.filter((s) => !ignored.has(s.id))
}

export function servicesOnPort(snapshot: DevSnapshot, port: number): Service[] {
  return snapshot.services.filter((s) => s.port === port)
}

export function commonRoot(snapshot: DevSnapshot): string | null {
  const parents = snapshot.projects.map((p) => p.root.slice(0, p.root.lastIndexOf('/')))
  if (parents.length === 0) return null
  return parents.every((p) => p === parents[0]) ? parents[0] : null
}
