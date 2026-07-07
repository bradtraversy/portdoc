import type { Service } from './types'

export type FilterChip =
  | 'framework'
  | 'runtime'
  | 'api'
  | 'database'
  | 'docker'
  | 'unknown'
  | 'lan'
  | 'stale'
  | 'conflict'

export const CHIPS: { id: FilterChip; label: string }[] = [
  { id: 'framework', label: 'Framework' },
  { id: 'runtime', label: 'Runtime' },
  { id: 'api', label: 'API' },
  { id: 'database', label: 'Database' },
  { id: 'docker', label: 'Docker' },
  { id: 'unknown', label: 'Unknown' },
  { id: 'lan', label: 'LAN visible' },
  { id: 'stale', label: 'Stale' },
  { id: 'conflict', label: 'Conflict' },
]

// Mirrors the Rust label vocabulary (src/label.rs), not shared with it.
const FRAMEWORKS = new Set(['Next.js', 'Vite', 'Astro', 'Remix', 'Nuxt', 'React scripts'])
const RUNTIMES = new Set(['Bun', 'Convex', 'Prisma Studio'])
const APIS = new Set(['Express'])
const DATABASES = new Set(['Postgres', 'Redis'])

export function matchesChip(chip: FilterChip, service: Service, conflicted: boolean): boolean {
  switch (chip) {
    case 'framework':
      return !!service.framework && FRAMEWORKS.has(service.framework)
    case 'runtime':
      return !!service.framework && RUNTIMES.has(service.framework)
    case 'api':
      return !!service.framework && APIS.has(service.framework)
    case 'database':
      return !!service.framework && DATABASES.has(service.framework)
    case 'docker':
      return service.exposure === 'docker'
    case 'unknown':
      return service.pid === undefined
    case 'lan':
      return service.exposure === 'lan'
    case 'stale':
      return !!service.stale
    case 'conflict':
      return conflicted
  }
}

export function matchesQuery(service: Service, query: string, projectName?: string): boolean {
  const q = query.trim().toLowerCase()
  if (!q) return true
  const haystack = [
    `:${service.port}`,
    service.process_name,
    service.command,
    service.cwd,
    service.user,
    service.framework,
    projectName,
  ]
    .filter(Boolean)
    .join(' ')
    .toLowerCase()
  return haystack.includes(q)
}
