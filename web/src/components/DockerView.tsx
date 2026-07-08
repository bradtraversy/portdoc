import { Box, Layers } from 'lucide-react'
import type { DevSnapshot, DockerHint } from '../lib/types'
import { Badge } from './ui/badge'
import { PortChip } from './PortChip'

interface Container {
  name: string
  image?: string
  hints: DockerHint[]
}

interface ComposeGroup {
  project?: string
  containers: Container[]
}

// Hints are per published port; fold them back into containers, then group
// containers by Compose project (named projects first, standalone last).
function composeGroups(hints: DockerHint[]): ComposeGroup[] {
  const containers = new Map<string, Container & { project?: string }>()
  for (const hint of hints) {
    const existing = containers.get(hint.container)
    if (existing) existing.hints.push(hint)
    else
      containers.set(hint.container, {
        name: hint.container,
        image: hint.image,
        project: hint.compose_project,
        hints: [hint],
      })
  }

  const groups = new Map<string | undefined, ComposeGroup>()
  for (const container of containers.values()) {
    const group = groups.get(container.project) ?? { project: container.project, containers: [] }
    group.containers.push(container)
    groups.set(container.project, group)
  }
  return [...groups.values()].sort((a, b) =>
    a.project === undefined ? 1 : b.project === undefined ? -1 : a.project.localeCompare(b.project),
  )
}

export function DockerView({ snapshot }: { snapshot: DevSnapshot }) {
  const groups = composeGroups(snapshot.docker_hints)

  if (groups.length === 0) {
    return (
      <div className="rounded-lg border border-border bg-surface px-6 py-14 text-center">
        <Box className="mx-auto mb-3 size-6 text-faint" />
        <h2 className="text-sm font-semibold">No Docker containers publishing TCP ports</h2>
        <p className="mx-auto mt-1.5 max-w-md text-xs text-muted">
          PortDoc asks the docker CLI directly, so this means nothing is published right now - or
          Docker itself is not installed, not running, or not on the PATH. A snapshot cannot tell
          those apart.
        </p>
      </div>
    )
  }

  return (
    <>
      <h2 className="text-xs font-semibold uppercase tracking-wider text-muted">Docker</h2>
      {groups.map((group) => (
        <section
          key={group.project ?? '(standalone)'}
          className="overflow-hidden rounded-lg border border-border bg-surface"
        >
          <div className="flex items-center gap-2.5 border-b border-border px-4 py-2.5">
            {group.project ? (
              <>
                <Layers className="size-[13px] text-faint" />
                <span className="font-semibold">{group.project}</span>
                <span className="font-mono text-xs text-faint">compose project</span>
              </>
            ) : (
              <span className="font-semibold">Standalone containers</span>
            )}
            <div className="ml-auto">
              <Badge variant="info" dot>
                <Box className="size-[11px]" />
                {group.containers.length} container{group.containers.length === 1 ? '' : 's'}
              </Badge>
            </div>
          </div>
          {group.containers.map((container) => (
            <ContainerRow key={container.name} container={container} snapshot={snapshot} />
          ))}
        </section>
      ))}
    </>
  )
}

function ContainerRow({ container, snapshot }: { container: Container; snapshot: DevSnapshot }) {
  return (
    <div className="grid grid-cols-[1fr_auto] items-center gap-3.5 border-b border-border px-4 py-2 last:border-b-0">
      <div className="min-w-0">
        <span className="text-sm font-semibold">{container.name}</span>
        {container.image && (
          <span className="block truncate font-mono text-xs text-faint">{container.image}</span>
        )}
      </div>
      <div className="flex items-center gap-1.5">
        {container.hints.map((hint) => (
          <PortChip key={hint.port} port={hint.port} snapshot={snapshot} />
        ))}
      </div>
    </div>
  )
}
