import { useMemo, type ReactNode } from 'react'
import {
  DIRECTION_ANGLE,
  NODE_GLYPH,
  NODE_HINT,
  NODE_LABEL,
  itemColor,
  itemInitial,
  itemName,
} from '../game/labels'
import type { Direction, GameView, ItemInfo, MachineKind, NodeView } from '../game/types'

interface GridProps {
  view: GameView
  items: Map<number, ItemInfo>
  selected: MachineKind
  direction: Direction
  onPlace(x: number, y: number): void
  onRemove(x: number, y: number): void
}

export function Grid({ view, items, selected, direction, onPlace, onRemove }: GridProps) {
  const [width, height] = view.size
  const placed = useMemo(() => {
    const index = new Map<string, NodeView>()
    for (const node of view.nodes) {
      index.set(`${node.pos.x},${node.pos.y}`, node)
    }
    return index
  }, [view.nodes])

  const cells: ReactNode[] = []
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const node = placed.get(`${x},${y}`)
      cells.push(
        <button
          key={`${x},${y}`}
          type="button"
          className={node === undefined ? 'cell cell-empty' : 'cell cell-filled'}
          data-kind={node?.kind}
          title={node === undefined ? `Empty cell ${x},${y}` : describe(node, items)}
          onClick={() => onPlace(x, y)}
          onContextMenu={(event) => {
            event.preventDefault()
            onRemove(x, y)
          }}
        >
          {node === undefined ? (
            <Ghost kind={selected} direction={direction} />
          ) : (
            <Machine node={node} items={items} />
          )}
        </button>,
      )
    }
  }

  return (
    <div className="grid-scroll">
      <div
        className="grid"
        style={{ gridTemplateColumns: `repeat(${width}, var(--cell-size))` }}
      >
        {cells}
      </div>
    </div>
  )
}

function Machine({ node, items }: { node: NodeView; items: Map<number, ItemInfo> }) {
  return (
    <>
      <span className="cell-glyph">{NODE_GLYPH[node.kind]}</span>
      <span
        className="cell-arrow"
        style={{ transform: `rotate(${DIRECTION_ANGLE[node.direction]}deg)` }}
      >
        {'\u2192'}
      </span>
      {node.kind === 'Mixer' && (
        <span className="cell-slots">
          {node.slots.map((slot, index) => (
            <span
              key={index}
              className={slot === null ? 'cell-slot cell-slot-empty' : 'cell-slot'}
              style={slot === null ? undefined : { background: itemColor(slot) }}
            />
          ))}
        </span>
      )}
      {node.held !== null && (
        <span className="cell-item" style={{ background: itemColor(node.held) }}>
          {itemInitial(itemName(node.held, items))}
        </span>
      )}
      {node.queued > 0 && <span className="cell-badge">{node.queued}</span>}
    </>
  )
}

function Ghost({ kind, direction }: { kind: MachineKind; direction: Direction }) {
  return (
    <span className="cell-ghost">
      <span className="cell-glyph">{NODE_GLYPH[kind]}</span>
      <span
        className="cell-arrow"
        style={{ transform: `rotate(${DIRECTION_ANGLE[direction]}deg)` }}
      >
        {'\u2192'}
      </span>
    </span>
  )
}

function describe(node: NodeView, items: Map<number, ItemInfo>): string {
  const lines = [`${NODE_LABEL[node.kind]} facing ${node.direction}`, NODE_HINT[node.kind]]
  if (node.held !== null) {
    lines.push(`Holding ${itemName(node.held, items)}`)
  }
  const filled = node.slots.filter((slot) => slot !== null).length
  if (filled > 0) {
    lines.push(`Inputs: ${filled}/${node.slots.length}`)
  }
  if (node.queued > 0) {
    lines.push(`Queued: ${node.queued}`)
  }
  return lines.join('\n')
}
