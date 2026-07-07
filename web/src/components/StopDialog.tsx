import { useEffect, useState } from 'react'
import { TriangleAlert, X } from 'lucide-react'
import type { Service } from '../lib/types'
import { postStop } from '../lib/stop'
import { Button } from './ui/button'

interface StopDialogProps {
  service: Service
  onClose: () => void
  onStopped: () => void
}

type Phase = 'confirm' | 'working' | 'escalate' | 'error'

export function StopDialog({ service, onClose, onStopped }: StopDialogProps) {
  const [phase, setPhase] = useState<Phase>('confirm')
  const [message, setMessage] = useState('')

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && phase !== 'working') onClose()
    }
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [phase, onClose])

  const stop = async (force: boolean) => {
    setPhase('working')
    try {
      const { outcome } = await postStop(service, force)
      if (outcome === 'released') {
        onStopped()
      } else {
        setPhase('escalate')
      }
    } catch (err) {
      setMessage(err instanceof Error ? err.message : String(err))
      setPhase('error')
    }
  }

  const field = (label: string, value: string) => (
    <div className="flex gap-2">
      <span className="w-16 flex-none text-faint">{label}</span>
      <span className="min-w-0 break-words font-mono text-text">{value}</span>
    </div>
  )

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
      onClick={(e) => e.target === e.currentTarget && phase !== 'working' && onClose()}
    >
      <div
        role="dialog"
        aria-modal="true"
        className="w-full max-w-md rounded-lg border border-border bg-surface shadow-xl"
      >
        <div className="flex items-center gap-2 border-b border-border px-4 py-3">
          <h2 className="text-sm font-semibold">
            {phase === 'escalate' ? 'Force kill?' : `Stop ${service.process_name ?? 'service'}?`}
          </h2>
          {phase !== 'working' && (
            <button className="ml-auto text-faint hover:text-text" onClick={onClose} title="Cancel">
              <X className="size-4" />
            </button>
          )}
        </div>

        <div className="space-y-3 px-4 py-3.5 text-sm">
          <div className="space-y-1 rounded-md border border-border bg-surface-2 px-3 py-2 text-xs">
            {field('Port', `:${service.port}`)}
            {service.pid !== undefined && field('PID', String(service.pid))}
            {service.command && field('Command', service.command)}
            {service.cwd && field('Path', service.cwd)}
          </div>

          {phase === 'confirm' && (
            <p className="text-muted">
              Sends a graceful stop (SIGTERM), then verifies the port releases.
            </p>
          )}
          {phase === 'working' && <p className="text-muted">Stopping and verifying the port…</p>}
          {phase === 'escalate' && (
            <p className="flex items-start gap-2 text-warn">
              <TriangleAlert className="mt-0.5 size-4 flex-none" />
              The process ignored the graceful stop and is still listening. Force kill (SIGKILL)
              ends it immediately without cleanup.
            </p>
          )}
          {phase === 'error' && (
            <p className="flex items-start gap-2 text-danger">
              <TriangleAlert className="mt-0.5 size-4 flex-none" />
              {message}
            </p>
          )}
        </div>

        <div className="flex justify-end gap-2 border-t border-border px-4 py-3">
          {phase === 'confirm' && (
            <>
              <Button variant="ghost" onClick={onClose}>
                Cancel
              </Button>
              <Button variant="accent" onClick={() => void stop(false)}>
                Stop service
              </Button>
            </>
          )}
          {phase === 'working' && (
            <Button variant="ghost" disabled>
              Working…
            </Button>
          )}
          {phase === 'escalate' && (
            <>
              <Button variant="ghost" onClick={onClose}>
                Leave it running
              </Button>
              <Button variant="danger" onClick={() => void stop(true)}>
                Force kill
              </Button>
            </>
          )}
          {phase === 'error' && (
            <Button variant="ghost" onClick={onClose}>
              Close
            </Button>
          )}
        </div>
      </div>
    </div>
  )
}
