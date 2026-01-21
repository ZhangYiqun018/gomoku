import React from 'react'
import type { AppMode } from '../../types'

type AppShellProps = {
  mode: AppMode
  welcomeContent: React.ReactNode
  playContent: React.ReactNode
  settingsContent: React.ReactNode
  errorBanner?: React.ReactNode
}

function AppShellComponent({ mode, welcomeContent, playContent, settingsContent, errorBanner }: AppShellProps) {
  return (
    <div className="app app-shell">
      <div className={`app-mode app-mode-welcome ${mode === 'welcome' ? 'active' : ''}`}>
        {welcomeContent}
      </div>
      <div className={`app-mode app-mode-play ${mode === 'play' ? 'active' : ''}`}>
        {playContent}
      </div>
      <div className={`app-mode app-mode-settings ${mode === 'settings' ? 'active' : ''}`}>
        {settingsContent}
      </div>
      {errorBanner}
    </div>
  )
}

export const AppShell = React.memo(AppShellComponent)
