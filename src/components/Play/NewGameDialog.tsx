import React, { useCallback, useState } from 'react'
import type { GameMode, Player, ProfileRating } from '../../types'
import { Button } from '../Shared'

export type GameModeType = 'human_vs_ai' | 'ai_vs_ai' | 'human_vs_human'

type NewGameDialogProps = {
  open: boolean
  profiles: ProfileRating[]
  initialMode?: GameModeType
  onClose: () => void
  onStartGame: (mode: GameMode) => void
}

function NewGameDialogComponent({ open, profiles, initialMode, onClose, onStartGame }: NewGameDialogProps) {
  const [modeType, setModeType] = useState<GameModeType>(initialMode ?? 'human_vs_ai')
  const [humanColor, setHumanColor] = useState<Player>('B')
  const [blackAiId, setBlackAiId] = useState('')
  const [whiteAiId, setWhiteAiId] = useState('')

  // Sync modeType when initialMode prop changes (e.g., opening dialog from welcome page)
  React.useEffect(() => {
    if (initialMode) {
      setModeType(initialMode)
    }
  }, [initialMode])

  // Initialize AI IDs when profiles change
  React.useEffect(() => {
    if (profiles.length > 0) {
      if (!blackAiId || !profiles.some((p) => p.id === blackAiId)) {
        setBlackAiId(profiles[0].id)
      }
      if (!whiteAiId || !profiles.some((p) => p.id === whiteAiId)) {
        setWhiteAiId(profiles[Math.min(1, profiles.length - 1)].id)
      }
    }
  }, [profiles, blackAiId, whiteAiId])

  const handleStart = useCallback(() => {
    let mode: GameMode
    switch (modeType) {
      case 'human_vs_ai':
        mode = { type: 'human_vs_ai', humanColor }
        break
      case 'ai_vs_ai':
        mode = { type: 'ai_vs_ai', blackId: blackAiId, whiteId: whiteAiId }
        break
      case 'human_vs_human':
        mode = { type: 'human_vs_human' }
        break
    }
    onStartGame(mode)
    onClose()
  }, [modeType, humanColor, blackAiId, whiteAiId, onStartGame, onClose])

  if (!open) return null

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog new-game-dialog" onClick={(e) => e.stopPropagation()}>
        <h2>New Game</h2>

        <div className="dialog-section">
          <label className="dialog-label">Game Mode</label>
          <div className="mode-buttons">
            <button
              className={`mode-button ${modeType === 'human_vs_ai' ? 'active' : ''}`}
              onClick={() => setModeType('human_vs_ai')}
            >
              <span className="mode-icon">vs AI</span>
              <span className="mode-name">Human vs AI</span>
            </button>
            <button
              className={`mode-button ${modeType === 'ai_vs_ai' ? 'active' : ''}`}
              onClick={() => setModeType('ai_vs_ai')}
            >
              <span className="mode-icon">Watch</span>
              <span className="mode-name">AI vs AI</span>
            </button>
            <button
              className={`mode-button ${modeType === 'human_vs_human' ? 'active' : ''}`}
              onClick={() => setModeType('human_vs_human')}
            >
              <span className="mode-icon">Local</span>
              <span className="mode-name">2 Players</span>
            </button>
          </div>
        </div>

        {modeType === 'human_vs_ai' && (
          <div className="dialog-section">
            <label className="dialog-label">Your Color</label>
            <div className="color-buttons">
              <button
                className={`color-button ${humanColor === 'B' ? 'active' : ''}`}
                onClick={() => setHumanColor('B')}
              >
                <span className="stone black" />
                <span>Black (First)</span>
              </button>
              <button
                className={`color-button ${humanColor === 'W' ? 'active' : ''}`}
                onClick={() => setHumanColor('W')}
              >
                <span className="stone white" />
                <span>White (Second)</span>
              </button>
            </div>
          </div>
        )}

        {modeType === 'ai_vs_ai' && (
          <>
            <div className="dialog-section">
              <label className="dialog-label">Black (First)</label>
              <select
                className="ai-select"
                value={blackAiId}
                onChange={(e) => setBlackAiId(e.target.value)}
              >
                {profiles.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name} ({Math.round(p.rating)})
                  </option>
                ))}
              </select>
            </div>
            <div className="dialog-section">
              <label className="dialog-label">White (Second)</label>
              <select
                className="ai-select"
                value={whiteAiId}
                onChange={(e) => setWhiteAiId(e.target.value)}
              >
                {profiles.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name} ({Math.round(p.rating)})
                  </option>
                ))}
              </select>
            </div>
          </>
        )}

        <div className="dialog-actions">
          <Button onClick={onClose}>Cancel</Button>
          <Button variant="primary" onClick={handleStart}>
            Start Game
          </Button>
        </div>
      </div>
    </div>
  )
}

export const NewGameDialog = React.memo(NewGameDialogComponent)
