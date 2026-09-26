import { useEffect, useMemo, useState } from 'react'
import { Codex } from './components/Codex'
import { GoalPanel } from './components/GoalPanel'
import { Grid } from './components/Grid'
import { NewRunOverlay } from './components/NewRunOverlay'
import { Palette } from './components/Palette'
import { RelicPanel } from './components/RelicPanel'
import { RewardOverlay } from './components/RewardOverlay'
import { RunBar } from './components/RunBar'
import { StatsPanel } from './components/StatsPanel'
import { rotate } from './game/labels'
import { useGame } from './game/useGame'
import type { Direction, MachineKind } from './game/types'

export function App() {
  const game = useGame()
  const [selected, setSelected] = useState<MachineKind>('Belt')
  const [direction, setDirection] = useState<Direction>('Right')
  const [newRunOpen, setNewRunOpen] = useState(false)

  const view = game.view
  const atlasItems = view?.atlas.items
  const items = useMemo(
    () => new Map((atlasItems ?? []).map((item) => [item.id, item])),
    [atlasItems],
  )

  const running = game.running
  const setRunning = game.setRunning
  const step = game.step

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.target instanceof HTMLInputElement) {
        return
      }
      if (event.key === 'r' || event.key === 'R') {
        setDirection(rotate)
      } else if (event.key === ' ') {
        event.preventDefault()
        setRunning(!running)
      } else if (event.key === 's' || event.key === 'S') {
        step()
      }
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [running, setRunning, step])

  if (view === null) {
    return <main className="boot">{game.notice ?? 'Starting the simulation\u2026'}</main>
  }

  return (
    <div className="app">
      <header className="topbar">
        <span className="brand">matlas</span>
        <RunBar
          ticks={game.ticks}
          running={game.running}
          speedIndex={game.speedIndex}
          onToggle={() => game.setRunning(!game.running)}
          onStep={game.step}
          onSpeed={game.setSpeedIndex}
          onNewRun={() => setNewRunOpen(true)}
        />
        {game.notice !== null && (
          <button type="button" className="notice" onClick={game.clearNotice}>
            {game.notice}
          </button>
        )}
      </header>

      <main className="columns">
        <div className="column">
          <GoalPanel goal={view.goal} />
          <Palette
            stock={view.machines.stock}
            selected={selected}
            direction={direction}
            onSelect={setSelected}
            onRotate={() => setDirection(rotate)}
          />
          <RelicPanel relics={view.relics} />
        </div>

        <Grid
          view={view}
          items={items}
          selected={selected}
          direction={direction}
          onPlace={(x, y) => game.place(selected, x, y, direction)}
          onRemove={game.remove}
        />

        <div className="column">
          <StatsPanel stats={view.stats} machines={view.machines} repo={view.repo} />
          <Codex atlas={view.atlas} repo={view.repo} />
        </div>
      </main>

      {view.offers.length > 0 && <RewardOverlay offers={view.offers} onClaim={game.claim} />}
      {newRunOpen && (
        <NewRunOverlay
          onStart={(config) => {
            game.startRun(config)
            setNewRunOpen(false)
          }}
          onClose={() => setNewRunOpen(false)}
        />
      )}
    </div>
  )
}
