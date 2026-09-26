// Mirrors of the view types the simulation hands out over the wasm boundary.
//
// Names and casing follow the Rust structs exactly, because the bridge serialises
// them straight into objects. Keeping the mirror in one file is what stops the two
// sides from drifting.

export type Direction = 'Right' | 'Down' | 'Up' | 'Left'

export type NodeKind =
  | 'Generator'
  | 'Belt'
  | 'Mixer'
  | 'Inbound'
  | 'Outbound'
  | 'Distributor'
  | 'Overflow'
  | 'Sell'

// Outbound terminals are configured with the item they ship, so they are layout
// rather than loot and never appear in the machine stock.
export type MachineKind = Exclude<NodeKind, 'Outbound'>

export type RandomType = 'Generic' | 'Loot' | 'Discover' | 'Sell'

export type Rarity = 'Common' | 'Uncommon' | 'Rare'

export interface Pos {
  x: number
  y: number
}

export interface NodeView {
  pos: Pos
  kind: NodeKind
  direction: Direction
  held: number | null
  slots: (number | null)[]
  queued: number
  outputs: Direction[]
  input_capacity: number
}

export interface ItemInfo {
  id: number
  name: string
  level: number
  value: number
}

export interface RecipeInfo {
  left: number
  right: number
  out: number
}

export interface AtlasView {
  items: ItemInfo[]
  recipes: RecipeInfo[]
}

export interface RepoEntry {
  item: number
  count: number
}

export interface StatsView {
  crafted: number
  revenue: number
}

export interface Progress {
  current: number
  target: number
}

export type GoalKind =
  | { CraftLevel: { level: number } }
  | { CraftValue: { value: number } }
  | { DiscoverCount: { count: number } }
  | { Revenue: { value: number } }
  | { CraftCount: { count: number } }

export interface GoalView {
  kind: GoalKind
  progress: Progress
  completed: number
}

export interface RelicView {
  name: string
  target: RandomType
  luck: number
}

export interface StockEntry {
  kind: MachineKind
  count: number
}

export interface MachinesView {
  capacity: number
  placed: number
  free_slots: number
  stock: StockEntry[]
}

export type Reward =
  | { Relic: { name: string; target: RandomType; luck: number } }
  | { Machine: MachineKind }
  | { Graph: { width: number; height: number } }

export interface RewardOffer {
  rarity: Rarity
  reward: Reward
}

export interface GameView {
  size: [number, number]
  nodes: NodeView[]
  atlas: AtlasView
  repo: RepoEntry[]
  stats: StatsView
  goal: GoalView
  relics: RelicView[]
  machines: MachinesView
  offers: RewardOffer[]
}

// The structured error every wasm call throws instead of a bare string.
export interface ApiError {
  code: string
  message: string
}
