import React from 'react'
import type { RatingEntry, ProfileRating, UserInfo } from '../../../types'
import { formatRecord, formatWinRate, BLACK_ADVANTAGE, expectedScore } from '../../../types'

type RatingOverviewProps = {
  player: RatingEntry | null
  activeProfile: ProfileRating | null
  activeUser: UserInfo | null
}

function RatingOverviewComponent({ player, activeProfile, activeUser }: RatingOverviewProps) {
  const expectedWin = player && activeProfile
    ? expectedScore(player.rating + BLACK_ADVANTAGE, activeProfile.rating)
    : null

  return (
    <div className="rating-cards">
      <div className="rating-card">
        <span className="status-label">Your Elo</span>
        <strong>{player ? Math.round(player.rating) : '—'}</strong>
        <span className="rating-sub">User: {activeUser?.name ?? '—'}</span>
        <span className="rating-sub">
          Record: {player ? formatRecord(player.wins, player.draws, player.losses) : '—'}
        </span>
        <span className="rating-sub">
          Win rate: {player ? formatWinRate(player.wins, player.draws, player.losses) : '—'}
        </span>
      </div>
      <div className="rating-card">
        <span className="status-label">Current opponent</span>
        <strong>{activeProfile?.name ?? '—'}</strong>
        <span className="rating-sub">
          Type: {activeProfile ? (activeProfile.kind === 'llm' ? 'LLM' : 'AI') : '—'}
        </span>
        <span className="rating-sub">Elo: {activeProfile ? Math.round(activeProfile.rating) : '—'}</span>
        <span className="rating-sub">
          Record: {activeProfile ? formatRecord(activeProfile.wins, activeProfile.draws, activeProfile.losses) : '—'}
        </span>
        <span className="rating-sub">
          Win rate: {activeProfile ? formatWinRate(activeProfile.wins, activeProfile.draws, activeProfile.losses) : '—'}
        </span>
      </div>
      <div className="rating-card">
        <span className="status-label">Expected win</span>
        <strong>{expectedWin !== null ? `${(expectedWin * 100).toFixed(1)}%` : '—'}</strong>
        <span className="rating-sub">Includes Black advantage</span>
      </div>
    </div>
  )
}

export const RatingOverview = React.memo(RatingOverviewComponent)
