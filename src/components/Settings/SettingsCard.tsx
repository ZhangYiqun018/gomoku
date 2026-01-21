import React from 'react'

type SettingsCardProps = {
  icon: React.ReactNode
  title: string
  description: string
  onClick: () => void
}

function SettingsCardComponent({ icon, title, description, onClick }: SettingsCardProps) {
  return (
    <button type="button" className="settings-card" onClick={onClick}>
      <div className="settings-card-icon">{icon}</div>
      <div className="settings-card-content">
        <h3>{title}</h3>
        <p>{description}</p>
      </div>
      <svg className="settings-card-arrow" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
        <path d="M9 18l6-6-6-6" />
      </svg>
    </button>
  )
}

export const SettingsCard = React.memo(SettingsCardComponent)
