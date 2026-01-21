import React, { useState, useRef, useEffect } from 'react'
import type { ProfileRating } from '../../types'

type OpponentSelectProps = {
  profiles: ProfileRating[]
  activeProfile: ProfileRating | null
  autoMatch: boolean
  matchOffset: number
  onSelectProfile: (id: string) => void
  onToggleAutoMatch: () => void
  onSetOffset: (offset: number) => void
}

function OpponentSelectComponent({
  profiles,
  activeProfile,
  autoMatch,
  matchOffset,
  onSelectProfile,
  onToggleAutoMatch,
  onSetOffset,
}: OpponentSelectProps) {
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!open) return
    const handleClickOutside = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [open])

  return (
    <div className="opponent-select" ref={ref}>
      <button
        type="button"
        className="opponent-select-trigger"
        onClick={() => setOpen(!open)}
        aria-expanded={open}
      >
        <span className="opponent-select-label">vs</span>
        <span className="opponent-select-name">{activeProfile?.name ?? 'Select opponent'}</span>
        <span className="opponent-select-rating">
          {activeProfile ? Math.round(activeProfile.rating) : '—'}
        </span>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M6 9l6 6 6-6" />
        </svg>
      </button>
      {open && (
        <div className="opponent-select-dropdown">
          <div className="opponent-select-header">
            <button
              type="button"
              className={`chip ${autoMatch ? 'active' : ''}`}
              onClick={() => {
                onToggleAutoMatch()
              }}
            >
              Auto Match
            </button>
            {autoMatch && (
              <div className="chip-row">
                {[-200, 0, 200].map((offset) => (
                  <button
                    key={offset}
                    type="button"
                    className={`chip chip-small ${matchOffset === offset ? 'active' : ''}`}
                    onClick={() => onSetOffset(offset)}
                  >
                    {offset > 0 ? `+${offset}` : offset}
                  </button>
                ))}
              </div>
            )}
          </div>
          {!autoMatch && (
            <div className="opponent-select-list">
              {profiles.map((profile) => (
                <button
                  key={profile.id}
                  type="button"
                  className={`opponent-select-item ${profile.id === activeProfile?.id ? 'active' : ''}`}
                  onClick={() => {
                    onSelectProfile(profile.id)
                    setOpen(false)
                  }}
                >
                  <span className="opponent-select-item-name">{profile.name}</span>
                  <span className="opponent-select-item-type">
                    {profile.kind === 'llm' ? 'LLM' : 'AI'}
                  </span>
                  <span className="opponent-select-item-rating">{Math.round(profile.rating)}</span>
                </button>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}

export const OpponentSelect = React.memo(OpponentSelectComponent)
