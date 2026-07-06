import {
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  useReactTable,
} from '@tanstack/react-table'
import { EllipsisVertical, ExternalLink, Square } from 'lucide-react'
import type { BadgeVariant } from './ui/badge'
import type { DevSnapshot, DockerHint, Exposure, ProjectGroup, Service } from '../lib/types'
import { conflictedIds, isSelf } from '../lib/derive'
import { Badge } from './ui/badge'
import { Button } from './ui/button'
import { cn } from '../lib/cn'

interface Row {
  service: Service
  project?: ProjectGroup
  dockerHint?: DockerHint
  conflicted: boolean
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
          {row.original.service.process_name ?? 'unknown'}
        </span>
        {row.original.service.framework && <Badge>{row.original.service.framework}</Badge>}
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
      const { service, conflicted } = row.original
      return (
        <span className="flex items-center gap-1.5">
          {conflicted && (
            <Badge variant="danger" dot>
              conflict
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
      const self = isSelf(service)
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
              disabled
              title={self ? 'Protected - PortDoc will not stop itself' : 'Safe stop lands with feature 12'}
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

export function ServicesTable({ snapshot }: { snapshot: DevSnapshot }) {
  const conflicted = conflictedIds(snapshot)
  const projectById = new Map(snapshot.projects.map((p) => [p.id, p]))
  const hintFor = new Map(
    snapshot.docker_hints.filter((h) => h.service_id).map((h) => [h.service_id, h]),
  )
  const rows: Row[] = snapshot.services.map((service) => ({
    service,
    project: service.project_id ? projectById.get(service.project_id) : undefined,
    dockerHint: hintFor.get(service.id),
    conflicted: conflicted.has(service.id),
  }))

  const table = useReactTable({ data: rows, columns, getCoreRowModel: getCoreRowModel() })

  return (
    <>
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
            {table.getRowModel().rows.map((row) => (
              <tr key={row.id} className="group border-b border-border last:border-b-0 hover:bg-surface-2">
                {row.getVisibleCells().map((cell, i) => (
                  <td
                    key={cell.id}
                    className={cn(
                      'whitespace-nowrap px-3 py-[7px] align-middle',
                      i === 0 &&
                        row.original.conflicted &&
                        'shadow-[inset_3px_0_0_var(--color-danger)]',
                    )}
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
        {snapshot.services.length} services · sorted by project · hover a row for actions
      </p>
    </>
  )
}
