// TypeScript mirror of the locked DevSnapshot contract (feature 1, archived in
// blueprint/history/features/01-mock-snapshot-contract.md). Optional fields are
// omitted by the API when absent, never null.

export type Exposure = 'local' | 'lan' | 'docker' | 'unknown'

export interface StaleHint {
  reason: string
}

export interface Service {
  id: string
  port: number
  pid?: number
  process_name?: string
  command?: string
  cwd?: string
  user?: string
  project_id?: string
  framework?: string
  exposure: Exposure
  url?: string
  started_age?: string
  stale?: StaleHint
}

export interface ProjectGroup {
  id: string
  name: string
  root: string
  package_manager?: string
  git_branch?: string
  service_ids: string[]
}

export interface Conflict {
  port: number
  service_ids: string[]
  hint: string
}

export interface DockerHint {
  port: number
  container: string
  service_id?: string
  image?: string
  compose_project?: string
}

export interface DevSnapshot {
  generated_at: number
  services: Service[]
  projects: ProjectGroup[]
  conflicts: Conflict[]
  docker_hints: DockerHint[]
}
