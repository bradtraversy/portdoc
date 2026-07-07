import { TriangleAlert } from 'lucide-react'
import type { DevSnapshot, Service } from '../lib/types'
import { ServiceRow } from './ServiceRow'

export function ConflictsView({ snapshot }: { snapshot: DevSnapshot }) {
  const byId = new Map(snapshot.services.map((s) => [s.id, s]))

  if (snapshot.conflicts.length === 0) {
    return (
      <section className="rounded-lg border border-dashed border-border-strong bg-surface px-4 py-10 text-center">
        <h2 className="text-base font-semibold">No port conflicts detected</h2>
        <p className="mt-1 text-sm text-faint">
          Conflicts appear when two processes share a port or a dev server gets bumped off its
          default.
        </p>
      </section>
    )
  }

  return (
    <>
      {snapshot.conflicts.map((conflict) => {
        const contenders = conflict.service_ids
          .map((id) => byId.get(id))
          .filter((s): s is Service => Boolean(s))
        return (
          <section
            key={conflict.port}
            className="overflow-hidden rounded-lg border border-border bg-surface"
          >
            <div className="flex items-start gap-2.5 border-b border-border px-4 py-3 text-sm">
              <TriangleAlert className="mt-0.5 size-3.5 flex-none text-danger" />
              <div>
                <strong className="font-semibold">Port {conflict.port} conflict.</strong>{' '}
                <span className="text-muted">{conflict.hint}</span>
              </div>
            </div>
            {contenders.map((service) => (
              <ServiceRow key={service.id} service={service} conflicted />
            ))}
          </section>
        )
      })}
    </>
  )
}
