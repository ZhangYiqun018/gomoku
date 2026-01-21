import { useCallback, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import type { GameMode, GameSnapshot } from '../types'
import { defaultGameMode, emptyBoard } from '../types'

const defaultSnapshot: GameSnapshot = {
  boardSize: 15,
  board: emptyBoard(15),
  ruleSet: 'standard',
  toMove: 'B',
  result: null,
  moves: [],
  mode: defaultGameMode,
  canHumanMove: true,
}

const isTauri = typeof window !== 'undefined' && '__TAURI__' in window

export function useGame() {
  const [game, setGame] = useState<GameSnapshot>(defaultSnapshot)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [canRetryAi, setCanRetryAi] = useState(false)
  const audioRef = useRef<AudioContext | null>(null)
  const noiseRef = useRef<AudioBuffer | null>(null)

  const call = useCallback(async <T,>(cmd: string, args?: Record<string, unknown>) => {
    if (!isTauri) {
      setError('Run via Tauri to enable game commands.')
      setCanRetryAi(false)
      return null
    }
    try {
      const result = await invoke<T>(cmd, args)
      setError(null)
      setCanRetryAi(false)
      return result
    } catch (err) {
      setError(String(err))
      setCanRetryAi(cmd === 'ai_move')
      return null
    }
  }, [])

  const playStoneSound = useCallback(() => {
    if (typeof window === 'undefined') return
    const AudioCtor =
      window.AudioContext ||
      (window as typeof window & { webkitAudioContext?: typeof AudioContext }).webkitAudioContext
    if (!AudioCtor) return
    if (!audioRef.current) {
      audioRef.current = new AudioCtor()
    }
    const ctx = audioRef.current
    if (ctx.state === 'suspended') {
      void ctx.resume()
    }
    const now = ctx.currentTime
    if (!noiseRef.current) {
      const length = Math.floor(ctx.sampleRate * 0.08)
      const buffer = ctx.createBuffer(1, length, ctx.sampleRate)
      const data = buffer.getChannelData(0)
      for (let i = 0; i < length; i += 1) {
        data[i] = (Math.random() * 2 - 1) * 0.6
      }
      noiseRef.current = buffer
    }

    const noise = ctx.createBufferSource()
    noise.buffer = noiseRef.current
    const bandpass = ctx.createBiquadFilter()
    bandpass.type = 'bandpass'
    bandpass.frequency.setValueAtTime(2600, now)
    bandpass.Q.setValueAtTime(0.9, now)
    const clickGain = ctx.createGain()
    clickGain.gain.setValueAtTime(0.0001, now)
    clickGain.gain.exponentialRampToValueAtTime(0.3, now + 0.003)
    clickGain.gain.exponentialRampToValueAtTime(0.0001, now + 0.035)
    noise.connect(bandpass)
    bandpass.connect(clickGain)
    clickGain.connect(ctx.destination)
    noise.start(now)
    noise.stop(now + 0.045)
  }, [])

  const refreshState = useCallback(async () => {
    const snapshot = await call<GameSnapshot>('get_state')
    if (snapshot) {
      setGame(snapshot)
    }
  }, [call])

  const newGame = useCallback(
    async (mode?: GameMode) => {
      const snapshot = await call<GameSnapshot>('new_game', { ruleSet: 'standard', mode })
      if (snapshot) {
        setGame(snapshot)
      }
    },
    [call],
  )

  const makeMove = useCallback(
    async (x: number, y: number, onRatingsChanged?: () => Promise<void>, autoTriggerAi = true) => {
      if (busy || game.result) return
      if (!game.canHumanMove) return
      const index = y * game.boardSize + x
      if (game.board[index]) return

      setBusy(true)
      const snapshot = await call<GameSnapshot>('make_move', { x, y })
      if (snapshot) {
        setGame(snapshot)
        playStoneSound()

        // Only auto-trigger AI in human_vs_ai mode when AI should move next
        if (!snapshot.result && autoTriggerAi && snapshot.mode.type === 'human_vs_ai' && !snapshot.canHumanMove) {
          const aiSnapshot = await call<GameSnapshot>('ai_move')
          if (aiSnapshot) {
            setGame(aiSnapshot)
            playStoneSound()
            if (aiSnapshot.result) {
              await onRatingsChanged?.()
            }
          }
        } else if (snapshot.result) {
          await onRatingsChanged?.()
        }
      }
      setBusy(false)
    },
    [busy, game.result, game.boardSize, game.board, game.canHumanMove, call, playStoneSound],
  )

  const requestAiMove = useCallback(
    async (onRatingsChanged?: () => Promise<void>, force = false) => {
      // force=true bypasses game.result check (used for initial AI move after new game)
      if (busy || (!force && game.result)) return
      setBusy(true)
      const snapshot = await call<GameSnapshot>('ai_move')
      if (snapshot) {
        setGame(snapshot)
        playStoneSound()
        if (snapshot.result) {
          await onRatingsChanged?.()
        }
      }
      setBusy(false)
    },
    [busy, game.result, call, playStoneSound],
  )

  const saveGame = useCallback(
    async (path: string) => {
      await call('save_game', { path })
    },
    [call],
  )

  const loadGame = useCallback(
    async (path: string) => {
      const snapshot = await call<GameSnapshot>('load_game', { path })
      if (snapshot) {
        setGame(snapshot)
      }
    },
    [call],
  )

  const exportTraining = useCallback(
    async (path: string) => {
      await call('export_training', { path })
    },
    [call],
  )

  const clearError = useCallback(() => {
    setError(null)
    setCanRetryAi(false)
  }, [])

  return {
    game,
    busy,
    error,
    canRetryAi,
    refreshState,
    newGame,
    makeMove,
    requestAiMove,
    saveGame,
    loadGame,
    exportTraining,
    clearError,
  }
}
