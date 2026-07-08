import type { DevSnapshot } from '../lib/types'
import { servicesOnPort } from '../lib/derive'
import { useInspect } from '../lib/inspect'

export function PortChip({ port, snapshot }: { port: number; snapshot: DevSnapshot }) {
  const inspect = useInspect()
  return (
    <button
      className="cursor-pointer rounded border border-border bg-surface-2 px-1.5 py-0.5 font-mono text-xs tabular-nums text-muted hover:text-text"
      title={`Inspect port ${port}`}
      onClick={() => inspect({ port, services: servicesOnPort(snapshot, port) })}
    >
      :{port}
    </button>
  )
}
