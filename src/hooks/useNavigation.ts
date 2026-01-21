import { useCallback, useState } from 'react'

export type AppMode = 'welcome' | 'play' | 'settings'
export type SettingsPage = 'home' | 'profile' | 'ai' | 'data' | 'users'

export function useNavigation() {
  const [mode, setMode] = useState<AppMode>('welcome')
  const [settingsPage, setSettingsPage] = useState<SettingsPage>('home')

  const goToWelcome = useCallback(() => {
    setMode('welcome')
  }, [])

  const goToPlay = useCallback(() => {
    setMode('play')
  }, [])

  const goToSettings = useCallback(() => {
    setMode('settings')
    setSettingsPage('home')
  }, [])

  const goToSettingsPage = useCallback((page: SettingsPage) => {
    setMode('settings')
    setSettingsPage(page)
  }, [])

  const goBack = useCallback(() => {
    if (mode === 'settings' && settingsPage !== 'home') {
      setSettingsPage('home')
    } else if (mode === 'settings') {
      setMode('welcome')
    } else if (mode === 'play') {
      setMode('welcome')
    }
  }, [mode, settingsPage])

  return {
    mode,
    settingsPage,
    goToWelcome,
    goToPlay,
    goToSettings,
    goToSettingsPage,
    goBack,
  }
}
