import { goalText } from '../game/labels'
import type { GoalView } from '../game/types'

export function GoalPanel({ goal }: { goal: GoalView }) {
  const ratio = goal.progress.target === 0 ? 1 : goal.progress.current / goal.progress.target
  const filled = Math.min(1, Math.max(0, ratio))

  return (
    <section className="panel">
      <header className="panel-head">
        <h2>Objective</h2>
        <span className="muted">{goal.completed} done</span>
      </header>
      <p className="goal-text">{goalText(goal.kind)}</p>
      <div className="bar">
        <div className="bar-fill" style={{ width: `${(filled * 100).toFixed(1)}%` }} />
      </div>
      <p className="muted">
        {Math.min(goal.progress.current, goal.progress.target)} / {goal.progress.target}
      </p>
    </section>
  )
}
