import React, { useCallback, useMemo, useState } from 'react'
import type { ProfileRating, RatingsSnapshot, SelfPlayProgress, SelfPlayReport, LlmConfig } from '../../../types'
import { BackHeader } from '../../Layout'
import { HeuristicProfiles } from './HeuristicProfiles'
import { LlmProfiles } from './LlmProfiles'
import { LlmProfileForm, type LlmFormState } from './LlmProfileForm'
import { SelfPlayPanel } from './SelfPlayPanel'

const defaultLlmForm: LlmFormState = {
  id: null,
  name: '',
  baseUrl: '',
  model: 'gpt-4o-mini',
  temperature: 0.4,
  topP: 1,
  maxTokens: 128,
  timeout: 20000,
  candidateLimit: 12,
  apiKey: '',
}

type AIPageProps = {
  ratings: RatingsSnapshot | null
  selfPlayBusy: boolean
  selfPlayProgress: SelfPlayProgress | null
  selfPlayReport: SelfPlayReport | null
  selfPlayEta: string | null
  selfPlayGames: number
  selfPlayParallel: number
  selfPlayMinLevel: number
  selfPlayMaxLevel: number
  selfPlayIncludeLlm: boolean
  selfPlayLlmIds: string[]
  onBack: () => void
  onCreateLlm: (name: string, config: LlmConfig, apiKey: string) => Promise<boolean>
  onUpdateLlm: (id: string, name: string, config: LlmConfig, apiKey: string | null) => Promise<boolean>
  onDeleteLlm: (id: string, deleteKey: boolean) => void
  onSelfPlayGamesChange: (value: number) => void
  onSelfPlayParallelChange: (value: number) => void
  onSelfPlayMinLevelChange: (value: number) => void
  onSelfPlayMaxLevelChange: (value: number) => void
  onSelfPlayIncludeLlmChange: (value: boolean, profiles: ProfileRating[]) => void
  onSelfPlayToggleLlmId: (id: string, checked: boolean) => void
  onSelfPlayStart: () => void
  onSelfPlayStop: () => void
}

function AIPageComponent({
  ratings,
  selfPlayBusy,
  selfPlayProgress,
  selfPlayReport,
  selfPlayEta,
  selfPlayGames,
  selfPlayParallel,
  selfPlayMinLevel,
  selfPlayMaxLevel,
  selfPlayIncludeLlm,
  selfPlayLlmIds,
  onBack,
  onCreateLlm,
  onUpdateLlm,
  onDeleteLlm,
  onSelfPlayGamesChange,
  onSelfPlayParallelChange,
  onSelfPlayMinLevelChange,
  onSelfPlayMaxLevelChange,
  onSelfPlayIncludeLlmChange,
  onSelfPlayToggleLlmId,
  onSelfPlayStart,
  onSelfPlayStop,
}: AIPageProps) {
  const [llmForm, setLlmForm] = useState<LlmFormState>(defaultLlmForm)
  const [deleteKey, setDeleteKey] = useState(true)

  const profiles = useMemo(() => ratings?.profiles ?? [], [ratings?.profiles])
  const llmProfiles = useMemo(() => profiles.filter((p) => p.kind === 'llm'), [profiles])

  const handleEditLlm = useCallback((profile: ProfileRating) => {
    setLlmForm({
      id: profile.id,
      name: profile.name,
      baseUrl: profile.llm?.baseUrl ?? '',
      model: profile.llm?.model ?? 'gpt-4o-mini',
      temperature: profile.llm?.temperature ?? 0.4,
      topP: profile.llm?.topP ?? 1,
      maxTokens: profile.llm?.maxTokens ?? 128,
      timeout: profile.llm?.timeoutMs ?? 20000,
      candidateLimit: profile.llm?.candidateLimit ?? 12,
      apiKey: '',
    })
  }, [])

  const handleSaveLlm = useCallback(async () => {
    if (!llmForm.name.trim()) return
    const config: LlmConfig = {
      baseUrl: llmForm.baseUrl.trim(),
      model: llmForm.model.trim(),
      temperature: llmForm.temperature,
      topP: llmForm.topP,
      maxTokens: llmForm.maxTokens,
      timeoutMs: llmForm.timeout,
      candidateLimit: llmForm.candidateLimit,
      apiKeySet: false,
    }
    let success = false
    if (llmForm.id) {
      success = await onUpdateLlm(
        llmForm.id,
        llmForm.name.trim(),
        config,
        llmForm.apiKey.length ? llmForm.apiKey : null,
      )
    } else {
      success = await onCreateLlm(llmForm.name.trim(), config, llmForm.apiKey)
    }
    if (success) {
      setLlmForm(defaultLlmForm)
    }
  }, [llmForm, onCreateLlm, onUpdateLlm])

  const handleCancelLlm = useCallback(() => {
    setLlmForm(defaultLlmForm)
  }, [])

  const handleDeleteLlm = useCallback(
    (id: string) => {
      onDeleteLlm(id, deleteKey)
      if (llmForm.id === id) {
        setLlmForm(defaultLlmForm)
      }
    },
    [deleteKey, llmForm.id, onDeleteLlm],
  )

  const handleIncludeLlmChange = useCallback(
    (value: boolean) => {
      onSelfPlayIncludeLlmChange(value, profiles)
    },
    [onSelfPlayIncludeLlmChange, profiles],
  )

  return (
    <div className="settings-page">
      <BackHeader title="AI Configuration" subtitle="Settings" onBack={onBack} />
      <div className="settings-page-content">
        <HeuristicProfiles
          profiles={profiles}
          activeProfileId={ratings?.activeProfile ?? null}
        />
        <LlmProfiles
          profiles={profiles}
          onEdit={handleEditLlm}
          onDelete={handleDeleteLlm}
        />
        <div className="panel">
          <h3>{llmForm.id ? 'Edit LLM Profile' : 'Create LLM Profile'}</h3>
          <LlmProfileForm
            form={llmForm}
            onChange={(updates) => setLlmForm((prev) => ({ ...prev, ...updates }))}
            onSave={handleSaveLlm}
            onCancel={handleCancelLlm}
            deleteKey={deleteKey}
            onDeleteKeyChange={setDeleteKey}
          />
        </div>
        <SelfPlayPanel
          busy={selfPlayBusy}
          gamesPerPair={selfPlayGames}
          parallelism={selfPlayParallel}
          minLevel={selfPlayMinLevel}
          maxLevel={selfPlayMaxLevel}
          includeLlm={selfPlayIncludeLlm}
          llmIds={selfPlayLlmIds}
          llmProfiles={llmProfiles}
          progress={selfPlayProgress}
          report={selfPlayReport}
          eta={selfPlayEta}
          onGamesPerPairChange={onSelfPlayGamesChange}
          onParallelismChange={onSelfPlayParallelChange}
          onMinLevelChange={onSelfPlayMinLevelChange}
          onMaxLevelChange={onSelfPlayMaxLevelChange}
          onIncludeLlmChange={handleIncludeLlmChange}
          onToggleLlmId={onSelfPlayToggleLlmId}
          onStart={onSelfPlayStart}
          onStop={onSelfPlayStop}
        />
      </div>
    </div>
  )
}

export const AIPage = React.memo(AIPageComponent)
