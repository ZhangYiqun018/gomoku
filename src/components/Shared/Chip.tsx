import React from 'react'

type ChipProps = {
  active?: boolean
  disabled?: boolean
  onClick?: () => void
  children: React.ReactNode
  className?: string
}

function ChipComponent({ active = false, disabled = false, onClick, children, className = '' }: ChipProps) {
  return (
    <button
      type="button"
      className={`chip ${active ? 'active' : ''} ${className}`.trim()}
      disabled={disabled}
      onClick={onClick}
    >
      {children}
    </button>
  )
}

export const Chip = React.memo(ChipComponent)
