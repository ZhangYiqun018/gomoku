import React from 'react'

type BackHeaderProps = {
  title: string
  subtitle?: string
  onBack: () => void
}

function BackHeaderComponent({ title, subtitle, onBack }: BackHeaderProps) {
  return (
    <header className="back-header">
      <button type="button" className="back-button" onClick={onBack} aria-label="Go back">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M19 12H5M12 19l-7-7 7-7" />
        </svg>
        <span>Back</span>
      </button>
      <div className="back-header-title">
        {subtitle && <span className="eyebrow">{subtitle}</span>}
        <h1>{title}</h1>
      </div>
    </header>
  )
}

export const BackHeader = React.memo(BackHeaderComponent)
