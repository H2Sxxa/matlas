import init, { GameHandle as WasmGameHandle } from '../wasm/pkg/matlas_wasm.js'
import type { Direction, GameView, MachineKind, Reward } from './types'

// The typed shape of the wasm handle. The generated bindings return `any`, so this
// is where the boundary gets a real signature and the rest of the app stays typed.
export interface GameHandle {
  // A u64 on the Rust side, so the counter arrives as a bigint.
  readonly ticks: bigint
  view(): GameView
  step(): void
  advance(steps: number): void
  place(kind: MachineKind, x: number, y: number, direction: Direction): void
  remove(x: number, y: number): MachineKind | null
  claim(index: number): Reward
  // Saves travel as JSON text so the front end can store them as they are.
  save(): string
}

export interface GameHandleFactory {
  new (seed: number, width: number, height: number, extraSlots: number): GameHandle
  load(value: string): GameHandle
}

let pending: Promise<GameHandleFactory> | undefined

export function loadGameApi(): Promise<GameHandleFactory> {
  pending ??= init().then(() => WasmGameHandle as unknown as GameHandleFactory)
  return pending
}
