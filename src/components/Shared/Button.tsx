import React from 'react'

type ButtonVariant = 'default' | 'primary' | 'danger' | 'ghost'

type ButtonProps = {
  variant?: ButtonVariant
  disabled?: boolean
  onClick?: () => void
  children: React.ReactNode
  className?: string
  type?: 'button' | 'submit'
}

function ButtonComponent({
  variant = 'default',
  disabled = false,
  onClick,
  children,
  className = '',
  type = 'button',
}: ButtonProps) {
  const variantClass = variant === 'default' ? '' : variant
  return (
    <button
      type={type}
      className={`${variantClass} ${className}`.trim()}
      disabled={disabled}
      onClick={onClick}
    >
      {children}
    </button>
  )
}

export const Button = React.memo(ButtonComponent)
