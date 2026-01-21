import React from 'react'
import type { AutoPlaySpeed } from '../../hooks'
import type { GameMode } from '../../types'
import { Button } from '../Shared'
import { WatchControls } from './WatchControls'

type ActionBarProps = {
  gameOver: boolean
  mode: GameMode
  isPlaying?: boolean
  autoPlaySpeed?: AutoPlaySpeed
  onNewGame: () => void
  onSave: () => void
  onLoad: () => void
  onAutoPlayStart?: () => void
  onAutoPlayStop?: () => void
  onAutoPlayStep?: () => void
  onAutoPlaySpeedChange?: (speed: AutoPlaySpeed) => void
}

function ActionBarComponent({
  gameOver,
  mode,
  isPlaying = false,
  autoPlaySpeed = 'medium',
  onNewGame,
  onSave,
  onLoad,
  onAutoPlayStart,
  onAutoPlayStop,
  onAutoPlayStep,
  onAutoPlaySpeedChange,
}: ActionBarProps) {
  const isAiVsAi = mode.type === 'ai_vs_ai'

  return (
    <div className="action-bar">
      <div className="action-bar-left">
        <Button variant="primary" onClick={onNewGame}>
          New Game
        </Button>
        {isAiVsAi && onAutoPlayStart && onAutoPlayStop && onAutoPlayStep && onAutoPlaySpeedChange && (
          <WatchControls
            isPlaying={isPlaying}
            speed={autoPlaySpeed}
            disabled={gameOver}
            onPlay={onAutoPlayStart}
            onPause={onAutoPlayStop}
            onStep={onAutoPlayStep}
            onSpeedChange={onAutoPlaySpeedChange}
          />
        )}
      </div>
      <div className="action-bar-right">
        <Button onClick={onSave}>Save</Button>
        <Button onClick={onLoad}>Load</Button>
      </div>
    </div>
  )
}

export const ActionBar = React.memo(ActionBarComponent)
