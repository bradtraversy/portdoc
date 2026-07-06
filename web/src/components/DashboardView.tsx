import type { DevSnapshot } from '../lib/types'
import type { TabId } from './TabBar'
import { StatCards } from './StatCards'
import { Callouts } from './Callouts'
import { ProjectGroups } from './ProjectGroups'

interface DashboardViewProps {
  snapshot: DevSnapshot
  onNavigate: (tab: TabId) => void
}

export function DashboardView({ snapshot, onNavigate }: DashboardViewProps) {
  return (
    <>
      <StatCards snapshot={snapshot} onNavigate={onNavigate} />
      <Callouts snapshot={snapshot} onNavigate={onNavigate} />
      <ProjectGroups snapshot={snapshot} />
    </>
  )
}
