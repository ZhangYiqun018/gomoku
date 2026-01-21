import React from 'react'
import type { ProfileRating } from '../../../types'
import { formatRecord } from '../../../types'
import { Chip } from '../../Shared'

type MatchSettingsProps = {
  profiles: ProfileRating[]
  activeProfileId: string | null
  autoMatch: boolean
  matchOffset: number
  onToggleAutoMatch: () => void
  onSetOffset: (offset: number) => void
  onSelectProfile: (id: string) => void
}

function MatchSettingsComponent({
  profiles,
  activeProfileId,
  autoMatch,
  matchOffset,
  onToggleAutoMatch,
  onSetOffset,
  onSelectProfile,
}: MatchSettingsProps) {
  return (
    <div className="panel">
      <h3>Match Settings</h3>
      <p>Use auto match to keep your opponent close to your current rating.</p>
      <div className="match-row">
        <Chip active={autoMatch} onClick={onToggleAutoMatch}>
          Auto match
        </Chip>
        <div className="chip-row">
          {[-200, 0, 200].map((offset) => (
            <Chip
              key={offset}
              active={matchOffset === offset}
              onClick={() => onSetOffset(offset)}
            >
              {offset > 0 ? `+${offset}` : offset}
            </Chip>
          ))}
        </div>
      </div>
      <div className="select-row">
        <span className="status-label">Manual profile</span>
        <select
          value={activeProfileId ?? ''}
          onChange={(e) => onSelectProfile(e.target.value)}
          disabled={autoMatch}
        >
          {profiles.map((profile) => (
            <option key={profile.id} value={profile.id}>
              {profile.name} ({profile.kind === 'llm' ? 'LLM' : 'AI'} · Elo {Math.round(profile.rating)} · {formatRecord(profile.wins, profile.draws, profile.losses)})
            </option>
          ))}
        </select>
      </div>
    </div>
  )
}

export const MatchSettings = React.memo(MatchSettingsComponent)
