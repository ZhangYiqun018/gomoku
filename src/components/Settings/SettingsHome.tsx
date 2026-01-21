import React from 'react'
import type { SettingsPage } from '../../types'
import { SettingsCard } from './SettingsCard'

type SettingsHomeProps = {
  onNavigate: (page: SettingsPage) => void
  onClose: () => void
}

const TrophyIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M6 9H4.5a2.5 2.5 0 0 1 0-5H6" />
    <path d="M18 9h1.5a2.5 2.5 0 0 0 0-5H18" />
    <path d="M4 22h16" />
    <path d="M10 14.66V17c0 .55-.47.98-.97 1.21C7.85 18.75 7 20.24 7 22" />
    <path d="M14 14.66V17c0 .55.47.98.97 1.21C16.15 18.75 17 20.24 17 22" />
    <path d="M18 2H6v7a6 6 0 0 0 12 0V2Z" />
  </svg>
)

const CpuIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <rect x="4" y="4" width="16" height="16" rx="2" />
    <rect x="9" y="9" width="6" height="6" />
    <path d="M9 1v3M15 1v3M9 20v3M15 20v3M20 9h3M20 14h3M1 9h3M1 14h3" />
  </svg>
)

const DatabaseIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <ellipse cx="12" cy="5" rx="9" ry="3" />
    <path d="M3 5V19a9 3 0 0 0 18 0V5" />
    <path d="M3 12a9 3 0 0 0 18 0" />
  </svg>
)

const UsersIcon = () => (
  <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
    <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
    <circle cx="9" cy="7" r="4" />
    <path d="M22 21v-2a4 4 0 0 0-3-3.87" />
    <path d="M16 3.13a4 4 0 0 1 0 7.75" />
  </svg>
)

function SettingsHomeComponent({ onNavigate, onClose }: SettingsHomeProps) {
  return (
    <div className="settings-home">
      <header className="settings-home-header">
        <div>
          <span className="eyebrow">Settings</span>
          <h1>Configure Your Game</h1>
        </div>
        <button type="button" className="settings-close" onClick={onClose} aria-label="Close settings">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </header>
      <div className="settings-grid">
        <SettingsCard
          icon={<TrophyIcon />}
          title="Rating & Match"
          description="View your Elo rating, match history, and opponent settings"
          onClick={() => onNavigate('profile')}
        />
        <SettingsCard
          icon={<CpuIcon />}
          title="AI Configuration"
          description="Manage AI profiles, LLM opponents, and calibration"
          onClick={() => onNavigate('ai')}
        />
        <SettingsCard
          icon={<DatabaseIcon />}
          title="Data Management"
          description="Export training data and game records"
          onClick={() => onNavigate('data')}
        />
        <SettingsCard
          icon={<UsersIcon />}
          title="User Profiles"
          description="Switch users and manage player profiles"
          onClick={() => onNavigate('users')}
        />
      </div>
    </div>
  )
}

export const SettingsHome = React.memo(SettingsHomeComponent)
