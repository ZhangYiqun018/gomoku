import React from 'react'

type FormFieldProps = {
  label: string
  help?: string
  children: React.ReactNode
  className?: string
}

function FormFieldComponent({ label, help, children, className = '' }: FormFieldProps) {
  return (
    <label className={`field ${className}`.trim()}>
      <span className="field-label">{label}</span>
      {children}
      {help && <span className="field-help">{help}</span>}
    </label>
  )
}

export const FormField = React.memo(FormFieldComponent)
