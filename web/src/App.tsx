import { useState } from 'react'
import { TriangleAlert } from 'lucide-react'
import { useSnapshot } from './lib/useSnapshot'
import { StopContext } from './lib/stop'
import { ConfigContext, useConfigState } from './lib/config'
import { InspectContext, type InspectTarget } from './lib/inspect'
import type { Service } from './lib/types'
import { TopBar } from './components/TopBar'
import { TabBar, type TabId } from './components/TabBar'
import { AdvancedView } from './components/AdvancedView'
import { DashboardView } from './components/DashboardView'
import { DockerView } from './components/DockerView'
import { ProjectGroups } from './components/ProjectGroups'
import { ServicesTable } from './components/ServicesTable'
import { StopDialog } from './components/StopDialog'
import { InspectDrawer } from './components/InspectDrawer'
import { Button } from './components/ui/button'

export default function App() {
  const [tab, setTab] = useState<TabId>('dashboard')
  const [query, setQuery] = useState('')
  const [stopTarget, setStopTarget] = useState<Service | null>(null)
  const [inspect, setInspect] = useState<InspectTarget | null>(null)
  const { snapshot, error, loading, fetchedAt, refresh } = useSnapshot()
  const configState = useConfigState()

  // typing anywhere jumps to the Services tab with the query applied
  const searchFrom = (q: string) => {
    setQuery(q)
    if (q && tab !== 'services') setTab('services')
  }

  return (
    <StopContext.Provider value={setStopTarget}>
    <InspectContext.Provider value={setInspect}>
    <ConfigContext.Provider value={configState}>
    <div className="min-h-screen">
      <TopBar
        fetchedAt={fetchedAt}
        refreshing={loading}
        onRefresh={() => void refresh()}
        query={query}
        onQueryChange={searchFrom}
      />
      <TabBar active={tab} onSelect={setTab} snapshot={snapshot} />
      <main className="mx-auto grid max-w-[1180px] content-start gap-4 p-5 pb-12">
        {error && (
          <div className="flex items-center gap-3 rounded-lg border border-danger/30 bg-danger/10 px-3.5 py-2.5 text-sm">
            <TriangleAlert className="size-3.5 flex-none text-danger" />
            <span>
              <strong className="font-semibold">Snapshot failed:</strong> {error}
            </span>
            <Button size="sm" className="ml-auto" onClick={() => void refresh()}>
              Retry
            </Button>
          </div>
        )}
        {!snapshot && !error && <p className="text-sm text-faint">Loading snapshot</p>}
        {snapshot && tab === 'dashboard' && <DashboardView snapshot={snapshot} />}
        {snapshot && tab === 'projects' && <ProjectGroups snapshot={snapshot} />}
        {snapshot && tab === 'services' && (
          <ServicesTable snapshot={snapshot} query={query} onQueryChange={setQuery} />
        )}
        {snapshot && tab === 'docker' && <DockerView snapshot={snapshot} />}
        {snapshot && tab === 'advanced' && <AdvancedView />}
      </main>
      {snapshot && inspect && (
        <InspectDrawer target={inspect} snapshot={snapshot} onClose={() => setInspect(null)} />
      )}
      {stopTarget && (
        <StopDialog
          service={stopTarget}
          onClose={() => setStopTarget(null)}
          onStopped={() => {
            setStopTarget(null)
            void refresh()
          }}
        />
      )}
    </div>
    </ConfigContext.Provider>
    </InspectContext.Provider>
    </StopContext.Provider>
  )
}
