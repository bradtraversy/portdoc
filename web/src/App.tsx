import { useState } from 'react'
import { TriangleAlert } from 'lucide-react'
import { useSnapshot } from './lib/useSnapshot'
import { TopBar } from './components/TopBar'
import { TabBar, type TabId } from './components/TabBar'
import { Placeholder } from './components/Placeholder'
import { DashboardView } from './components/DashboardView'
import { ServicesTable } from './components/ServicesTable'
import { Button } from './components/ui/button'

const placeholders: Record<Exclude<TabId, 'dashboard' | 'services'>, { title: string; note: string }> = {
  projects: { title: 'Projects', note: 'Project-grouped view lands with real project detection (feature 7).' },
  conflicts: { title: 'Conflicts', note: 'Conflict details and actions land with feature 10.' },
  docker: { title: 'Docker', note: 'Container and Compose view lands with feature 14.' },
  advanced: { title: 'Advanced', note: 'Raw sockets, JSON export, and diagnostics land with feature 14.' },
}

export default function App() {
  const [tab, setTab] = useState<TabId>('dashboard')
  const { snapshot, error, loading, fetchedAt, refresh } = useSnapshot()

  return (
    <div className="min-h-screen">
      <TopBar fetchedAt={fetchedAt} refreshing={loading} onRefresh={() => void refresh()} />
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
        {snapshot && tab === 'dashboard' && (
          <DashboardView snapshot={snapshot} onNavigate={setTab} />
        )}
        {snapshot && tab === 'services' && <ServicesTable snapshot={snapshot} />}
        {snapshot && tab !== 'dashboard' && tab !== 'services' && (
          <Placeholder title={placeholders[tab].title} note={placeholders[tab].note} />
        )}
      </main>
    </div>
  )
}
