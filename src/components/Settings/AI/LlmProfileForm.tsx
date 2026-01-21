import React from 'react'
import { Button, FormField } from '../../Shared'

type LlmFormState = {
  id: string | null
  name: string
  baseUrl: string
  model: string
  temperature: number
  topP: number
  maxTokens: number
  timeout: number
  candidateLimit: number
  apiKey: string
}

type LlmProfileFormProps = {
  form: LlmFormState
  onChange: (updates: Partial<LlmFormState>) => void
  onSave: () => void
  onCancel: () => void
  deleteKey: boolean
  onDeleteKeyChange: (value: boolean) => void
}

function LlmProfileFormComponent({
  form,
  onChange,
  onSave,
  onCancel,
  deleteKey,
  onDeleteKeyChange,
}: LlmProfileFormProps) {
  const isEditing = !!form.id

  return (
    <div className="llm-form">
      <FormField label="Profile name">
        <input
          type="text"
          value={form.name}
          onChange={(e) => onChange({ name: e.target.value })}
          placeholder="LLM profile name"
        />
      </FormField>
      <FormField label="Base URL">
        <input
          type="text"
          value={form.baseUrl}
          onChange={(e) => onChange({ baseUrl: e.target.value })}
          placeholder="https://api.openai.com/v1"
        />
      </FormField>
      <FormField label="Model">
        <input
          type="text"
          value={form.model}
          onChange={(e) => onChange({ model: e.target.value })}
          placeholder="gpt-4o-mini"
        />
      </FormField>
      <div className="field-row">
        <FormField label="Temperature">
          <input
            type="number"
            step="0.1"
            min="0"
            max="2"
            value={form.temperature}
            onChange={(e) => onChange({ temperature: Number(e.target.value) })}
          />
        </FormField>
        <FormField label="Top P">
          <input
            type="number"
            step="0.1"
            min="0"
            max="1"
            value={form.topP}
            onChange={(e) => onChange({ topP: Number(e.target.value) })}
          />
        </FormField>
      </div>
      <div className="field-row">
        <FormField label="Max tokens">
          <input
            type="number"
            min="32"
            max="512"
            value={form.maxTokens}
            onChange={(e) => onChange({ maxTokens: Number(e.target.value) })}
          />
        </FormField>
        <FormField label="Timeout (ms)">
          <input
            type="number"
            min="5000"
            max="60000"
            value={form.timeout}
            onChange={(e) => onChange({ timeout: Number(e.target.value) })}
          />
        </FormField>
      </div>
      <FormField label="Candidate limit" help="Higher values give more options but cost more tokens.">
        <input
          type="number"
          min="6"
          max="30"
          value={form.candidateLimit}
          onChange={(e) => onChange({ candidateLimit: Number(e.target.value) })}
        />
      </FormField>
      <FormField label="API key">
        <input
          type="text"
          value={form.apiKey}
          onChange={(e) => onChange({ apiKey: e.target.value })}
          placeholder={isEditing ? 'Leave blank to keep existing key' : 'sk-...'}
        />
      </FormField>
      <div className="button-row">
        <Button variant="primary" onClick={onSave}>
          {isEditing ? 'Update profile' : 'Create profile'}
        </Button>
        {isEditing && <Button onClick={onCancel}>Cancel</Button>}
      </div>
      <label className="toggle-row">
        <input
          type="checkbox"
          checked={deleteKey}
          onChange={(e) => onDeleteKeyChange(e.target.checked)}
        />
        <span>Delete API key when removing profile</span>
      </label>
    </div>
  )
}

export const LlmProfileForm = React.memo(LlmProfileFormComponent)

export type { LlmFormState }
