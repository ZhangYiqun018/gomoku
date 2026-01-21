import { useCallback, useEffect, useRef, useState } from 'react'

export type AutoPlaySpeed = 'slow' | 'medium' | 'fast'

const SPEED_INTERVALS: Record<AutoPlaySpeed, number> = {
  slow: 2000,
  medium: 1000,
  fast: 500,
}

type UseAutoPlayOptions = {
  onStep: () => Promise<void>
  isGameOver: boolean
  isAiVsAi: boolean
}

export function useAutoPlay({ onStep, isGameOver, isAiVsAi }: UseAutoPlayOptions) {
  const [isPlaying, setIsPlaying] = useState(false)
  const [speed, setSpeed] = useState<AutoPlaySpeed>('medium')
  const intervalRef = useRef<number | null>(null)
  const steppingRef = useRef(false)

  const clearAutoPlay = useCallback(() => {
    if (intervalRef.current !== null) {
      clearInterval(intervalRef.current)
      intervalRef.current = null
    }
  }, [])

  const step = useCallback(async () => {
    if (steppingRef.current || isGameOver) return
    steppingRef.current = true
    try {
      await onStep()
    } finally {
      steppingRef.current = false
    }
  }, [onStep, isGameOver])

  const start = useCallback(() => {
    if (!isAiVsAi || isGameOver) return
    setIsPlaying(true)
  }, [isAiVsAi, isGameOver])

  const stop = useCallback(() => {
    setIsPlaying(false)
    clearAutoPlay()
  }, [clearAutoPlay])

  // Auto-step on interval when playing
  useEffect(() => {
    if (!isPlaying || isGameOver || !isAiVsAi) {
      clearAutoPlay()
      if (isGameOver) {
        setIsPlaying(false)
      }
      return
    }

    const doStep = async () => {
      await step()
    }

    // Run first step immediately
    void doStep()

    // Then set up interval for subsequent steps
    const interval = SPEED_INTERVALS[speed]
    intervalRef.current = window.setInterval(() => {
      void doStep()
    }, interval)

    return () => {
      clearAutoPlay()
    }
  }, [isPlaying, isGameOver, isAiVsAi, speed, step, clearAutoPlay])

  // Reset when game mode changes
  useEffect(() => {
    if (!isAiVsAi) {
      setIsPlaying(false)
      clearAutoPlay()
    }
  }, [isAiVsAi, clearAutoPlay])

  return {
    isPlaying,
    speed,
    setSpeed,
    start,
    stop,
    step,
  }
}
