import { useEffect, useState } from 'react'
import { Clock, RefreshCw } from 'lucide-react'
import { Button } from './ui/button'

function SnapshotAge({ fetchedAt }: { fetchedAt: number | null }) {
  const [, setTick] = useState(0)

  useEffect(() => {
    const id = setInterval(() => setTick((t) => t + 1), 5000)
    return () => clearInterval(id)
  }, [])

  if (fetchedAt === null) return null
  const secs = Math.max(0, Math.round((Date.now() - fetchedAt) / 1000))
  const label = secs < 60 ? `${secs}s ago` : `${Math.floor(secs / 60)}m ago`
  return (
    <span className="inline-flex items-center gap-1.5 text-xs text-faint">
      <Clock className="size-3" />
      snapshot {label}
    </span>
  )
}

interface TopBarProps {
  fetchedAt: number | null
  refreshing: boolean
  onRefresh: () => void
}

export function TopBar({ fetchedAt, refreshing, onRefresh }: TopBarProps) {
  return (
    <header className="flex items-center gap-3 border-b border-border bg-surface px-5 py-2.5">
      <div className="flex items-center gap-2 text-[15px] font-bold">
        <span className="inline-flex size-[22px] items-center justify-center rounded-md bg-accent text-accent-ink">
          <svg
            viewBox="0 0 24 24"
            className="size-[15px]"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.6"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <polyline points="3 12 7.5 12 10 6.5 13.5 17.5 16 12 21 12" />
          </svg>
        </span>
        <span>
          Port<span className="text-accent">Doc</span>
        </span>
      </div>
      <span className="rounded-full border border-border bg-surface-2 px-2.5 py-[3px] font-mono text-xs text-muted">
        {window.location.host}
      </span>
      <div className="ml-auto flex items-center gap-3">
        <SnapshotAge fetchedAt={fetchedAt} />
        <Button onClick={onRefresh} disabled={refreshing}>
          <RefreshCw className="size-3.5" />
          Refresh
        </Button>
      </div>
    </header>
  )
}
