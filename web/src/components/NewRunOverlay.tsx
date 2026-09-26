import { useState, type FormEvent } from 'react'
import { DEFAULT_RUN, type RunConfig } from '../game/useGame'

interface NewRunOverlayProps {
  onStart(config: RunConfig): void
  onClose(): void
}

export function NewRunOverlay({ onStart, onClose }: NewRunOverlayProps) {
  const [seed, setSeed] = useState(String(DEFAULT_RUN.seed))
  const [width, setWidth] = useState(String(DEFAULT_RUN.width))
  const [height, setHeight] = useState(String(DEFAULT_RUN.height))
  const [extraSlots, setExtraSlots] = useState(String(DEFAULT_RUN.extraSlots))

  const submit = (event: FormEvent) => {
    event.preventDefault()
    onStart({
      seed: clamp(seed, DEFAULT_RUN.seed, 0, 4_294_967_295),
      width: clamp(width, DEFAULT_RUN.width, 2, 64),
      height: clamp(height, DEFAULT_RUN.height, 2, 64),
      extraSlots: clamp(extraSlots, DEFAULT_RUN.extraSlots, 0, 64),
    })
  }

  return (
    <div className="overlay">
      <form className="overlay-card form" onSubmit={submit}>
        <h2>New run</h2>
        <p className="muted">
          A run starts with a fixed machine kit, so the grid is never empty. The seed decides
          every roll, so the same seed replays the same run.
        </p>
        <label className="field">
          <span>Seed</span>
          <input value={seed} onChange={(event) => setSeed(event.target.value)} inputMode="numeric" />
        </label>
        <label className="field">
          <span>Width</span>
          <input value={width} onChange={(event) => setWidth(event.target.value)} inputMode="numeric" />
        </label>
        <label className="field">
          <span>Height</span>
          <input
            value={height}
            onChange={(event) => setHeight(event.target.value)}
            inputMode="numeric"
          />
        </label>
        <label className="field">
          <span>Open slots</span>
          <input
            value={extraSlots}
            onChange={(event) => setExtraSlots(event.target.value)}
            inputMode="numeric"
          />
        </label>
        <div className="form-actions">
          <button
            type="button"
            className="ghost-button"
            onClick={() => setSeed(String(Math.floor(Math.random() * 4_294_967_296)))}
          >
            Random seed
          </button>
          <button type="button" className="ghost-button" onClick={onClose}>
            Cancel
          </button>
          <button type="submit" className="primary-button">
            Start
          </button>
        </div>
      </form>
    </div>
  )
}

// Clamps a text field into the range the simulation accepts. An out of range value is
// not something the player should have to fix by hand, so it is brought back instead.
function clamp(raw: string, fallback: number, min: number, max: number): number {
  const value = Number.parseInt(raw, 10)
  if (Number.isNaN(value)) {
    return fallback
  }
  return Math.min(max, Math.max(min, value))
}
