import type { DevSnapshot, Service } from './types'

export function conflictedIds(snapshot: DevSnapshot): Set<string> {
  return new Set(snapshot.conflicts.flatMap((c) => c.service_ids))
}

// The dashboard's own server row: never offered a Stop action
export function isSelf(service: Service): boolean {
  return service.process_name === 'portdoc'
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

// Stale services already covered by a conflict callout don't get a second one
export function staleUnconflicted(snapshot: DevSnapshot): Service[] {
  const inConflict = conflictedIds(snapshot)
  return snapshot.services.filter((s) => s.stale && !inConflict.has(s.id))
}

export function commonRoot(snapshot: DevSnapshot): string | null {
  const parents = snapshot.projects.map((p) => p.root.slice(0, p.root.lastIndexOf('/')))
  if (parents.length === 0) return null
  return parents.every((p) => p === parents[0]) ? parents[0] : null
}
