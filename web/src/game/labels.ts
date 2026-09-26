import type {
  Direction,
  GoalKind,
  ItemInfo,
  NodeKind,
  RandomType,
  Rarity,
  Reward,
} from './types'

export const NODE_LABEL: Record<NodeKind, string> = {
  Generator: 'Generator',
  Belt: 'Belt',
  Mixer: 'Mixer',
  Inbound: 'Inbound',
  Outbound: 'Outbound',
  Distributor: 'Distributor',
  Overflow: 'Overflow',
  Sell: 'Sell',
}

export const NODE_GLYPH: Record<NodeKind, string> = {
  Generator: 'G',
  Belt: 'B',
  Mixer: 'M',
  Inbound: 'I',
  Outbound: 'O',
  Distributor: 'D',
  Overflow: 'V',
  Sell: '$',
}

export const NODE_HINT: Record<NodeKind, string> = {
  Generator: 'Emits one base item every tick while its output is empty',
  Belt: 'Carries a single item one cell per tick',
  Mixer: 'Combines two items into a new discovery',
  Inbound: 'Stores everything it receives in the repository',
  Outbound: 'Draws a chosen item back out of the repository',
  Distributor: 'Rotates through forward, left and right',
  Overflow: 'Prefers forward, then alternates the sides',
  Sell: 'Turns items into revenue',
}

export const DIRECTION_ANGLE: Record<Direction, number> = {
  Right: 0,
  Down: 90,
  Left: 180,
  Up: -90,
}

// Click order for the rotate button, so a machine walks around the compass.
export const DIRECTION_ORDER: Direction[] = ['Right', 'Down', 'Left', 'Up']

export const DIRECTION_ARROW: Record<Direction, string> = {
  Right: '\u2192',
  Down: '\u2193',
  Left: '\u2190',
  Up: '\u2191',
}

export const RARITY_LABEL: Record<Rarity, string> = {
  Common: 'Common',
  Uncommon: 'Uncommon',
  Rare: 'Rare',
}

export const STREAM_LABEL: Record<RandomType, string> = {
  Generic: 'discovery',
  Loot: 'loot',
  Discover: 'mixing',
  Sell: 'sales',
}

export function rotate(direction: Direction): Direction {
  const index = DIRECTION_ORDER.indexOf(direction)
  return DIRECTION_ORDER[(index + 1) % DIRECTION_ORDER.length]
}

export function goalText(kind: GoalKind): string {
  if ('CraftLevel' in kind) {
    return `Craft a Lv.${kind.CraftLevel.level} item`
  }
  if ('CraftValue' in kind) {
    return `Craft an item worth ${kind.CraftValue.value}`
  }
  if ('DiscoverCount' in kind) {
    return `Discover ${kind.DiscoverCount.count} items`
  }
  if ('Revenue' in kind) {
    return `Ship items worth ${kind.Revenue.value}`
  }
  return `Craft ${kind.CraftCount.count} items`
}

export function rewardText(reward: Reward): string {
  if ('Relic' in reward) {
    return `+${Math.round(reward.Relic.luck * 100)}% ${STREAM_LABEL[reward.Relic.target]} luck`
  }
  if ('Machine' in reward) {
    return `A ${NODE_LABEL[reward.Machine]} for the stock`
  }
  const { width, height } = reward.Graph
  const axis = width > 0 ? `${width} wide` : `${height} tall`
  return `Grows the factory ${axis}`
}

// A stable colour per item id, so the same item reads the same in the grid, the
// repository and the codex.
export function itemColor(id: number): string {
  const hue = (id * 137.508) % 360
  const saturation = 58 + ((id * 7) % 18)
  const lightness = 52 + ((id * 13) % 12)
  return `hsl(${hue.toFixed(1)} ${saturation}% ${lightness}%)`
}

export function itemInitial(name: string): string {
  return name.slice(0, 1).toUpperCase()
}

export function itemName(id: number, items: Map<number, ItemInfo>): string {
  return items.get(id)?.name ?? `Item #${id}`
}
