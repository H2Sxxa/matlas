# matlas

A factory-building simulation, with the tools to play it in a browser.

## Layout

- `src/` — the simulation: nodes, the grid, loot, relics and the goal chain. Pure
  Rust, no rendering, no platform assumptions.
- `crates/matlas-wasm/` — the WebAssembly bridge. It hands out read-only views and
  takes player intents, so the front end never reaches into the simulation.
- `web/` — the React front end. It renders one view per frame and sends back
  placements, removals and reward choices.

The simulation is the source of truth. `GameView` in `src/view.rs` is the contract
between the two sides, and `web/src/game/types.ts` mirrors it.

## Playing

```sh
cd web
pnpm install
pnpm dev
```

Click a machine in the palette, aim it with `R`, then click a cell to place it.
Right-click removes a machine and returns it to the stock. Space plays and pauses,
`S` steps one tick.

## Checks

```sh
cargo test          # the simulation and the view projection
cd web && pnpm typecheck
cd web && pnpm smoke   # plays a short run through the wasm boundary in Node
```

## Design

Blog: https://h2sxxa.github.io/blog/about_factory_game_design/

## Deploying

Pushing to `main` builds the wasm package and the site, then publishes `web/dist`
through `.github/workflows/pages.yml`. Enable Pages once, with GitHub Actions as
the source.

GitHub Pages serves a project site from `/<repository>/`, so `base` in
`web/vite.config.ts` has to match the repository name. It is `/matlas/` today.
