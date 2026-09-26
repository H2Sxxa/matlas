// Smoke test for the wasm boundary.
//
// The Rust tests cover the simulation and the view projection, but they cannot see
// what actually crosses into JavaScript. This runs the real module in Node, plays a
// short run through the same calls the UI makes, and asserts on the objects that
// come back, so a change to the bridge cannot silently break the front end.

import { readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import init, { GameHandle } from '../src/wasm/pkg/matlas_wasm.js'

const here = dirname(fileURLToPath(import.meta.url))
const wasm = await readFile(join(here, '..', 'src', 'wasm', 'pkg', 'matlas_wasm_bg.wasm'))
await init({ module_or_path: wasm })

let checks = 0
function assert(condition, message) {
  checks += 1
  if (!condition) {
    throw new Error(`check ${checks} failed: ${message}`)
  }
}

function throwsCode(action, code) {
  try {
    action()
  } catch (error) {
    assert(error.code === code, `expected code ${code}, got ${JSON.stringify(error)}`)
    return
  }
  throw new Error(`expected the call to fail with ${code}`)
}

const game = new GameHandle(7, 12, 8, 6)

const start = game.view()
assert(start.size[0] === 12 && start.size[1] === 8, 'the grid takes the requested size')
assert(start.nodes.length === 0, 'a new run starts on an empty grid')
assert(game.ticks === 0n, 'the tick counter starts at zero')

const kit = new Map(start.machines.stock.map((entry) => [entry.kind, entry.count]))
assert(kit.get('Generator') === 2, 'the kit grants generators')
assert(kit.get('Belt') === 4, 'the kit grants belts')
assert(kit.get('Mixer') === 1, 'the kit grants a mixer')
assert(kit.get('Inbound') === 1, 'the kit grants an inbound')
assert(kit.get('Sell') === 1, 'the kit grants a sell')
assert(kit.get('Outbound') === undefined, 'outbound terminals are not loot')
assert(start.machines.capacity === 15, 'the kit plus the open slots make the capacity')
assert(start.machines.free_slots === 6, 'the open slots stay free for loot')
assert(start.goal.progress.target > 0, 'the first objective is already drawn')

// (0,0) Gv
// (0,1) M>  (1,1) B>  (2,1) I
// (0,2) G^
for (const [kind, x, y, direction] of [
  ['Generator', 0, 0, 'Down'],
  ['Mixer', 0, 1, 'Right'],
  ['Belt', 1, 1, 'Right'],
  ['Inbound', 2, 1, 'Right'],
  ['Generator', 0, 2, 'Up'],
]) {
  game.place(kind, x, y, direction)
}

const placed = game.view()
assert(placed.nodes.length === 5, 'every placed machine shows up in the view')
const mixer = placed.nodes.find((node) => node.kind === 'Mixer')
assert(mixer.pos.x === 0 && mixer.pos.y === 1, 'positions survive the bridge')
assert(mixer.direction === 'Right', 'directions survive the bridge as names')
assert(Array.isArray(mixer.slots) && mixer.slots.length === 2, 'mixer slots cross as an array')
assert(mixer.input_capacity === 2, 'an empty mixer has both inputs free')

game.advance(2)
const running = game.view()
assert(game.ticks === 2n, 'advance counts the steps it ran')
assert(running.stats.crafted === 1, 'the mixer crafted an item')
const belt = running.nodes.find((node) => node.kind === 'Belt')
assert(belt.held === 1, 'the belt carries the crafted item')
assert(
  running.atlas.items.some((item) => item.id === 1 && item.level >= 1),
  'the crafted item is in the codex with its level',
)
assert(running.atlas.recipes.length === 1, 'the mix is recorded as a recipe')

// Loot only opens a choice once an objective lands, which needs a working factory.
for (let attempt = 0; attempt < 20 && game.view().offers.length === 0; attempt += 1) {
  game.advance(200)
}
const rewarded = game.view()
assert(rewarded.offers.length === 3, 'a finished objective offers three rewards')
assert(
  rewarded.offers.every((offer) => typeof offer.rarity === 'string'),
  'rarity crosses as a name',
)

const claimed = game.view().offers[0]
game.claim(0)
const afterClaim = game.view()
assert(afterClaim.offers.length === 0, 'claiming clears the pending offers')
assert(
  afterClaim.goal.completed > rewarded.goal.completed,
  'claiming draws the next objective',
)
assert(claimed.reward !== undefined, 'the claim reports what was applied')

throwsCode(() => game.claim(9), 'no_such_offer')
throwsCode(() => game.place('Nope', 0, 0, 'Right'), 'unknown_variant')
throwsCode(() => game.place('Belt', 0, 0, 'Sideways'), 'unknown_variant')
throwsCode(() => game.place('Belt', 99, 0, 'Right'), 'out_of_bounds')
throwsCode(() => game.place('Overflow', 3, 3, 'Right'), 'machine_not_in_stock')

// Autosave stores the save as text in localStorage, so it has to be plain JSON.
const save = game.save()
assert(typeof save === 'string', 'a save is JSON text')
assert(JSON.parse(save).game !== undefined, 'a save carries the run')
const restored = GameHandle.load(save)
const reloaded = restored.view()
assert(restored.ticks === game.ticks, 'a save keeps the tick counter')
assert(
  JSON.stringify(reloaded) === JSON.stringify(game.view()),
  'a restored run matches the run it was saved from',
)

console.log(`smoke: ${checks} checks passed`)
