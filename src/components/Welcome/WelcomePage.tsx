import React from 'react'
import type { GameSnapshot, ProfileRating, RatingEntry, UserInfo } from '../../types'
import { Button } from '../Shared'

type WelcomePageProps = {
  user: UserInfo | null
  playerRating: RatingEntry | null
  profiles: ProfileRating[]
  activeProfile: ProfileRating | null
  game: GameSnapshot
  onPlayHumanVsAi: (humanColor: 'B' | 'W') => void
  onPlayHumanVsHuman: () => void
  onOpenAiVsAiDialog: () => void
  onContinueGame: () => void
  onLoad: () => void
  onSettings: () => void
}

function WelcomePageComponent({
  user,
  playerRating,
  activeProfile,
  game,
  onPlayHumanVsAi,
  onPlayHumanVsHuman,
  onOpenAiVsAiDialog,
  onContinueGame,
  onLoad,
  onSettings,
}: WelcomePageProps) {
  const userDisplay = user?.name ?? 'Guest'
  const userRating = playerRating ? Math.round(playerRating.rating) : '—'
  const hasOngoingGame = game.moves.length > 0 && !game.result

  return (
    <div className="welcome-page">
      <div className="welcome-content">
        <header className="welcome-header">
          <h1 className="welcome-title">GOMOKU</h1>
          <div className="welcome-user-info">
            <span className="welcome-user-name">{userDisplay}</span>
            <span className="welcome-user-rating">{userRating}</span>
          </div>
        </header>

        <div className="welcome-modes">
          <div className="mode-card mode-card-primary">
            <div className="mode-card-icon">
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <circle cx="12" cy="8" r="4" />
                <path d="M6 21v-2a4 4 0 0 1 4-4h4a4 4 0 0 1 4 4v2" />
              </svg>
            </div>
            <div className="mode-card-content">
              <h2>Play vs AI</h2>
              <p>Challenge {activeProfile?.name ?? 'AI'} (Elo {activeProfile ? Math.round(activeProfile.rating) : '—'})</p>
            </div>
            <div className="mode-card-actions">
              <Button variant="primary" onClick={() => onPlayHumanVsAi('B')}>
                Play Black (First)
              </Button>
              <Button onClick={() => onPlayHumanVsAi('W')}>
                Play White (Second)
              </Button>
            </div>
          </div>

          <div className="mode-cards-row">
            <button type="button" className="mode-card mode-card-secondary" onClick={onPlayHumanVsHuman}>
              <div className="mode-card-icon">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <circle cx="9" cy="7" r="3" />
                  <circle cx="15" cy="7" r="3" />
                  <path d="M3 21v-2a4 4 0 0 1 4-4h2" />
                  <path d="M15 15h4a4 4 0 0 1 4 4v2" />
                </svg>
              </div>
              <div className="mode-card-content">
                <h3>Two Players</h3>
                <p>Play locally with a friend</p>
              </div>
            </button>

            <button type="button" className="mode-card mode-card-secondary" onClick={onOpenAiVsAiDialog}>
              <div className="mode-card-icon">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <rect x="2" y="6" width="8" height="12" rx="2" />
                  <rect x="14" y="6" width="8" height="12" rx="2" />
                  <path d="M10 12h4" />
                </svg>
              </div>
              <div className="mode-card-content">
                <h3>Watch AI</h3>
                <p>Watch two AIs compete</p>
              </div>
            </button>
          </div>

          {hasOngoingGame && (
            <button type="button" className="mode-card mode-card-continue" onClick={onContinueGame}>
              <div className="mode-card-icon">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <polygon points="5 3 19 12 5 21 5 3" />
                </svg>
              </div>
              <div className="mode-card-content">
                <h3>Continue Game</h3>
                <p>
                  {game.toMove === 'B' ? 'Black' : 'White'} to move (Move #{game.moves.length + 1})
                </p>
              </div>
            </button>
          )}
        </div>

        <div className="welcome-footer">
          <Button onClick={onLoad}>Load Game</Button>
          <Button onClick={onSettings}>Settings</Button>
        </div>
      </div>
    </div>
  )
}

export const WelcomePage = React.memo(WelcomePageComponent)
