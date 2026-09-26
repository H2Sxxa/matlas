import { useCallback, useEffect, useRef, useState } from 'react'
import { loadGameApi, type GameHandle, type GameHandleFactory } from './api'
import type { ApiError, Direction, GameView, MachineKind } from './types'

// The shape of a run. The seed is what makes a run reproducible: the same seed and
// the same placements always play out the same way.
export interface RunConfig {
  seed: number
  width: number
  height: number
  extraSlots: number
}

export const DEFAULT_RUN: RunConfig = {
  seed: 7,
  width: 12,
  height: 8,
  // Machines loot may still hand out beyond the starting kit.
  extraSlots: 6,
}

const SAVE_KEY = 'matlas.run'

// Ticks per second. The simulation steps are fixed, so speed only decides how many
// of them a second of real time is worth.
export const SPEEDS: readonly number[] = [1, 3, 8, 20]

// How often the grid is rebuilt from the simulation. Items move one cell per tick,
// so refreshing much faster than this only burns frames.
const RENDER_INTERVAL_MS = 60

// A tab that was hidden can ask for a large catch-up. Cap it so the page survives.
const MAX_STEPS_PER_FRAME = 240

export interface Game {
  view: GameView | null
  ticks: number
  ready: boolean
  notice: string | null
  running: boolean
  speedIndex: number
  setRunning(running: boolean): void
  setSpeedIndex(index: number): void
  place(kind: MachineKind, x: number, y: number, direction: Direction): void
  remove(x: number, y: number): void
  claim(index: number): void
  step(): void
  startRun(config: RunConfig): void
  clearNotice(): void
}

function describeError(error: unknown): string {
  if (typeof error === 'object' && error !== null && 'message' in error) {
    return String((error as ApiError).message)
  }
  return String(error)
}

function store(handle: GameHandle): void {
  try {
    localStorage.setItem(SAVE_KEY, handle.save())
  } catch (error: unknown) {
    console.warn('the run cannot be stored', error)
  }
}

function restore(factory: GameHandleFactory): GameHandle | null {
  const stored = localStorage.getItem(SAVE_KEY)
  if (stored === null) {
    return null
  }
  try {
    return factory.load(stored)
  } catch (error: unknown) {
    console.warn('the stored run cannot be restored', error)
    localStorage.removeItem(SAVE_KEY)
    return null
  }
}

export function useGame(): Game {
  const [view, setView] = useState<GameView | null>(null)
  const [ticks, setTicks] = useState(0)
  const [ready, setReady] = useState(false)
  const [notice, setNotice] = useState<string | null>(null)
  const [running, setRunning] = useState(false)
  const [speedIndex, setSpeedIndex] = useState(1)

  const factoryRef = useRef<GameHandleFactory | null>(null)
  const handleRef = useRef<GameHandle | null>(null)

  const refresh = useCallback(() => {
    const handle = handleRef.current
    if (handle === null) {
      return
    }
    setView(handle.view())
    setTicks(Number(handle.ticks))
  }, [])

  const startRun = useCallback(
    (config: RunConfig) => {
      const factory = factoryRef.current
      if (factory === null) {
        return
      }
      const handle = new factory(config.seed, config.width, config.height, config.extraSlots)
      handleRef.current = handle
      setRunning(false)
      setNotice(null)
      store(handle)
      refresh()
    },
    [refresh],
  )

  useEffect(() => {
    let cancelled = false
    loadGameApi()
      .then((factory) => {
        if (cancelled) {
          return
        }
        factoryRef.current = factory
        const restored = restore(factory)
        const handle =
          restored ??
          new factory(DEFAULT_RUN.seed, DEFAULT_RUN.width, DEFAULT_RUN.height, DEFAULT_RUN.extraSlots)
        handleRef.current = handle
        if (restored === null) {
          store(handle)
        } else {
          setNotice('Loaded the stored run')
        }
        setReady(true)
        refresh()
      })
      .catch((error: unknown) => {
        if (cancelled) {
          return
        }
        setNotice(`The simulation cannot start: ${describeError(error)}`)
      })
    return () => {
      cancelled = true
    }
  }, [refresh])

  useEffect(() => {
    const handle = handleRef.current
    if (handle === null) {
      return
    }
    const onUnload = () => store(handle)
    window.addEventListener('beforeunload', onUnload)
    return () => window.removeEventListener('beforeunload', onUnload)
  }, [ready])

  useEffect(() => {
    if (!running) {
      return
    }
    const speed = SPEEDS[speedIndex] ?? SPEEDS[0]
    let frame = 0
    let last = performance.now()
    let carry = 0
    let lastRender = last

    const pump = (now: number) => {
      const handle = handleRef.current
      if (handle === null) {
        return
      }
      const elapsed = (now - last) / 1000
      last = now

      carry += elapsed * speed
      const steps = Math.min(Math.floor(carry), MAX_STEPS_PER_FRAME)
      if (steps > 0) {
        carry -= steps
        try {
          handle.advance(steps)
        } catch (error: unknown) {
          setNotice(describeError(error))
          setRunning(false)
          return
        }
      }

      if (now - lastRender < RENDER_INTERVAL_MS) {
        frame = requestAnimationFrame(pump)
        return
      }
      lastRender = now
      const next = handle.view()
      setView(next)
      setTicks(Number(handle.ticks))

      // A finished goal opens a reward choice. Stopping here keeps the choice from
      // rolling past while the player is not looking at it.
      if (next.offers.length > 0) {
        setRunning(false)
        store(handle)
        return
      }
      frame = requestAnimationFrame(pump)
    }

    frame = requestAnimationFrame(pump)
    return () => cancelAnimationFrame(frame)
  }, [running, speedIndex])

  const attempt = useCallback(
    (action: (handle: GameHandle) => void) => {
      const handle = handleRef.current
      if (handle === null) {
        return
      }
      try {
        action(handle)
        setNotice(null)
      } catch (error: unknown) {
        setNotice(describeError(error))
      }
      store(handle)
      refresh()
    },
    [refresh],
  )

  const place = useCallback(
    (kind: MachineKind, x: number, y: number, direction: Direction) => {
      attempt((handle) => handle.place(kind, x, y, direction))
    },
    [attempt],
  )

  const remove = useCallback(
    (x: number, y: number) => {
      attempt((handle) => handle.remove(x, y))
    },
    [attempt],
  )

  const claim = useCallback(
    (index: number) => {
      attempt((handle) => handle.claim(index))
    },
    [attempt],
  )

  const step = useCallback(() => {
    attempt((handle) => handle.step())
  }, [attempt])

  const clearNotice = useCallback(() => setNotice(null), [])

  return {
    view,
    ticks,
    ready,
    notice,
    running,
    speedIndex,
    setRunning,
    setSpeedIndex,
    place,
    remove,
    claim,
    step,
    startRun,
    clearNotice,
  }
}
