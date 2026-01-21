import React from 'react'
import type { GameSnapshot, ProfileRating } from '../../types'

type StatusCardProps = {
  game: GameSnapshot
  activeProfile?: ProfileRating
}

function StatusCardComponent({ game, activeProfile }: StatusCardProps) {
  const statusText = (() => {
    if (game.result === 'B_WIN') return 'Black wins'
    if (game.result === 'W_WIN') return 'White wins'
    if (game.result === 'DRAW') return 'Draw'
    return game.toMove === 'B' ? 'Black to move' : 'White to move'
  })()

  return (
    <div className="status-card">
      <div>
        <span className="status-label">Status</span>
        <strong>{statusText}</strong>
      </div>
      <div>
        <span className="status-label">Moves</span>
        <strong>{game.moves.length}</strong>
      </div>
      <div>
        <span className="status-label">Opponent</span>
        <strong>{activeProfile?.name ?? 'AI'}</strong>
      </div>
    </div>
  )
}

export const StatusCard = React.memo(StatusCardComponent)
