import React, { useMemo } from 'react'
import type { Move, Player } from '../../types'

type BoardProps = {
  boardSize: number
  board: Array<Player | null>
  lastMove: Move | null
  onCellClick: (x: number, y: number) => void
}

function BoardComponent({ boardSize, board, lastMove, onCellClick }: BoardProps) {
  const max = boardSize - 1
  const viewBox = `-0.5 -0.5 ${max + 1} ${max + 1}`

  const gridIndices = useMemo(
    () => Array.from({ length: boardSize }, (_, i) => i),
    [boardSize],
  )

  return (
    <div className="board" role="grid">
      <svg className="board-svg" viewBox={viewBox} aria-hidden>
        <defs>
          <radialGradient id="stone-black" cx="30%" cy="30%" r="70%">
            <stop offset="0%" stopColor="#4b5563" />
            <stop offset="70%" stopColor="#0c0f14" />
          </radialGradient>
          <radialGradient id="stone-white" cx="30%" cy="30%" r="70%">
            <stop offset="0%" stopColor="#ffffff" />
            <stop offset="70%" stopColor="#cfd5df" />
          </radialGradient>
        </defs>
        <g className="board-lines">
          {gridIndices.map((i) => (
            <line key={`h-${i}`} x1={0} y1={i} x2={max} y2={i} />
          ))}
          {gridIndices.map((i) => (
            <line key={`v-${i}`} x1={i} y1={0} x2={i} y2={max} />
          ))}
        </g>
        <g className="board-stones">
          {board.map((cell, index) => {
            if (!cell) return null
            const x = index % boardSize
            const y = Math.floor(index / boardSize)
            return (
              <circle
                key={`stone-${x}-${y}`}
                className={`stone ${cell === 'B' ? 'black' : 'white'}`}
                cx={x}
                cy={y}
                r={0.38}
                fill={cell === 'B' ? 'url(#stone-black)' : 'url(#stone-white)'}
              />
            )
          })}
          {lastMove && (
            <circle className="last-ring" cx={lastMove.x} cy={lastMove.y} r={0.52} />
          )}
        </g>
        <g className="board-hits">
          {gridIndices.map((row) =>
            gridIndices.map((col) => (
              <circle
                key={`hit-${row}-${col}`}
                className="hit"
                cx={col}
                cy={row}
                r={0.5}
                onClick={() => onCellClick(col, row)}
              />
            )),
          )}
        </g>
      </svg>
    </div>
  )
}

// Memoize the board to prevent unnecessary re-renders
// Note: We must compare onCellClick to ensure the callback is always current
export const Board = React.memo(BoardComponent, (prev, next) => {
  // Always re-render if callback changes to avoid stale closures
  if (prev.onCellClick !== next.onCellClick) return false
  // Only re-render if board content or lastMove changed
  if (prev.boardSize !== next.boardSize) return false
  if (prev.lastMove?.x !== next.lastMove?.x || prev.lastMove?.y !== next.lastMove?.y) return false
  // Compare board arrays by reference first (fast path), then by content if needed
  if (prev.board === next.board) return true
  if (prev.board.length !== next.board.length) return false
  for (let i = 0; i < prev.board.length; i++) {
    if (prev.board[i] !== next.board[i]) return false
  }
  return true
})
