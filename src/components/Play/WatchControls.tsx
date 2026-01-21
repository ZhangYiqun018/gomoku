import React from 'react'
import type { AutoPlaySpeed } from '../../hooks'
import { Button } from '../Shared'

type WatchControlsProps = {
  isPlaying: boolean
  speed: AutoPlaySpeed
  disabled: boolean
  onPlay: () => void
  onPause: () => void
  onStep: () => void
  onSpeedChange: (speed: AutoPlaySpeed) => void
}

function WatchControlsComponent({
  isPlaying,
  speed,
  disabled,
  onPlay,
  onPause,
  onStep,
  onSpeedChange,
}: WatchControlsProps) {
  return (
    <div className="watch-controls">
      {isPlaying ? (
        <Button onClick={onPause} disabled={disabled}>
          Pause
        </Button>
      ) : (
        <Button variant="primary" onClick={onPlay} disabled={disabled}>
          Play
        </Button>
      )}
      <Button onClick={onStep} disabled={disabled || isPlaying}>
        Step
      </Button>
      <div className="speed-control">
        <label>Speed:</label>
        <select
          value={speed}
          onChange={(e) => onSpeedChange(e.target.value as AutoPlaySpeed)}
          disabled={disabled}
        >
          <option value="slow">Slow (2s)</option>
          <option value="medium">Medium (1s)</option>
          <option value="fast">Fast (0.5s)</option>
        </select>
      </div>
    </div>
  )
}

export const WatchControls = React.memo(WatchControlsComponent)
