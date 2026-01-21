import { useCallback, useEffect, useMemo, useState } from 'react'
import { open, save } from '@tauri-apps/api/dialog'
import './App.css'

import {
  AppShell,
  PlayPage,
  WelcomePage,
  SettingsHome,
  ProfilePage,
  AIPage,
  DataPage,
  UsersPage,
  NewGameDialog,
} from './components'
import type { GameModeType } from './components'
import { useAutoPlay, useGame, useNavigation, useRating, useSelfPlay, useUsers } from './hooks'
import type { GameMode } from './types'

const formatTimestamp = (date: Date) => {
  const pad = (value: number) => String(value).padStart(2, '0')
  const year = date.getFullYear()
  const month = pad(date.getMonth() + 1)
  const day = pad(date.getDate())
  const hours = pad(date.getHours())
  const minutes = pad(date.getMinutes())
  const seconds = pad(date.getSeconds())
  return `${year}${month}${day}-${hours}${minutes}${seconds}`
}

const joinUserPath = (dir: string, filename: string) => {
  if (dir.endsWith('/') || dir.endsWith('\\')) {
    return `${dir}${filename}`
  }
  return `${dir}/${filename}`
}

function App() {
  const { mode, settingsPage, goToWelcome, goToPlay, goToSettings, goToSettingsPage, goBack } = useNavigation()

  const {
    game,
    busy,
    error: gameError,
    canRetryAi,
    refreshState,
    newGame,
    makeMove,
    requestAiMove,
    saveGame,
    loadGame,
    exportTraining,
    clearError,
  } = useGame()

  const {
    ratings,
    refreshRatings,
    setAutoMatch,
    setActiveProfile,
    createLlmProfile,
    updateLlmProfile,
    deleteLlmProfile,
  } = useRating()

  const { users, activeUser, activeUserDir, refreshUsers, createUser, switchUser, deleteUser, updateUser } =
    useUsers()

  const selfPlay = useSelfPlay(refreshRatings)

  // Auto-play for AI vs AI mode
  const autoPlay = useAutoPlay({
    onStep: async () => {
      await requestAiMove(refreshRatings)
    },
    isGameOver: !!game.result,
    isAiVsAi: game.mode.type === 'ai_vs_ai',
  })

  // State for NewGameDialog opened from welcome page
  const [showNewGameDialogFromWelcome, setShowNewGameDialogFromWelcome] = useState(false)
  const [newGameInitialMode, setNewGameInitialMode] = useState<GameModeType | undefined>(undefined)

  // Initial data fetch
  useEffect(() => {
    const timer = setTimeout(() => {
      void refreshState()
      void refreshRatings()
      void refreshUsers()
    }, 0)
    return () => clearTimeout(timer)
  }, [refreshRatings, refreshState, refreshUsers])

  // Derived state
  const activeProfile = ratings
    ? ratings.profiles.find((p) => p.id === ratings.activeProfile)
    : null

  // Handlers
  const handleNewGame = useCallback(
    async (gameMode?: GameMode) => {
      await newGame(gameMode)
      await refreshRatings()
      goToPlay()

      // If user plays white in human vs AI, AI should move first
      if (gameMode?.type === 'human_vs_ai' && gameMode.humanColor === 'W') {
        setTimeout(() => {
          // force=true to bypass stale game.result check from previous game
          void requestAiMove(refreshRatings, true)
        }, 300)
      }
    },
    [newGame, refreshRatings, goToPlay, requestAiMove],
  )

  const handleMove = useCallback(
    async (x: number, y: number) => {
      await makeMove(x, y, refreshRatings)
    },
    [makeMove, refreshRatings],
  )

  const handleAiMove = useCallback(async () => {
    await requestAiMove(refreshRatings)
  }, [requestAiMove, refreshRatings])

  const handleSave = useCallback(async () => {
    const stamp = formatTimestamp(new Date())
    const defaultPath = activeUserDir
      ? joinUserPath(activeUserDir, `gomoku-${stamp}.json`)
      : `gomoku-${stamp}.json`
    const path = await save({
      filters: [{ name: 'Gomoku JSON', extensions: ['json'] }],
      defaultPath,
    })
    if (path) {
      await saveGame(path)
    }
  }, [activeUserDir, saveGame])

  const handleLoad = useCallback(async () => {
    const defaultPath = activeUserDir ?? undefined
    const path = await open({
      filters: [{ name: 'Gomoku JSON', extensions: ['json'] }],
      multiple: false,
      defaultPath,
    })
    if (typeof path === 'string') {
      await loadGame(path)
      await refreshRatings()
    }
  }, [activeUserDir, loadGame, refreshRatings])

  const handleExport = useCallback(async () => {
    const stamp = formatTimestamp(new Date())
    const defaultPath = activeUserDir
      ? joinUserPath(activeUserDir, `gomoku-training-${stamp}.json`)
      : `gomoku-training-${stamp}.json`
    const path = await save({
      filters: [{ name: 'Training Samples', extensions: ['json'] }],
      defaultPath,
    })
    if (path) {
      await exportTraining(path)
    }
  }, [activeUserDir, exportTraining])

  const handleToggleAutoMatch = useCallback(async () => {
    if (!ratings) return
    await setAutoMatch(!ratings.autoMatch, ratings.matchOffset)
  }, [ratings, setAutoMatch])

  const handleSetOffset = useCallback(
    async (offset: number) => {
      if (!ratings) return
      await setAutoMatch(true, offset)
    },
    [ratings, setAutoMatch],
  )

  const handleSelectProfile = useCallback(
    async (id: string) => {
      await setActiveProfile(id)
    },
    [setActiveProfile],
  )

  const handleCreateUser = useCallback(
    async (name: string) => {
      await createUser(name)
      await refreshRatings()
    },
    [createUser, refreshRatings],
  )

  const handleSwitchUser = useCallback(
    async (id: string) => {
      await switchUser(id)
      await refreshRatings()
    },
    [switchUser, refreshRatings],
  )

  const handleDeleteUser = useCallback(
    async (id: string, deleteData: boolean) => {
      await deleteUser(id, deleteData)
      await refreshRatings()
    },
    [deleteUser, refreshRatings],
  )

  const handleUpdateUser = useCallback(
    async (id: string, name: string) => {
      await updateUser(id, name)
    },
    [updateUser],
  )

  // Error banner
  const error = gameError || selfPlay.error
  const errorBanner = error ? (
    <div className="error-banner">
      <span>{error}</span>
      {canRetryAi && !game.result && (
        <button className="retry-btn" onClick={handleAiMove}>
          Retry
        </button>
      )}
      <button className="dismiss-btn" onClick={clearError}>
        Dismiss
      </button>
    </div>
  ) : null

  // Welcome page handlers
  const handlePlayHumanVsAi = useCallback(
    (humanColor: 'B' | 'W') => {
      void handleNewGame({ type: 'human_vs_ai', humanColor })
    },
    [handleNewGame],
  )

  const handlePlayHumanVsHuman = useCallback(() => {
    void handleNewGame({ type: 'human_vs_human' })
  }, [handleNewGame])

  const handleOpenAiVsAiDialog = useCallback(() => {
    setNewGameInitialMode('ai_vs_ai')
    setShowNewGameDialogFromWelcome(true)
  }, [])

  const handleCloseNewGameDialog = useCallback(() => {
    setShowNewGameDialogFromWelcome(false)
    setNewGameInitialMode(undefined)
  }, [])

  const handleStartGameFromDialog = useCallback(
    (mode: GameMode) => {
      void handleNewGame(mode)
      setShowNewGameDialogFromWelcome(false)
      setNewGameInitialMode(undefined)
    },
    [handleNewGame],
  )

  const handleContinueGame = useCallback(() => {
    goToPlay()
  }, [goToPlay])

  // Welcome page content
  const welcomeContent = (
    <WelcomePage
      user={activeUser ?? null}
      playerRating={ratings?.player ?? null}
      profiles={ratings?.profiles ?? []}
      activeProfile={activeProfile ?? null}
      game={game}
      onPlayHumanVsAi={handlePlayHumanVsAi}
      onPlayHumanVsHuman={handlePlayHumanVsHuman}
      onOpenAiVsAiDialog={handleOpenAiVsAiDialog}
      onContinueGame={handleContinueGame}
      onLoad={handleLoad}
      onSettings={goToSettings}
    />
  )

  // Play mode content
  const playContent = (
    <PlayPage
      game={game}
      busy={busy}
      user={activeUser ?? null}
      playerRating={ratings?.player ?? null}
      profiles={ratings?.profiles ?? []}
      activeProfile={activeProfile ?? null}
      autoMatch={ratings?.autoMatch ?? false}
      matchOffset={ratings?.matchOffset ?? 0}
      isAutoPlaying={autoPlay.isPlaying}
      autoPlaySpeed={autoPlay.speed}
      onMove={handleMove}
      onNewGame={handleNewGame}
      onSave={handleSave}
      onLoad={handleLoad}
      onSelectProfile={handleSelectProfile}
      onToggleAutoMatch={handleToggleAutoMatch}
      onSetOffset={handleSetOffset}
      onBackClick={goToWelcome}
      onSettingsClick={goToSettings}
      onAutoPlayStart={autoPlay.start}
      onAutoPlayStop={autoPlay.stop}
      onAutoPlayStep={autoPlay.step}
      onAutoPlaySpeedChange={autoPlay.setSpeed}
    />
  )

  // Settings mode content
  const settingsContent = useMemo(() => {
    if (settingsPage === 'profile') {
      return (
        <ProfilePage
          ratings={ratings}
          activeUser={activeUser ?? null}
          onBack={goBack}
          onToggleAutoMatch={handleToggleAutoMatch}
          onSetOffset={handleSetOffset}
          onSelectProfile={handleSelectProfile}
        />
      )
    }
    if (settingsPage === 'ai') {
      return (
        <AIPage
          ratings={ratings}
          selfPlayBusy={selfPlay.busy}
          selfPlayProgress={selfPlay.progress}
          selfPlayReport={selfPlay.report}
          selfPlayEta={selfPlay.eta}
          selfPlayGames={selfPlay.gamesPerPair}
          selfPlayParallel={selfPlay.parallelism}
          selfPlayMinLevel={selfPlay.minLevel}
          selfPlayMaxLevel={selfPlay.maxLevel}
          selfPlayIncludeLlm={selfPlay.includeLlm}
          selfPlayLlmIds={selfPlay.llmIds}
          onBack={goBack}
          onCreateLlm={createLlmProfile}
          onUpdateLlm={updateLlmProfile}
          onDeleteLlm={deleteLlmProfile}
          onSelfPlayGamesChange={selfPlay.setGamesPerPair}
          onSelfPlayParallelChange={selfPlay.setParallelism}
          onSelfPlayMinLevelChange={selfPlay.setMinLevel}
          onSelfPlayMaxLevelChange={selfPlay.setMaxLevel}
          onSelfPlayIncludeLlmChange={selfPlay.toggleIncludeLlm}
          onSelfPlayToggleLlmId={selfPlay.toggleLlmId}
          onSelfPlayStart={selfPlay.start}
          onSelfPlayStop={selfPlay.stop}
        />
      )
    }
    if (settingsPage === 'data') {
      return <DataPage activeUserDir={activeUserDir} onBack={goBack} onExport={handleExport} />
    }
    if (settingsPage === 'users') {
      return (
        <UsersPage
          users={users}
          onBack={goBack}
          onCreate={handleCreateUser}
          onSwitch={handleSwitchUser}
          onDelete={handleDeleteUser}
          onUpdate={handleUpdateUser}
        />
      )
    }
    return <SettingsHome onNavigate={goToSettingsPage} onClose={goToWelcome} />
  }, [
    settingsPage,
    ratings,
    activeUser,
    activeUserDir,
    users,
    selfPlay,
    goBack,
    goToWelcome,
    goToSettingsPage,
    handleToggleAutoMatch,
    handleSetOffset,
    handleSelectProfile,
    handleExport,
    handleCreateUser,
    handleSwitchUser,
    handleDeleteUser,
    handleUpdateUser,
    createLlmProfile,
    updateLlmProfile,
    deleteLlmProfile,
  ])

  return (
    <>
      <AppShell
        mode={mode}
        welcomeContent={welcomeContent}
        playContent={playContent}
        settingsContent={settingsContent}
        errorBanner={errorBanner}
      />
      <NewGameDialog
        open={showNewGameDialogFromWelcome}
        profiles={ratings?.profiles ?? []}
        initialMode={newGameInitialMode}
        onClose={handleCloseNewGameDialog}
        onStartGame={handleStartGameFromDialog}
      />
    </>
  )
}

export default App
