import React from 'react'
import type { ProfileRating, SelfPlayProgress, SelfPlayReport } from '../../../types'
import { Button, FormField, ProgressBar } from '../../Shared'

type SelfPlayPanelProps = {
  busy: boolean
  gamesPerPair: number
  parallelism: number
  minLevel: number
  maxLevel: number
  includeLlm: boolean
  llmIds: string[]
  llmProfiles: ProfileRating[]
  progress: SelfPlayProgress | null
  report: SelfPlayReport | null
  eta: string | null
  onGamesPerPairChange: (value: number) => void
  onParallelismChange: (value: number) => void
  onMinLevelChange: (value: number) => void
  onMaxLevelChange: (value: number) => void
  onIncludeLlmChange: (value: boolean) => void
  onToggleLlmId: (id: string, checked: boolean) => void
  onStart: () => void
  onStop: () => void
}

function SelfPlayPanelComponent({
  busy,
  gamesPerPair,
  parallelism,
  minLevel,
  maxLevel,
  includeLlm,
  llmIds,
  llmProfiles,
  progress,
  report,
  eta,
  onGamesPerPairChange,
  onParallelismChange,
  onMinLevelChange,
  onMaxLevelChange,
  onIncludeLlmChange,
  onToggleLlmId,
  onStart,
  onStop,
}: SelfPlayPanelProps) {
  const selectableLlm = llmProfiles.filter((p) => p.kind === 'llm' && p.llm?.apiKeySet)
  const selectedLlmCount = selectableLlm.filter((p) => llmIds.includes(p.id)).length
  const filteredHeuristicCount = maxLevel - minLevel + 1
  const profileCount = filteredHeuristicCount + (includeLlm ? selectedLlmCount : 0)
  const totalPairs = profileCount > 1 ? (profileCount * (profileCount - 1)) / 2 : 0
  const estimatedGames = totalPairs * gamesPerPair

  return (
    <div className="panel">
      <h3>Self-Play Calibration</h3>
      <p>Run AI and optional LLM games to stabilize the ladder. Higher counts take longer.</p>
      <div className="calibrate-grid">
        <FormField label="Games per pair" help="Each profile pair plays this many games.">
          <input
            type="number"
            min={10}
            max={60}
            value={gamesPerPair}
            onChange={(e) => onGamesPerPairChange(Number(e.target.value))}
          />
        </FormField>
        <FormField label="Parallel workers" help="Threads used for background self-play.">
          <input
            type="number"
            min={1}
            max={8}
            value={parallelism}
            onChange={(e) => onParallelismChange(Number(e.target.value))}
          />
        </FormField>
        <div className="field">
          <span className="field-label">Level range</span>
          <div className="field-row">
            <input
              type="number"
              min={1}
              max={maxLevel}
              value={minLevel}
              onChange={(e) => onMinLevelChange(Math.max(1, Math.min(Number(e.target.value), maxLevel)))}
              style={{ width: '60px' }}
            />
            <span style={{ margin: '0 8px' }}>—</span>
            <input
              type="number"
              min={minLevel}
              max={12}
              value={maxLevel}
              onChange={(e) => onMaxLevelChange(Math.max(minLevel, Math.min(Number(e.target.value), 12)))}
              style={{ width: '60px' }}
            />
          </div>
          <span className="field-help">Only calibrate AI levels within this range (1-12).</span>
        </div>
        <label className="toggle-row">
          <input
            type="checkbox"
            checked={includeLlm}
            onChange={(e) => onIncludeLlmChange(e.target.checked)}
          />
          <span>Include LLM profiles in self-play</span>
        </label>
        <div className="button-row">
          <Button variant="primary" onClick={onStart} disabled={busy}>
            {busy ? 'Calibrating...' : 'Run self-play'}
          </Button>
          <Button onClick={onStop} disabled={!busy}>
            Stop
          </Button>
        </div>
      </div>
      {includeLlm && (
        <div className="llm-select">
          {selectableLlm.length === 0 && <span className="muted">No LLM profiles available.</span>}
          {llmProfiles.filter((p) => p.kind === 'llm').map((profile) => {
            const hasKey = profile.llm?.apiKeySet ?? false
            const checked = llmIds.includes(profile.id)
            return (
              <label key={profile.id} className={`toggle-row ${hasKey ? '' : 'disabled'}`}>
                <input
                  type="checkbox"
                  disabled={!hasKey}
                  checked={checked}
                  onChange={(e) => onToggleLlmId(profile.id, e.target.checked)}
                />
                <span>
                  {profile.name} · Elo {Math.round(profile.rating)} {hasKey ? '' : '· API key missing'}
                </span>
              </label>
            )
          })}
        </div>
      )}
      <div className="helper-row">
        <span>
          Levels: L{String(minLevel).padStart(2, '0')}-L{String(maxLevel).padStart(2, '0')} · Profiles: {profileCount} · Pairs: {totalPairs} · Estimated games: {estimatedGames}
        </span>
        {eta && <span>Estimated time remaining: {eta}</span>}
      </div>
      {progress && (
        <ProgressBar
          value={progress.completed}
          max={progress.total}
          label={`${progress.completed}/${progress.total} (${progress.percent.toFixed(1)}%)`}
        />
      )}
      <p>
        {report
          ? `Last run: ${report.completedGames}/${report.totalGames} games (${report.gamesPerPair}/pair)${
              report.stopped ? ' · Stopped early' : ''
            }`
          : 'Run AI vs AI games to stabilize the Elo ladder.'}
      </p>
    </div>
  )
}

export const SelfPlayPanel = React.memo(SelfPlayPanelComponent)
