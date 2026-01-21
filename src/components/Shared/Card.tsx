import React from 'react'

type CardProps = {
  children: React.ReactNode
  className?: string
  clickable?: boolean
  active?: boolean
  onClick?: () => void
}

function CardComponent({ children, className = '', clickable = false, active = false, onClick }: CardProps) {
  const classes = [
    'card',
    clickable ? 'card-clickable' : '',
    active ? 'card-active' : '',
    className,
  ].filter(Boolean).join(' ')

  if (clickable) {
    return (
      <button type="button" className={classes} onClick={onClick}>
        {children}
      </button>
    )
  }

  return <div className={classes}>{children}</div>
}

export const Card = React.memo(CardComponent)
