import React from 'react'
import type { ProfileRating, RatingEntry, UserInfo } from '../../types'

type TopBarProps = {
  user: UserInfo | null
  playerRating: RatingEntry | null
  opponent: ProfileRating | null
  showBackButton?: boolean
  onBackClick?: () => void
  onSettingsClick: () => void
}

function TopBarComponent({ user, playerRating, opponent, showBackButton, onBackClick, onSettingsClick }: TopBarProps) {
  const userDisplay = user?.name ?? 'Guest'
  const userRating = playerRating ? Math.round(playerRating.rating) : '—'
  const opponentDisplay = opponent?.name ?? 'AI'
  const opponentRating = opponent ? Math.round(opponent.rating) : '—'

  return (
    <header className="top-bar">
      <div className="top-bar-left">
        {showBackButton && onBackClick && (
          <button
            type="button"
            className="top-bar-back"
            onClick={onBackClick}
            aria-label="Back to menu"
          >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <path d="M19 12H5M12 19l-7-7 7-7" />
            </svg>
          </button>
        )}
        <div className="top-bar-user">
          <span className="top-bar-name">{userDisplay}</span>
          <span className="top-bar-rating">{userRating}</span>
        </div>
      </div>
      <h1 className="top-bar-title">GOMOKU</h1>
      <div className="top-bar-right">
        <div className="top-bar-opponent">
          <span className="top-bar-name">{opponentDisplay}</span>
          <span className="top-bar-rating">{opponentRating}</span>
        </div>
        <button
          type="button"
          className="top-bar-settings"
          onClick={onSettingsClick}
          aria-label="Settings"
        >
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="12" cy="12" r="3" />
            <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
          </svg>
        </button>
      </div>
    </header>
  )
}

export const TopBar = React.memo(TopBarComponent)
