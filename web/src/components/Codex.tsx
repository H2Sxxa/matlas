import { itemColor, itemInitial } from '../game/labels'
import type { AtlasView, RepoEntry } from '../game/types'

interface CodexProps {
  atlas: AtlasView
  repo: RepoEntry[]
}

export function Codex({ atlas, repo }: CodexProps) {
  const counts = new Map(repo.map((entry) => [entry.item, entry.count]))
  const byId = new Map(atlas.items.map((item) => [item.id, item]))
  const items = [...atlas.items].sort((left, right) => right.level - left.level || left.id - right.id)
  const nameOf = (id: number) => byId.get(id)?.name ?? `Item #${id}`

  return (
    <section className="panel panel-scroll">
      <header className="panel-head">
        <h2>Codex</h2>
        <span className="muted">{items.length} items</span>
      </header>
      <ul className="codex">
        {items.map((item) => (
          <li key={item.id} className="codex-row">
            <span className="codex-dot" style={{ background: itemColor(item.id) }}>
              {itemInitial(item.name)}
            </span>
            <span className="codex-name">{item.name}</span>
            <span className="muted">Lv.{item.level}</span>
            <span className="codex-value">{item.value}</span>
            <span className="muted">{counts.get(item.id) ?? 0} stored</span>
          </li>
        ))}
      </ul>
      <header className="panel-head">
        <h2>Recipes</h2>
        <span className="muted">{atlas.recipes.length} found</span>
      </header>
      <ul className="codex">
        {atlas.recipes.map((recipe, index) => (
          <li key={index} className="codex-row">
            <span className="codex-recipe">
              {nameOf(recipe.left)} + {nameOf(recipe.right)}
            </span>
            <span className="muted">{'\u2192'}</span>
            <span className="codex-name">{nameOf(recipe.out)}</span>
          </li>
        ))}
      </ul>
    </section>
  )
}
