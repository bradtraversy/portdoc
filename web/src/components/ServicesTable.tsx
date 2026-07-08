import { useMemo, useState } from 'react'
import {
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from '@tanstack/react-table'
import { EllipsisVertical, ExternalLink, Square } from 'lucide-react'
import type { BadgeVariant } from './ui/badge'
import type { DevSnapshot, DockerHint, Exposure, ProjectGroup, Service } from '../lib/types'
import {
  canStop,
  conflictedIds,
  displayName,
  isSelf,
  stopBlockedReason,
  wellKnownHint,
} from '../lib/derive'
import { useRequestStop } from '../lib/stop'
import { useInspect } from '../lib/inspect'
import { useConfig } from '../lib/config'
import { CHIPS, type FilterChip, matchesChip, matchesQuery } from '../lib/filter'
import { Badge } from './ui/badge'
import { Button } from './ui/button'
import { cn } from '../lib/cn'

interface Row {
  service: Service
  project?: ProjectGroup
  dockerHint?: DockerHint
  conflicted: boolean
  ignored: boolean
  onStop: (service: Service) => void
}

const exposureVariant: Record<Exposure, BadgeVariant> = {
  local: 'default',
  lan: 'warn',
  docker: 'info',
  unknown: 'default',
}

const columnHelper = createColumnHelper<Row>()

const columns = [
  columnHelper.display({
    id: 'service',
    header: 'Service',
    cell: ({ row }) => (
      <span className="flex items-center gap-2">
        <span className={cn('font-semibold', row.original.service.stale && 'text-muted')}>
          {displayName(row.original.service)}
        </span>
        {row.original.service.framework && row.original.service.process_name && (
          <span className="font-mono text-xs text-faint">
            {row.original.service.process_name}
          </span>
        )}
        {wellKnownHint(row.original.service) && (
          <span className="text-xs text-faint">{wellKnownHint(row.original.service)}</span>
        )}
      </span>
    ),
  }),
  columnHelper.display({
    id: 'port',
    header: 'Port',
    cell: ({ row }) => (
      <span className="font-mono font-semibold tabular-nums">:{row.original.service.port}</span>
    ),
  }),
  columnHelper.display({
    id: 'pid',
    header: 'PID',
    cell: ({ row }) => (
      <span className="font-mono tabular-nums text-muted">{row.original.service.pid ?? '-'}</span>
    ),
  }),
  columnHelper.display({
    id: 'command',
    header: 'Command',
    cell: ({ row }) => {
      const { service, dockerHint } = row.original
      const text = service.command ?? (dockerHint ? `container ${dockerHint.container}` : '-')
      return (
        <span
          className={cn(
            'block max-w-[260px] truncate font-mono text-xs',
            row.original.service.stale ? 'text-faint' : 'text-muted',
          )}
          title={text}
        >
          {text}
        </span>
      )
    },
  }),
  columnHelper.display({
    id: 'project',
    header: 'Project',
    cell: ({ row }) => {
      const { project, dockerHint } = row.original
      const name = project?.name ?? (dockerHint ? 'Docker' : '-')
      const path = project?.root ?? (dockerHint ? `container ${dockerHint.container}` : undefined)
      return (
        <span className="block leading-tight">
          <span className="font-medium">{name}</span>
          {path && <span className="block font-mono text-[10.5px] text-faint">{path}</span>}
        </span>
      )
    },
  }),
  columnHelper.display({
    id: 'exposure',
    header: 'Exposure',
    cell: ({ row }) => {
      const { exposure } = row.original.service
      return (
        <Badge variant={exposureVariant[exposure]} dot>
          {exposure === 'lan' ? 'LAN' : exposure}
        </Badge>
      )
    },
  }),
  columnHelper.display({
    id: 'status',
    header: 'Status',
    cell: ({ row }) => {
      const { service, conflicted, ignored } = row.original
      return (
        <span className="flex items-center gap-1.5">
          {ignored && <Badge title="Hidden from the dashboard">ignored</Badge>}
          {conflicted && (
            <Badge dot title="Another process is also listening on this port">
              shared port
            </Badge>
          )}
          {service.stale ? (
            <Badge variant="warn" dot title={service.stale.reason}>
              stale
            </Badge>
          ) : (
            <Badge variant="ok" dot>
              running
            </Badge>
          )}
          {isSelf(service) && <Badge title="PortDoc will not stop itself">protected</Badge>}
        </span>
      )
    },
  }),
  columnHelper.display({
    id: 'age',
    header: 'Age',
    cell: ({ row }) => (
      <span className="text-xs text-muted">{row.original.service.started_age ?? '-'}</span>
    ),
  }),
  columnHelper.display({
    id: 'actions',
    header: '',
    cell: ({ row }) => {
      const { service } = row.original
      const stoppable = canStop(service)
      return (
        <span className="flex items-center justify-end gap-0.5">
          <span className="hidden gap-0.5 group-hover:inline-flex">
            <Button
              size="sm"
              variant="ghost"
              disabled={!service.url}
              title={service.url ? undefined : 'No local URL detected'}
              onClick={() => service.url && window.open(service.url)}
            >
              <ExternalLink className="size-3" />
              Open
            </Button>
            <Button
              size="sm"
              variant="ghost"
              disabled={!stoppable}
              title={stoppable ? undefined : stopBlockedReason(service)}
              onClick={() => row.original.onStop(service)}
            >
              <Square className="size-3" />
              Stop
            </Button>
          </span>
          <Button size="sm" variant="ghost" title="More actions">
            <EllipsisVertical className="size-3" />
          </Button>
        </span>
      )
    },
  }),
]

interface ServicesTableProps {
  snapshot: DevSnapshot
  query: string
  onQueryChange: (query: string) => void
}

export function ServicesTable({ snapshot, query, onQueryChange }: ServicesTableProps) {
  const [chips, setChips] = useState<ReadonlySet<FilterChip>>(new Set())
  const [showIgnored, setShowIgnored] = useState(false)
  const requestStop = useRequestStop()
  const inspect = useInspect()
  const { ignored } = useConfig()

  // TanStack compares data by reference; unstable arrays here cause
  // infinite re-render loops (froze the tab when search landed).
  const rows: Row[] = useMemo(() => {
    const conflicted = conflictedIds(snapshot)
    const projectById = new Map(snapshot.projects.map((p) => [p.id, p]))
    const hintFor = new Map(
      snapshot.docker_hints.filter((h) => h.service_id).map((h) => [h.service_id, h]),
    )
    return snapshot.services.map((service) => ({
      service,
      project: service.project_id ? projectById.get(service.project_id) : undefined,
      dockerHint: hintFor.get(service.id),
      conflicted: conflicted.has(service.id),
      ignored: ignored.has(service.id),
      onStop: requestStop,
    }))
  }, [snapshot, requestStop, ignored])

  const ignoredCount = useMemo(() => rows.filter((r) => r.ignored).length, [rows])
  const visibleRows = useMemo(
    () => (showIgnored ? rows : rows.filter((r) => !r.ignored)),
    [rows, showIgnored],
  )

  // chips OR together; the text query ANDs on top
  const filtered = useMemo(
    () =>
      visibleRows.filter(
        (row) =>
          matchesQuery(row.service, query, row.project?.name) &&
          (chips.size === 0 ||
            [...chips].some((c) => matchesChip(c, row.service, row.conflicted))),
      ),
    [visibleRows, query, chips],
  )
  const table = useReactTable({ data: filtered, columns, getCoreRowModel: getCoreRowModel() })

  const toggleChip = (chip: FilterChip) => {
    setChips((prev) => {
      const next = new Set(prev)
      if (!next.delete(chip)) next.add(chip)
      return next
    })
  }
  const hasFilters = query !== '' || chips.size > 0
  const clearFilters = () => {
    onQueryChange('')
    setChips(new Set())
  }

  return (
    <>
      <div className="flex flex-wrap items-center gap-2">
        {CHIPS.map(({ id, label }) => {
          const active = chips.has(id)
          return (
            <button
              key={id}
              type="button"
              onClick={() => toggleChip(id)}
              aria-pressed={active}
              className={cn(
                'rounded-full border px-2.5 py-1 text-xs transition-colors',
                active
                  ? 'border-accent text-accent'
                  : 'border-border bg-surface text-muted hover:border-border-strong hover:text-text',
              )}
            >
              {label}
            </button>
          )
        })}
        {hasFilters && (
          <Button size="sm" variant="ghost" onClick={clearFilters}>
            Clear
          </Button>
        )}
        {ignoredCount > 0 && (
          <button
            type="button"
            onClick={() => setShowIgnored((v) => !v)}
            aria-pressed={showIgnored}
            className="ml-auto text-xs text-muted hover:text-text hover:underline"
          >
            {showIgnored
              ? 'hide ignored'
              : `${ignoredCount} ignored - show`}
          </button>
        )}
      </div>
      <div className="overflow-x-auto rounded-lg border border-border bg-surface">
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr>
              {table.getFlatHeaders().map((header) => (
                <th
                  key={header.id}
                  className="whitespace-nowrap border-b border-border-strong px-3 py-2 text-left text-xs font-semibold uppercase tracking-wide text-muted"
                >
                  {flexRender(header.column.columnDef.header, header.getContext())}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {filtered.length === 0 && (
              <tr>
                <td colSpan={columns.length} className="px-3 py-8 text-center text-sm text-faint">
                  No services match.{' '}
                  <button
                    type="button"
                    onClick={clearFilters}
                    className="text-accent hover:underline"
                  >
                    Clear filters
                  </button>
                </td>
              </tr>
            )}
            {table.getRowModel().rows.map((row) => (
              <tr
                key={row.id}
                className="group cursor-pointer border-b border-border last:border-b-0 hover:bg-surface-2"
                onClick={() => inspect({ services: [row.original.service] })}
              >
                {row.getVisibleCells().map((cell) => (
                  <td
                    key={cell.id}
                    onClick={cell.column.id === 'actions' ? (e) => e.stopPropagation() : undefined}
                    className="whitespace-nowrap px-3 py-[7px] align-middle"
                  >
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p className="px-1 text-xs text-faint">
        {filtered.length === visibleRows.length
          ? `${visibleRows.length} services`
          : `${filtered.length} of ${visibleRows.length} services`}{' '}
        · click a row to inspect
      </p>
    </>
  )
}
