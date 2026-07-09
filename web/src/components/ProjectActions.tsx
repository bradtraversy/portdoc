import { useState } from 'react'
import { Code, Copy, Square } from 'lucide-react'
import type { ProjectGroup, Service } from '../lib/types'
import { useRequestStopAll } from '../lib/stop'
import { Button } from './ui/button'

/// Project-level quick actions (16c). Rendered in the Projects tab group
/// headers and the project drawer; stops click-through to the drawer.
export function ProjectActions({
  project,
  services,
}: {
  project: ProjectGroup
  services: Service[]
}) {
  const requestStopAll = useRequestStopAll()
  return (
    <div className="flex flex-wrap items-center gap-1.5" onClick={(e) => e.stopPropagation()}>
      <OpenInEditorButton root={project.root} />
      <CopyCdButton root={project.root} />
      <Button
        size="sm"
        variant="ghost"
        title="Stop every service in this project, with confirmation"
        onClick={() => requestStopAll({ projectName: project.name, services })}
      >
        <Square className="size-3" />
        Stop all
      </Button>
    </div>
  )
}

function OpenInEditorButton({ root }: { root: string }) {
  const [error, setError] = useState('')
  const open = async () => {
    setError('')
    try {
      const res = await fetch('/api/open', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ path: root }),
      })
      if (!res.ok) {
        const body = (await res.json()) as { error?: string }
        throw new Error(body.error ?? `open failed (${res.status})`)
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }
  return (
    <Button
      size="sm"
      variant="ghost"
      title={error || 'Open the project folder in your editor (config "editor", default code)'}
      onClick={() => void open()}
    >
      <Code className="size-3" />
      {error ? 'Failed' : 'Open in editor'}
    </Button>
  )
}

function CopyCdButton({ root }: { root: string }) {
  const [copied, setCopied] = useState(false)
  const copy = async () => {
    if (!navigator.clipboard) return
    await navigator.clipboard.writeText(`cd ${root}`)
    setCopied(true)
    setTimeout(() => setCopied(false), 1200)
  }
  return (
    <Button size="sm" variant="ghost" title={`cd ${root}`} onClick={() => void copy()}>
      <Copy className="size-3" />
      {copied ? 'Copied' : 'Copy cd'}
    </Button>
  )
}
