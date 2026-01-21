import React from 'react'
import type { ProfileRating } from '../../../types'
import { formatRecord, formatWinRate } from '../../../types'
import { Button } from '../../Shared'

type LlmProfilesProps = {
  profiles: ProfileRating[]
  onEdit: (profile: ProfileRating) => void
  onDelete: (id: string) => void
}

function LlmProfilesComponent({ profiles, onEdit, onDelete }: LlmProfilesProps) {
  const llmProfiles = profiles.filter((p) => p.kind === 'llm')

  return (
    <div className="panel">
      <h3>LLM Profiles</h3>
      <p>Create LLM opponents with custom base URL, model, and sampling settings.</p>
      <p className="muted">API keys are stored locally in each user folder.</p>
      <div className="user-grid">
        {llmProfiles.length === 0 && <span className="muted">No LLM profiles yet.</span>}
        {llmProfiles.map((profile) => (
          <div key={profile.id} className="user-card">
            <div className="user-info">
              <strong>{profile.name}</strong>
              <span className="user-meta">Model: {profile.llm?.model ?? '—'}</span>
              <span className="user-meta">Base URL: {profile.llm?.baseUrl || 'default'}</span>
              <span className="user-meta">Elo: {Math.round(profile.rating)} · Games: {profile.games}</span>
              <span className="user-meta">
                Record: {formatRecord(profile.wins, profile.draws, profile.losses)} · Win rate: {formatWinRate(profile.wins, profile.draws, profile.losses)}
              </span>
              <span className="user-meta">API key: {profile.llm?.apiKeySet ? 'set' : 'missing'}</span>
            </div>
            <div className="user-actions">
              <Button onClick={() => onEdit(profile)}>Edit</Button>
              <Button variant="danger" onClick={() => onDelete(profile.id)}>
                Delete
              </Button>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

export const LlmProfiles = React.memo(LlmProfilesComponent)
