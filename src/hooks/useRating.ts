import { useCallback, useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import type { LlmConfig, RatingsSnapshot } from '../types'

const isTauri = typeof window !== 'undefined' && '__TAURI__' in window

export function useRating() {
  const [ratings, setRatings] = useState<RatingsSnapshot | null>(null)
  const [error, setError] = useState<string | null>(null)

  const call = useCallback(async <T,>(cmd: string, args?: Record<string, unknown>) => {
    if (!isTauri) {
      setError('Run via Tauri to enable game commands.')
      return null
    }
    try {
      const result = await invoke<T>(cmd, args)
      setError(null)
      return result
    } catch (err) {
      setError(String(err))
      return null
    }
  }, [])

  const refreshRatings = useCallback(async () => {
    const snapshot = await call<RatingsSnapshot>('get_ratings')
    if (snapshot) {
      setRatings(snapshot)
    }
  }, [call])

  const setAutoMatch = useCallback(
    async (autoMatch: boolean, matchOffset: number) => {
      const snapshot = await call<RatingsSnapshot>('set_match_mode', {
        autoMatch,
        matchOffset,
      })
      if (snapshot) {
        setRatings(snapshot)
      }
    },
    [call],
  )

  const setActiveProfile = useCallback(
    async (id: string) => {
      const snapshot = await call<RatingsSnapshot>('set_active_profile', { id })
      if (snapshot) {
        setRatings(snapshot)
      }
    },
    [call],
  )

  const createLlmProfile = useCallback(
    async (name: string, config: LlmConfig, apiKey: string) => {
      const snapshot = await call<RatingsSnapshot>('create_llm_profile', {
        name,
        config,
        apiKey,
      })
      if (snapshot) {
        setRatings(snapshot)
        return true
      }
      return false
    },
    [call],
  )

  const updateLlmProfile = useCallback(
    async (id: string, name: string, config: LlmConfig, apiKey: string | null) => {
      const snapshot = await call<RatingsSnapshot>('update_llm_profile', {
        id,
        name,
        config,
        apiKey,
      })
      if (snapshot) {
        setRatings(snapshot)
        return true
      }
      return false
    },
    [call],
  )

  const deleteLlmProfile = useCallback(
    async (id: string, deleteKey: boolean) => {
      const snapshot = await call<RatingsSnapshot>('delete_llm_profile', {
        id,
        deleteKey,
      })
      if (snapshot) {
        setRatings(snapshot)
        return true
      }
      return false
    },
    [call],
  )

  return {
    ratings,
    error,
    refreshRatings,
    setAutoMatch,
    setActiveProfile,
    createLlmProfile,
    updateLlmProfile,
    deleteLlmProfile,
  }
}
