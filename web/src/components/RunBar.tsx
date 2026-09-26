import { SPEEDS } from '../game/useGame'

interface RunBarProps {
  ticks: number
  running: boolean
  speedIndex: number
  onToggle(): void
  onStep(): void
  onSpeed(index: number): void
  onNewRun(): void
}

export function RunBar({
  ticks,
  running,
  speedIndex,
  onToggle,
  onStep,
  onSpeed,
  onNewRun,
}: RunBarProps) {
  return (
    <div className="runbar">
      <button type="button" className="primary-button" onClick={onToggle} title="Space">
        {running ? 'Pause' : 'Play'}
      </button>
      <button type="button" className="ghost-button" onClick={onStep} title="Step (S)">
        Step
      </button>
      <div className="speeds" role="group" aria-label="Speed">
        {SPEEDS.map((speed, index) => (
          <button
            key={speed}
            type="button"
            className={index === speedIndex ? 'speed is-active' : 'speed'}
            onClick={() => onSpeed(index)}
          >
            {speed}/s
          </button>
        ))}
      </div>
      <span className="ticks" title="Simulation ticks since the run started">
        T+{ticks}
      </span>
      <button type="button" className="ghost-button" onClick={onNewRun}>
        New run
      </button>
    </div>
  )
}
