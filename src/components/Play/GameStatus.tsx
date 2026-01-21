import React from 'react'
import type { GameSnapshot } from '../../types'

type GameStatusProps = {
  game: GameSnapshot
  busy: boolean
  expectedWin: number | null
}

function GameStatusComponent({ game, busy, expectedWin }: GameStatusProps) {
  const moveNumber = game.moves.length
  const toMoveText = game.toMove === 'B' ? 'Black to move' : 'White to move'
  const winPct = expectedWin !== null ? `${(expectedWin * 100).toFixed(0)}%` : '—'

  if (game.result) {
    return null
  }

  return (
    <div className="game-status">
      <span className="game-status-item">
        <span className="game-status-stone" data-player={game.toMove} />
        {busy ? 'AI thinking...' : toMoveText}
      </span>
      <span className="game-status-item game-status-move">Move {moveNumber}</span>
      <span className="game-status-item">Win rate {winPct}</span>
    </div>
  )
}

export const GameStatus = React.memo(GameStatusComponent)
