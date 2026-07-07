import { Activity, Folder, Wifi } from 'lucide-react'
import type { DevSnapshot } from '../lib/types'
import { commonRoot, dockerServices, lanServices } from '../lib/derive'
import { cn } from '../lib/cn'

interface Stat {
  label: string
  icon: React.ReactNode
  value: number
  valueClass?: string
  sub?: React.ReactNode
}

export function StatCards({ snapshot }: { snapshot: DevSnapshot }) {
  const lan = lanServices(snapshot)
  const root = commonRoot(snapshot)
  const hasDocker = dockerServices(snapshot).length > 0

  const stats: Stat[] = [
    {
      label: 'Running services',
      icon: <Activity className="size-3.5 text-ok" />,
      value: snapshot.services.length,
      sub: `across ${snapshot.projects.length} projects${hasDocker ? ' + Docker' : ''}`,
    },
    {
      label: 'Projects active',
      icon: <Folder className="size-3.5" />,
      value: snapshot.projects.length,
      sub: root ? `under ${root}` : undefined,
    },
    {
      label: 'LAN visible',
      icon: <Wifi className="size-3.5 text-warn" />,
      value: lan.length,
      valueClass: lan.length > 0 ? 'text-warn' : undefined,
      sub: lan[0] ? `${lan[0].framework ?? lan[0].process_name ?? 'service'} on :${lan[0].port}` : undefined,
    },
  ]

  return (
    <section className="grid grid-cols-2 gap-3 lg:grid-cols-3">
      {stats.map((stat) => (
        <div key={stat.label} className="rounded-lg border border-border bg-surface px-4 py-3.5">
          <div className="flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-muted">
            {stat.icon}
            {stat.label}
          </div>
          <div className={cn('mt-1 text-xl font-bold tabular-nums', stat.valueClass)}>
            {stat.value}
          </div>
          <div className="text-xs text-faint">{stat.sub}</div>
        </div>
      ))}
    </section>
  )
}
