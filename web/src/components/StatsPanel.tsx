import type { MachinesView, RepoEntry, StatsView } from '../game/types'

interface StatsPanelProps {
  stats: StatsView
  machines: MachinesView
  repo: RepoEntry[]
}

export function StatsPanel({ stats, machines, repo }: StatsPanelProps) {
  const stored = repo.reduce((total, entry) => total + entry.count, 0)

  return (
    <section className="panel">
      <header className="panel-head">
        <h2>Factory</h2>
      </header>
      <dl className="stats">
        <Stat label="Revenue" value={stats.revenue} />
        <Stat label="Crafted" value={stats.crafted} />
        <Stat label="Stored" value={stored} />
        <Stat
          label="Machines"
          value={`${machines.placed} / ${machines.capacity}`}
          hint={`${machines.free_slots} slots left for loot`}
        />
      </dl>
    </section>
  )
}

function Stat({ label, value, hint }: { label: string; value: string | number; hint?: string }) {
  return (
    <div className="stat" title={hint}>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  )
}
