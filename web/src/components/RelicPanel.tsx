import { STREAM_LABEL } from '../game/labels'
import type { RelicView } from '../game/types'

export function RelicPanel({ relics }: { relics: RelicView[] }) {
  return (
    <section className="panel">
      <header className="panel-head">
        <h2>Relics</h2>
        <span className="muted">{relics.length}</span>
      </header>
      {relics.length === 0 ? (
        <p className="hint">No relics yet. Loot hands them out as rewards.</p>
      ) : (
        <ul className="relics">
          {relics.map((relic, index) => (
            <li key={index} className="relic">
              <span>{relic.name}</span>
              <span className="muted">
                +{Math.round(relic.luck * 100)}% {STREAM_LABEL[relic.target]}
              </span>
            </li>
          ))}
        </ul>
      )}
    </section>
  )
}
