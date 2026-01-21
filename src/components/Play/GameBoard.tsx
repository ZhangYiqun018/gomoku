import React from 'react'
import { Board } from '../Board'
import type { GameSnapshot, Move } from '../../types'

type GameBoardProps = {
  game: GameSnapshot
  lastMove: Move | null
  onCellClick: (x: number, y: number) => void
}

function GameBoardComponent({ game, lastMove, onCellClick }: GameBoardProps) {
  return (
    <div className="game-board-container">
      <Board
        boardSize={game.boardSize}
        board={game.board}
        lastMove={lastMove}
        onCellClick={onCellClick}
      />
    </div>
  )
}

export const GameBoard = React.memo(GameBoardComponent)
