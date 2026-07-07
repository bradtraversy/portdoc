import { useState } from 'react'
import { ArrowRight, Search } from 'lucide-react'
import type { DevSnapshot } from '../lib/types'
import { servicesOnPort } from '../lib/derive'
import { useInspect } from '../lib/inspect'

export function PortLookup({ snapshot }: { snapshot: DevSnapshot }) {
  const [value, setValue] = useState('')
  const inspect = useInspect()

  const lookup = () => {
    const port = Number(value.trim().replace(/^:/, ''))
    if (!Number.isInteger(port) || port < 1 || port > 65535) return
    inspect({ port, services: servicesOnPort(snapshot, port) })
  }

  return (
    <section className="flex items-center gap-2 rounded-lg border border-border bg-surface px-4 py-3">
      <label className="text-xs font-semibold uppercase tracking-wider text-muted">
        Look up a port
      </label>
      <form
        className="relative"
        onSubmit={(e) => {
          e.preventDefault()
          lookup()
        }}
      >
        <Search className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-faint" />
        <input
          inputMode="numeric"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          placeholder="e.g. 3000"
          className="w-32 rounded-md border border-border bg-surface-2 py-1 pl-8 pr-2.5 font-mono text-sm text-text placeholder:text-faint focus:border-accent focus:outline-none"
        />
      </form>
      <button
        type="button"
        onClick={lookup}
        title="Inspect this port"
        className="inline-flex items-center gap-1 rounded-md border border-border-strong bg-surface-2 px-2 py-1 text-xs text-muted hover:text-text"
      >
        Inspect
        <ArrowRight className="size-3" />
      </button>
      <span className="text-xs text-faint">see exactly what owns a port, and act on it</span>
    </section>
  )
}
