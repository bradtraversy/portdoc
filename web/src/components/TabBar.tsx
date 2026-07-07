import { Box, Folder, LayoutDashboard, Server, SlidersHorizontal } from 'lucide-react'
import type { DevSnapshot } from '../lib/types'
import { visibleServices } from '../lib/derive'
import { useConfig } from '../lib/config'
import { cn } from '../lib/cn'

export type TabId = 'dashboard' | 'projects' | 'services' | 'docker' | 'advanced'

interface TabBarProps {
  active: TabId
  onSelect: (tab: TabId) => void
  snapshot: DevSnapshot | null
}

export function TabBar({ active, onSelect, snapshot }: TabBarProps) {
  const { ignored } = useConfig()
  const serviceCount = snapshot ? visibleServices(snapshot, ignored).length : undefined
  const tabs = [
    { id: 'dashboard' as const, label: 'Dashboard', Icon: LayoutDashboard },
    { id: 'projects' as const, label: 'Projects', Icon: Folder, count: snapshot?.projects.length },
    { id: 'services' as const, label: 'Services', Icon: Server, count: serviceCount },
    { id: 'docker' as const, label: 'Docker', Icon: Box, count: snapshot?.docker_hints.length },
    { id: 'advanced' as const, label: 'Advanced', Icon: SlidersHorizontal },
  ]

  return (
    <nav className="flex gap-0.5 overflow-x-auto border-b border-border bg-surface px-5">
      {tabs.map(({ id, label, Icon, count }) => (
        <button
          key={id}
          onClick={() => onSelect(id)}
          className={cn(
            'flex cursor-pointer items-center gap-1.5 whitespace-nowrap border-b-2 px-3.5 py-2 text-sm font-medium',
            active === id ? 'border-accent text-text' : 'border-transparent text-muted hover:text-text',
          )}
        >
          <Icon className="size-[13px]" />
          {label}
          {count !== undefined && (
            <span className="rounded-full border border-border bg-surface-2 px-1.5 text-[10px] text-muted">
              {count}
            </span>
          )}
        </button>
      ))}
    </nav>
  )
}
