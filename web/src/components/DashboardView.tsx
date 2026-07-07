import type { DevSnapshot } from '../lib/types'
import { StatCards } from './StatCards'
import { Callouts } from './Callouts'
import { ProjectGroups } from './ProjectGroups'
import { PortLookup } from './PortLookup'

export function DashboardView({ snapshot }: { snapshot: DevSnapshot }) {
  return (
    <>
      <StatCards snapshot={snapshot} />
      <PortLookup snapshot={snapshot} />
      <Callouts snapshot={snapshot} />
      <ProjectGroups snapshot={snapshot} />
    </>
  )
}
