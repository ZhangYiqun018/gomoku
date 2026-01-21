import React from 'react'
import type { GameResult } from '../../types'
import { Button } from '../Shared'

type ResultOverlayProps = {
  result: GameResult | null
  onNewGame: () => void
}

function ResultOverlayComponent({ result, onNewGame }: ResultOverlayProps) {
  if (!result) return null

  const getMessage = () => {
    if (result === 'B_WIN') return 'Black Wins!'
    if (result === 'W_WIN') return 'White Wins!'
    return 'Draw!'
  }

  const getSubtext = () => {
    if (result === 'B_WIN') return 'You connected five stones'
    if (result === 'W_WIN') return 'The opponent connected five stones'
    return 'The board is full with no winner'
  }

  return (
    <div className={`result-overlay result-${result.toLowerCase()}`}>
      <div className="result-overlay-content">
        <h2>{getMessage()}</h2>
        <p>{getSubtext()}</p>
        <Button variant="primary" onClick={onNewGame}>
          Play Again
        </Button>
      </div>
    </div>
  )
}

export const ResultOverlay = React.memo(ResultOverlayComponent)
