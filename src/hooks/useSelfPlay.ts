import { useCallback, useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import type { SelfPlayProgress, SelfPlayReport, ProfileRating } from '../types'

const isTauri = typeof window !== 'undefined' && '__TAURI__' in window

const formatDuration = (seconds: number) => {
  if (!Number.isFinite(seconds) || seconds <= 0) return '0s'
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  if (mins >= 60) {
    const hours = Math.floor(mins / 60)
    const remMins = mins % 60
    return `${hours}h ${String(remMins).padStart(2, '0')}m`
  }
  if (mins > 0) {
    return `${mins}m ${String(secs).padStart(2, '0')}s`
  }
  return `${secs}s`
}

export function useSelfPlay(onComplete: () => Promise<void>) {
  const [busy, setBusy] = useState(false)
  const [progress, setProgress] = useState<SelfPlayProgress | null>(null)
  const [report, setReport] = useState<SelfPlayReport | null>(null)
  const [eta, setEta] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  const [gamesPerPair, setGamesPerPair] = useState(30)
  const [parallelism, setParallelism] = useState(4)
  const [minLevel, setMinLevel] = useState(1)
  const [maxLevel, setMaxLevel] = useState(12)
  const [includeLlm, setIncludeLlm] = useState(false)
  const [llmIds, setLlmIds] = useState<string[]>([])

  const startRef = useRef<number | null>(null)

  useEffect(() => {
    if (!isTauri) return
    let unlistenProgress: (() => void) | null = null
    let unlistenDone: (() => void) | null = null
    let unlistenError: (() => void) | null = null

    listen<SelfPlayProgress>('self_play_progress', (event) => {
      setProgress(event.payload)
      if (!startRef.current) {
        startRef.current = Date.now()
      }
      if (event.payload.completed > 0 && event.payload.total > 0) {
        const elapsed = (Date.now() - startRef.current) / 1000
        const remaining = (event.payload.total - event.payload.completed) * (elapsed / event.payload.completed)
        setEta(formatDuration(remaining))
      } else {
        setEta(null)
      }
    }).then((fn) => {
      unlistenProgress = fn
    })

    listen<SelfPlayReport>('self_play_done', async (event) => {
      setReport(event.payload)
      setBusy(false)
      setProgress(null)
      startRef.current = null
      setEta(null)
      await onComplete()
    }).then((fn) => {
      unlistenDone = fn
    })

    listen<string>('self_play_error', (event) => {
      setError(event.payload)
      setBusy(false)
      setProgress(null)
      startRef.current = null
      setEta(null)
    }).then((fn) => {
      unlistenError = fn
    })

    return () => {
      unlistenProgress?.()
      unlistenDone?.()
      unlistenError?.()
    }
  }, [onComplete])

  const start = useCallback(async () => {
    if (busy) return
    setBusy(true)
    setReport(null)
    setProgress(null)
    startRef.current = Date.now()
    setEta(null)
    setError(null)
    try {
      const result = await invoke<boolean>('start_self_play', {
        gamesPerPair,
        parallelism,
        includeLlm,
        llmIds: includeLlm ? llmIds : [],
        minLevel,
        maxLevel,
      })
      if (result === null) {
        setBusy(false)
        startRef.current = null
      }
    } catch (err) {
      setError(String(err))
      setBusy(false)
      startRef.current = null
    }
  }, [busy, gamesPerPair, parallelism, includeLlm, llmIds, minLevel, maxLevel])

  const stop = useCallback(async () => {
    if (!busy) return
    try {
      await invoke('stop_self_play')
    } catch (err) {
      setError(String(err))
    }
  }, [busy])

  const toggleIncludeLlm = useCallback(
    (checked: boolean, profiles: ProfileRating[]) => {
      setIncludeLlm(checked)
      if (checked) {
        const defaults = profiles
          .filter((p) => p.kind === 'llm' && p.llm?.apiKeySet)
          .map((p) => p.id)
        setLlmIds(defaults)
      } else {
        setLlmIds([])
      }
    },
    [],
  )

  const toggleLlmId = useCallback((id: string, checked: boolean) => {
    setLlmIds((prev) => {
      if (checked) {
        if (prev.includes(id)) return prev
        return [...prev, id]
      }
      return prev.filter((item) => item !== id)
    })
  }, [])

  return {
    busy,
    progress,
    report,
    eta,
    error,
    gamesPerPair,
    parallelism,
    minLevel,
    maxLevel,
    includeLlm,
    llmIds,
    setGamesPerPair,
    setParallelism,
    setMinLevel,
    setMaxLevel,
    toggleIncludeLlm,
    toggleLlmId,
    start,
    stop,
  }
}
