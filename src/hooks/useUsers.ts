import { useCallback, useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import type { UsersSnapshot } from '../types'

const isTauri = typeof window !== 'undefined' && '__TAURI__' in window

export function useUsers() {
  const [users, setUsers] = useState<UsersSnapshot | null>(null)
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

  const refreshUsers = useCallback(async () => {
    const snapshot = await call<UsersSnapshot>('get_users')
    if (snapshot) {
      setUsers(snapshot)
    }
  }, [call])

  const createUser = useCallback(
    async (name: string) => {
      const snapshot = await call<UsersSnapshot>('create_user', { name })
      if (snapshot) {
        setUsers(snapshot)
        return true
      }
      return false
    },
    [call],
  )

  const switchUser = useCallback(
    async (id: string) => {
      const snapshot = await call<UsersSnapshot>('set_active_user', { id })
      if (snapshot) {
        setUsers(snapshot)
        return true
      }
      return false
    },
    [call],
  )

  const deleteUser = useCallback(
    async (id: string, deleteData: boolean) => {
      const snapshot = await call<UsersSnapshot>('delete_user', { id, deleteData })
      if (snapshot) {
        setUsers(snapshot)
        return true
      }
      return false
    },
    [call],
  )

  const updateUser = useCallback(
    async (id: string, name: string) => {
      const snapshot = await call<UsersSnapshot>('update_user', { id, name })
      if (snapshot) {
        setUsers(snapshot)
        return true
      }
      return false
    },
    [call],
  )

  const activeUser = users ? users.users.find((u) => u.id === users.activeUser) : null
  const activeUserDir = activeUser?.dataDir ?? null

  return {
    users,
    activeUser,
    activeUserDir,
    error,
    refreshUsers,
    createUser,
    switchUser,
    deleteUser,
    updateUser,
  }
}
