import { useEffect, useState } from 'react'
import { TriangleAlert, X } from 'lucide-react'
import type { Service } from '../lib/types'
import { displayName } from '../lib/derive'
import { postStop } from '../lib/stop'
import { Button } from './ui/button'
import { Badge } from './ui/badge'

interface StopAllDialogProps {
  projectName: string
  services: Service[]
  onClose: () => void
}

type Phase = 'confirm' | 'working' | 'escalate' | 'done'

interface Outcome {
  state: 'pending' | 'stopping' | 'released' | 'still_listening' | 'error'
  message?: string
}

/// Batch stop on the feature 12 contract: the full list is visible before
/// anything is signaled, stops are graceful, and force kill only exists for
/// survivors behind a second explicit confirmation.
export function StopAllDialog({ projectName, services, onClose }: StopAllDialogProps) {
  const stoppable = services.filter((s) => s.pid !== undefined)
  const unstoppable = services.filter((s) => s.pid === undefined)
  const [phase, setPhase] = useState<Phase>('confirm')
  const [outcomes, setOutcomes] = useState<Record<string, Outcome>>({})

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && phase !== 'working') onClose()
    }
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [phase, onClose])

  const mark = (id: string, outcome: Outcome) =>
    setOutcomes((prev) => ({ ...prev, [id]: outcome }))

  const run = async (targets: Service[], force: boolean) => {
    setPhase('working')
    const results: Record<string, Outcome> = {}
    for (const service of targets) {
      mark(service.id, { state: 'stopping' })
      try {
        const { outcome } = await postStop(service, force)
        results[service.id] = { state: outcome }
      } catch (err) {
        results[service.id] = {
          state: 'error',
          message: err instanceof Error ? err.message : String(err),
        }
      }
      mark(service.id, results[service.id])
    }
    const survivors = targets.filter((s) => results[s.id]?.state === 'still_listening')
    setPhase(!force && survivors.length > 0 ? 'escalate' : 'done')
  }

  const survivors = stoppable.filter((s) => outcomes[s.id]?.state === 'still_listening')

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      onClick={(e) => e.target === e.currentTarget && phase !== 'working' && onClose()}
    >
      <div
        role="dialog"
        aria-modal="true"
        className="w-full max-w-lg rounded-lg border border-border bg-surface shadow-xl"
      >
        <div className="flex items-center gap-2 border-b border-border px-4 py-3">
          <h2 className="text-sm font-semibold">
            {phase === 'escalate' ? 'Force kill survivors?' : `Stop all in ${projectName}?`}
          </h2>
          {phase !== 'working' && (
            <button className="ml-auto text-faint hover:text-text" onClick={onClose} title="Close">
              <X className="size-4" />
            </button>
          )}
        </div>

        <div className="space-y-3 px-4 py-3.5 text-sm">
          {stoppable.length === 0 ? (
            <p className="text-muted">
              Nothing here can be stopped - no service in this project has a readable owner
              PID. The Advanced tab explains why owners are unreadable.
            </p>
          ) : (
            <div className="max-h-72 space-y-1 overflow-y-auto rounded-md border border-border bg-surface-2 px-3 py-2 text-xs">
              {stoppable.map((service) => (
                <ServiceLine key={service.id} service={service} outcome={outcomes[service.id]} />
              ))}
              {unstoppable.map((service) => (
                <div key={service.id} className="flex items-baseline gap-2 opacity-60">
                  <span className="w-14 flex-none text-right font-mono font-semibold">
                    :{service.port}
                  </span>
                  <span className="truncate">{displayName(service)}</span>
                  <span className="ml-auto flex-none text-faint">no owner pid - skipped</span>
                </div>
              ))}
            </div>
          )}

          {phase === 'confirm' && stoppable.length > 0 && (
            <p className="text-muted">
              Sends each service a graceful stop (SIGTERM), then verifies its port releases.
            </p>
          )}
          {phase === 'working' && <p className="text-muted">Stopping and verifying ports…</p>}
          {phase === 'escalate' && (
            <p className="flex items-start gap-2 text-warn">
              <TriangleAlert className="mt-0.5 size-4 flex-none" />
              {survivors.length} service{survivors.length === 1 ? '' : 's'} ignored the graceful
              stop and {survivors.length === 1 ? 'is' : 'are'} still listening. Force kill
              (SIGKILL) ends {survivors.length === 1 ? 'it' : 'them'} immediately without
              cleanup.
            </p>
          )}
          {phase === 'done' && <Summary outcomes={outcomes} stoppable={stoppable} />}
        </div>

        <div className="flex justify-end gap-2 border-t border-border px-4 py-3">
          {phase === 'confirm' &&
            (stoppable.length > 0 ? (
              <>
                <Button variant="ghost" onClick={onClose}>
                  Cancel
                </Button>
                <Button variant="accent" onClick={() => void run(stoppable, false)}>
                  Stop {stoppable.length} service{stoppable.length === 1 ? '' : 's'}
                </Button>
              </>
            ) : (
              <Button variant="ghost" onClick={onClose}>
                Close
              </Button>
            ))}
          {phase === 'working' && (
            <Button variant="ghost" disabled>
              Working…
            </Button>
          )}
          {phase === 'escalate' && (
            <>
              <Button variant="ghost" onClick={onClose}>
                Leave them running
              </Button>
              <Button variant="danger" onClick={() => void run(survivors, true)}>
                Force kill {survivors.length}
              </Button>
            </>
          )}
          {phase === 'done' && (
            <Button variant="ghost" onClick={onClose}>
              Close
            </Button>
          )}
        </div>
      </div>
    </div>
  )
}

function ServiceLine({ service, outcome }: { service: Service; outcome?: Outcome }) {
  return (
    <div className="flex items-baseline gap-2">
      <span className="w-14 flex-none text-right font-mono font-semibold">:{service.port}</span>
      <span className="truncate">
        {displayName(service)}
        {service.pid !== undefined && (
          <span className="text-faint"> · PID {service.pid}</span>
        )}
      </span>
      <span className="ml-auto flex-none">
        <OutcomeBadge outcome={outcome} />
      </span>
    </div>
  )
}

function OutcomeBadge({ outcome }: { outcome?: Outcome }) {
  switch (outcome?.state) {
    case undefined:
    case 'pending':
      return null
    case 'stopping':
      return <span className="text-faint">stopping…</span>
    case 'released':
      return (
        <Badge variant="ok" dot>
          stopped
        </Badge>
      )
    case 'still_listening':
      return (
        <Badge variant="warn" dot>
          still listening
        </Badge>
      )
    case 'error':
      return (
        <Badge variant="danger" dot title={outcome.message}>
          {outcome.message ?? 'failed'}
        </Badge>
      )
  }
}

function Summary({
  outcomes,
  stoppable,
}: {
  outcomes: Record<string, Outcome>
  stoppable: Service[]
}) {
  const count = (state: Outcome['state']) =>
    stoppable.filter((s) => outcomes[s.id]?.state === state).length
  const stopped = count('released')
  const alive = count('still_listening')
  const failed = count('error')
  return (
    <p className="text-muted">
      {stopped} stopped
      {alive > 0 && `, ${alive} still listening`}
      {failed > 0 && `, ${failed} failed`}.
    </p>
  )
}
