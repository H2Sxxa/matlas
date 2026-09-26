import { DIRECTION_ARROW, NODE_HINT, NODE_LABEL } from '../game/labels'
import type { Direction, MachineKind, StockEntry } from '../game/types'

interface PaletteProps {
  stock: StockEntry[]
  selected: MachineKind
  direction: Direction
  onSelect(kind: MachineKind): void
  onRotate(): void
}

export function Palette({ stock, selected, direction, onSelect, onRotate }: PaletteProps) {
  return (
    <section className="panel">
      <header className="panel-head">
        <h2>Machines</h2>
        <button type="button" className="ghost-button" onClick={onRotate} title="Rotate (R)">
          {DIRECTION_ARROW[direction]}
        </button>
      </header>
      <ul className="palette">
        {stock.map((entry) => (
          <li key={entry.kind}>
            <button
              type="button"
              className={entry.kind === selected ? 'palette-row is-selected' : 'palette-row'}
              disabled={entry.count === 0}
              onClick={() => onSelect(entry.kind)}
              title={NODE_HINT[entry.kind]}
            >
              <span className="palette-name">{NODE_LABEL[entry.kind]}</span>
              <span className="palette-count">{entry.count}</span>
            </button>
          </li>
        ))}
      </ul>
      <p className="hint">Click a cell to place, right-click to remove.</p>
    </section>
  )
}
