import React from 'react'
import type { ProfileRating } from '../../../types'
import { formatRecord, formatWinRate } from '../../../types'

type HeuristicProfilesProps = {
  profiles: ProfileRating[]
  activeProfileId: string | null
}

function HeuristicProfilesComponent({ profiles, activeProfileId }: HeuristicProfilesProps) {
  const heuristic = profiles.filter((p) => p.kind === 'heuristic')

  return (
    <div className="panel">
      <h3>AI Profiles</h3>
      <p>12 calibrated AI profiles ordered from easiest to strongest.</p>
      <div className="table-wrap">
        <table className="profile-table">
          <thead>
            <tr>
              <th>Profile</th>
              <th>Elo</th>
              <th>Games</th>
              <th>W-D-L</th>
              <th>Win rate</th>
              <th>Depth</th>
              <th>Candidates</th>
              <th>Nodes</th>
              <th>Defense</th>
              <th>Random</th>
            </tr>
          </thead>
          <tbody>
            {heuristic.map((profile) => (
              <tr key={profile.id} className={profile.id === activeProfileId ? 'active' : ''}>
                <td>{profile.name}</td>
                <td>{Math.round(profile.rating)}</td>
                <td>{profile.games}</td>
                <td>{formatRecord(profile.wins, profile.draws, profile.losses)}</td>
                <td>{formatWinRate(profile.wins, profile.draws, profile.losses)}</td>
                <td>{profile.config?.depth ?? '—'}</td>
                <td>{profile.config?.maxCandidates ?? '—'}</td>
                <td>{profile.config?.maxNodes ?? '—'}</td>
                <td>{profile.config?.defenseWeight ?? '—'}</td>
                <td>{profile.config?.randomness ?? '—'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}

export const HeuristicProfiles = React.memo(HeuristicProfilesComponent)
