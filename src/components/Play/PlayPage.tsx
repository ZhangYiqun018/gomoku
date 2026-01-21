import React, { useCallback, useMemo, useState } from 'react'
import type { AutoPlaySpeed } from '../../hooks'
import type { GameMode, GameSnapshot, ProfileRating, RatingEntry, UserInfo } from '../../types'
import { BLACK_ADVANTAGE, expectedScore } from '../../types'
import { TopBar } from '../Layout'
import { GameBoard } from './GameBoard'
import { GameStatus } from './GameStatus'
import { ActionBar } from './ActionBar'
import { NewGameDialog } from './NewGameDialog'
import { OpponentSelect } from './OpponentSelect'
import { ResultOverlay } from './ResultOverlay'

type PlayPageProps = {
  game: GameSnapshot
  busy: boolean
  user: UserInfo | null
  playerRating: RatingEntry | null
  profiles: ProfileRating[]
  activeProfile: ProfileRating | null
  autoMatch: boolean
  matchOffset: number
  // Auto-play props for AI vs AI mode
  isAutoPlaying?: boolean
  autoPlaySpeed?: AutoPlaySpeed
  onMove: (x: number, y: number) => void
  onNewGame: (mode?: GameMode) => void
  onSave: () => void
  onLoad: () => void
  onSelectProfile: (id: string) => void
  onToggleAutoMatch: () => void
  onSetOffset: (offset: number) => void
  onBackClick: () => void
  onSettingsClick: () => void
  onAutoPlayStart?: () => void
  onAutoPlayStop?: () => void
  onAutoPlayStep?: () => void
  onAutoPlaySpeedChange?: (speed: AutoPlaySpeed) => void
}

function PlayPageComponent({
  game,
  busy,
  user,
  playerRating,
  profiles,
  activeProfile,
  autoMatch,
  matchOffset,
  isAutoPlaying = false,
  autoPlaySpeed = 'medium',
  onMove,
  onNewGame,
  onSave,
  onLoad,
  onSelectProfile,
  onToggleAutoMatch,
  onSetOffset,
  onBackClick,
  onSettingsClick,
  onAutoPlayStart,
  onAutoPlayStop,
  onAutoPlayStep,
  onAutoPlaySpeedChange,
}: PlayPageProps) {
  const [showNewGameDialog, setShowNewGameDialog] = useState(false)

  const lastMove = game.moves.length ? game.moves[game.moves.length - 1] : null
  const gameOver = !!game.result
  const isAiVsAi = game.mode.type === 'ai_vs_ai'
  const isHumanVsHuman = game.mode.type === 'human_vs_human'

  const expectedWin = useMemo(() => {
    if (!playerRating || !activeProfile) return null
    if (isAiVsAi || isHumanVsHuman) return null
    const playerRatingValue = playerRating.rating + BLACK_ADVANTAGE
    return expectedScore(playerRatingValue, activeProfile.rating)
  }, [playerRating, activeProfile, isAiVsAi, isHumanVsHuman])

  const handleNewGameClick = useCallback(() => {
    setShowNewGameDialog(true)
  }, [])

  const handleCloseDialog = useCallback(() => {
    setShowNewGameDialog(false)
  }, [])

  const handleStartGame = useCallback(
    (mode: GameMode) => {
      onNewGame(mode)
    },
    [onNewGame],
  )

  // Get opponent info for display
  const opponentDisplay = useMemo(() => {
    if (isAiVsAi) {
      const mode = game.mode as { type: 'ai_vs_ai'; blackId: string; whiteId: string }
      const blackProfile = profiles.find((p) => p.id === mode.blackId)
      const whiteProfile = profiles.find((p) => p.id === mode.whiteId)
      return {
        black: blackProfile?.name ?? mode.blackId,
        white: whiteProfile?.name ?? mode.whiteId,
      }
    }
    return null
  }, [isAiVsAi, game.mode, profiles])

  return (
    <div className="play-page">
      <TopBar
        user={user}
        playerRating={playerRating}
        opponent={activeProfile}
        showBackButton
        onBackClick={onBackClick}
        onSettingsClick={onSettingsClick}
      />
      <div className="play-content">
        <div className="play-main">
          {isAiVsAi && opponentDisplay ? (
            <div className="ai-vs-ai-header">
              <span className="ai-name black">{opponentDisplay.black}</span>
              <span className="vs">vs</span>
              <span className="ai-name white">{opponentDisplay.white}</span>
            </div>
          ) : isHumanVsHuman ? (
            <div className="human-vs-human-header">
              <span>Two Player Mode</span>
            </div>
          ) : (
            <OpponentSelect
              profiles={profiles}
              activeProfile={activeProfile}
              autoMatch={autoMatch}
              matchOffset={matchOffset}
              onSelectProfile={onSelectProfile}
              onToggleAutoMatch={onToggleAutoMatch}
              onSetOffset={onSetOffset}
            />
          )}
          <div className="play-board-area">
            <GameBoard game={game} lastMove={lastMove} onCellClick={onMove} />
            <ResultOverlay result={game.result} onNewGame={handleNewGameClick} />
          </div>
          <GameStatus game={game} busy={busy} expectedWin={expectedWin} />
        </div>
      </div>
      <ActionBar
        gameOver={gameOver}
        mode={game.mode}
        isPlaying={isAutoPlaying}
        autoPlaySpeed={autoPlaySpeed}
        onNewGame={handleNewGameClick}
        onSave={onSave}
        onLoad={onLoad}
        onAutoPlayStart={onAutoPlayStart}
        onAutoPlayStop={onAutoPlayStop}
        onAutoPlayStep={onAutoPlayStep}
        onAutoPlaySpeedChange={onAutoPlaySpeedChange}
      />
      <NewGameDialog
        open={showNewGameDialog}
        profiles={profiles}
        onClose={handleCloseDialog}
        onStartGame={handleStartGame}
      />
    </div>
  )
}

export const PlayPage = React.memo(PlayPageComponent)
