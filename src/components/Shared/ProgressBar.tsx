import React from 'react'

type ProgressBarProps = {
  value: number
  max?: number
  label?: string
  className?: string
}

function ProgressBarComponent({ value, max = 100, label, className = '' }: ProgressBarProps) {
  const percent = Math.min(100, Math.max(0, (value / max) * 100))

  return (
    <div className={`progress-row ${className}`.trim()}>
      <div className="progress-bar">
        <div className="progress-fill" style={{ width: `${percent.toFixed(1)}%` }} />
      </div>
      {label && <span>{label}</span>}
    </div>
  )
}

export const ProgressBar = React.memo(ProgressBarComponent)
