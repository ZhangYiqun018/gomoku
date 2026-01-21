import React, { useMemo } from 'react'
import type { RatingsSnapshot, UserInfo, LadderEntry } from '../../../types'
import { BackHeader } from '../../Layout'
import { RatingOverview } from './RatingOverview'
import { MatchSettings } from './MatchSettings'
import { LadderTable } from './LadderTable'

type ProfilePageProps = {
  ratings: RatingsSnapshot | null
  activeUser: UserInfo | null
  onBack: () => void
  onToggleAutoMatch: () => void
  onSetOffset: (offset: number) => void
  onSelectProfile: (id: string) => void
}

function ProfilePageComponent({
  ratings,
  activeUser,
  onBack,
  onToggleAutoMatch,
  onSetOffset,
  onSelectProfile,
}: ProfilePageProps) {
  const activeProfile = ratings
    ? ratings.profiles.find((p) => p.id === ratings.activeProfile)
    : null

  const ladderEntries = useMemo((): LadderEntry[] => {
    if (!ratings) return []
    const entries: LadderEntry[] = ratings.profiles.map((profile) => ({
      id: profile.id,
      name: profile.name,
      kind: profile.kind,
      rating: profile.rating,
      games: profile.games,
      wins: profile.wins,
      draws: profile.draws,
      losses: profile.losses,
    }))
    entries.push({
      id: 'user',
      name: activeUser?.name ?? 'You',
      kind: 'user',
      rating: ratings.player.rating,
      games: ratings.player.games,
      wins: ratings.player.wins,
      draws: ratings.player.draws,
      losses: ratings.player.losses,
    })
    return entries
  }, [ratings, activeUser])

  return (
    <div className="settings-page">
      <BackHeader title="Rating & Match" subtitle="Settings" onBack={onBack} />
      <div className="settings-page-content">
        <RatingOverview
          player={ratings?.player ?? null}
          activeProfile={activeProfile ?? null}
          activeUser={activeUser}
        />
        <MatchSettings
          profiles={ratings?.profiles ?? []}
          activeProfileId={ratings?.activeProfile ?? null}
          autoMatch={ratings?.autoMatch ?? false}
          matchOffset={ratings?.matchOffset ?? 0}
          onToggleAutoMatch={onToggleAutoMatch}
          onSetOffset={onSetOffset}
          onSelectProfile={onSelectProfile}
        />
        <LadderTable
          entries={ladderEntries}
          activeProfileId={ratings?.activeProfile ?? null}
        />
      </div>
    </div>
  )
}

export const ProfilePage = React.memo(ProfilePageComponent)
