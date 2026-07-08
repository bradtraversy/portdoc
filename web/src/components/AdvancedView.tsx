import { useEffect, useState } from 'react'
import { Copy, Download, HelpCircle, TriangleAlert } from 'lucide-react'
import { Badge } from './ui/badge'
import { Button } from './ui/button'

interface SocketDetail {
  protocol: string
  local_addr: string
  port: number
  pid?: number
  process_name?: string
  uid?: number
  user?: string
}

interface Diagnostic {
  port: number
  hint?: string
  reason: string
}

interface SocketsResponse {
  probe?: string
  sockets: SocketDetail[]
  diagnostics: Diagnostic[]
}

export function AdvancedView() {
  const [data, setData] = useState<SocketsResponse | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    fetch('/api/sockets')
      .then((res) => {
        if (!res.ok) throw new Error(`sockets request failed (${res.status})`)
        return res.json() as Promise<SocketsResponse>
      })
      .then((json) => !cancelled && setData(json))
      .catch((err: unknown) => !cancelled && setError(err instanceof Error ? err.message : 'request failed'))
    return () => {
      cancelled = true
    }
  }, [])

  if (error) {
    return (
      <div className="flex items-center gap-3 rounded-lg border border-danger/30 bg-danger/10 px-3.5 py-2.5 text-sm">
        <TriangleAlert className="size-3.5 flex-none text-danger" />
        <span>
          <strong className="font-semibold">Socket details failed:</strong> {error}
        </span>
      </div>
    )
  }
  if (!data) return <p className="text-sm text-faint">Loading socket details</p>

  return (
    <>
      {data.diagnostics.length > 0 && <UnknownOwners diagnostics={data.diagnostics} />}
      <RawSockets sockets={data.sockets} probe={data.probe} />
      <JsonExport />
    </>
  )
}

function UnknownOwners({ diagnostics }: { diagnostics: Diagnostic[] }) {
  return (
    <section className="overflow-hidden rounded-lg border border-border bg-surface">
      <div className="flex items-center gap-2.5 border-b border-border px-4 py-2.5">
        <HelpCircle className="size-[13px] text-faint" />
        <span className="font-semibold">Unknown owners</span>
        <span className="text-xs text-faint">why some rows have no process</span>
      </div>
      {diagnostics.map((d) => (
        <div
          key={d.port}
          className="grid grid-cols-[64px_1fr] items-baseline gap-3.5 border-b border-border px-4 py-2 last:border-b-0"
        >
          <span className="text-right font-mono text-sm font-semibold tabular-nums">:{d.port}</span>
          <div className="min-w-0">
            {d.hint && <Badge>{d.hint}</Badge>}
            <span className={`block text-xs text-muted ${d.hint ? 'mt-1' : ''}`}>{d.reason}</span>
          </div>
        </div>
      ))}
    </section>
  )
}

function RawSockets({ sockets, probe }: { sockets: SocketDetail[]; probe?: string }) {
  return (
    <section className="overflow-hidden rounded-lg border border-border bg-surface">
      <div className="flex items-center gap-2.5 border-b border-border px-4 py-2.5">
        <span className="font-semibold">Raw sockets</span>
        <span className="text-xs text-faint">
          pre-merge listening sockets, TCP only{probe ? ` · probe: ${probe}` : ''}
        </span>
      </div>
      <div className="overflow-x-auto">
        <table className="w-full text-left font-mono text-xs">
          <thead>
            <tr className="border-b border-border text-faint">
              <th className="px-4 py-1.5 font-medium">proto</th>
              <th className="px-2 py-1.5 font-medium">bind address</th>
              <th className="px-2 py-1.5 font-medium">port</th>
              <th className="px-2 py-1.5 font-medium">pid</th>
              <th className="px-2 py-1.5 font-medium">process</th>
              <th className="px-2 py-1.5 font-medium">owner</th>
            </tr>
          </thead>
          <tbody>
            {sockets.map((s, i) => (
              <tr key={i} className="border-b border-border last:border-b-0">
                <td className="px-4 py-1.5 text-muted">{s.protocol}</td>
                <td className="px-2 py-1.5">{s.local_addr}</td>
                <td className="px-2 py-1.5 font-semibold tabular-nums">:{s.port}</td>
                <td className="px-2 py-1.5 text-muted">{s.pid ?? '-'}</td>
                <td className="px-2 py-1.5">{s.process_name ?? '-'}</td>
                <td className="px-2 py-1.5 text-muted">
                  {s.user ?? (s.uid !== undefined ? `uid ${s.uid}` : '-')}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  )
}

function JsonExport() {
  const [state, setState] = useState<'idle' | 'copied' | 'failed'>('idle')
  const flash = (s: 'copied' | 'failed') => {
    setState(s)
    setTimeout(() => setState('idle'), 1600)
  }

  const snapshotText = async () => {
    const res = await fetch('/api/snapshot')
    if (!res.ok) throw new Error()
    return JSON.stringify(await res.json(), null, 2)
  }

  const download = async () => {
    try {
      const blob = new Blob([await snapshotText()], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `portdoc-snapshot-${new Date().toISOString().slice(0, 19).replaceAll(':', '-')}.json`
      a.click()
      URL.revokeObjectURL(url)
    } catch {
      flash('failed')
    }
  }

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(await snapshotText())
      flash('copied')
    } catch {
      flash('failed')
    }
  }

  return (
    <section className="rounded-lg border border-border bg-surface px-4 py-3">
      <div className="flex flex-wrap items-center gap-2">
        <span className="font-semibold">JSON export</span>
        <span className="text-xs text-faint">
          the full snapshot; <code className="font-mono">portdoc --json</code> prints the same thing
        </span>
        <div className="ml-auto flex gap-1.5">
          <Button size="sm" variant="ghost" onClick={() => void download()}>
            <Download className="size-3" />
            Download
          </Button>
          <Button size="sm" variant="ghost" onClick={() => void copy()}>
            <Copy className="size-3" />
            {state === 'copied' ? 'Copied' : state === 'failed' ? 'Failed' : 'Copy JSON'}
          </Button>
        </div>
      </div>
    </section>
  )
}
