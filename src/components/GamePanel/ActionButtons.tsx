import React from 'react'
import type { GameResult } from '../../types'

type ActionButtonsProps = {
  busy: boolean
  result: GameResult | null
  onNewGame: () => void
  onAiMove: () => void
}

function ActionButtonsComponent({ busy, result, onNewGame, onAiMove }: ActionButtonsProps) {
  return (
    <div className="action-grid">
      <button className="primary" onClick={onNewGame}>
        New game
      </button>
      <button onClick={onAiMove} disabled={busy || !!result}>
        AI move
      </button>
    </div>
  )
}

export const ActionButtons = React.memo(ActionButtonsComponent)
