import React, { useMemo, useState } from 'react'
import type { LadderEntry, LadderFilter } from '../../../types'
import { formatRecord, formatWinRate } from '../../../types'
import { Chip } from '../../Shared'

type LadderTableProps = {
  entries: LadderEntry[]
  activeProfileId: string | null
}

function LadderTableComponent({ entries, activeProfileId }: LadderTableProps) {
  const [filter, setFilter] = useState<LadderFilter>('all')

  const filteredEntries = useMemo(() => {
    const filtered =
      filter === 'all'
        ? entries
        : entries.filter((entry) => {
          if (filter === 'user') return entry.kind === 'user'
          if (filter === 'ai') return entry.kind === 'heuristic'
          if (filter === 'llm') return entry.kind === 'llm'
          return true
        })
    return [...filtered].sort((a, b) => b.rating - a.rating)
  }, [entries, filter])

  return (
    <div className="panel">
      <h3>Ladder Rankings</h3>
      <div className="chip-row">
        {([
          ['all', 'All'],
          ['user', 'User'],
          ['ai', 'AI'],
          ['llm', 'LLM'],
        ] as const).map(([key, label]) => (
          <Chip
            key={key}
            active={filter === key}
            onClick={() => setFilter(key)}
          >
            {label}
          </Chip>
        ))}
      </div>
      <div className="table-wrap">
        <table className="profile-table">
          <thead>
            <tr>
              <th>Profile</th>
              <th>Type</th>
              <th>Elo</th>
              <th>Games</th>
              <th>W-D-L</th>
              <th>Win rate</th>
            </tr>
          </thead>
          <tbody>
            {filteredEntries.map((entry) => (
              <tr key={entry.id} className={entry.id === activeProfileId || entry.id === 'user' ? 'active' : ''}>
                <td>{entry.name}</td>
                <td>
                  {entry.kind === 'user'
                    ? 'User'
                    : entry.kind === 'llm'
                    ? 'LLM'
                    : 'AI'}
                </td>
                <td>{Math.round(entry.rating)}</td>
                <td>{entry.games}</td>
                <td>{formatRecord(entry.wins, entry.draws, entry.losses)}</td>
                <td>{formatWinRate(entry.wins, entry.draws, entry.losses)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}

export const LadderTable = React.memo(LadderTableComponent)
