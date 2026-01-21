export type Player = 'B' | 'W'
export type GameResult = 'B_WIN' | 'W_WIN' | 'DRAW'
export type RuleSetKind = 'standard'
export type ProfileKind = 'heuristic' | 'llm'

export type Move = {
  x: number
  y: number
  player: Player
  t?: number
}

export type GameModeHumanVsAi = {
  type: 'human_vs_ai'
  humanColor: Player
}

export type GameModeAiVsAi = {
  type: 'ai_vs_ai'
  blackId: string
  whiteId: string
}

export type GameModeHumanVsHuman = {
  type: 'human_vs_human'
}

export type GameMode = GameModeHumanVsAi | GameModeAiVsAi | GameModeHumanVsHuman

export type GameSnapshot = {
  boardSize: number
  board: Array<Player | null>
  ruleSet: RuleSetKind
  toMove: Player
  result: GameResult | null
  moves: Move[]
  mode: GameMode
  canHumanMove: boolean
}

export type RatingEntry = {
  rating: number
  games: number
  wins: number
  draws: number
  losses: number
}

export type AiConfig = {
  depth: number
  maxCandidates: number
  randomness: number
  maxNodes: number
  defenseWeight: number
}

export type LlmConfig = {
  baseUrl: string
  model: string
  temperature: number
  topP: number
  maxTokens: number
  timeoutMs: number
  candidateLimit: number
  apiKeySet: boolean
}

export type ProfileRating = {
  id: string
  name: string
  rating: number
  games: number
  wins: number
  draws: number
  losses: number
  kind: ProfileKind
  config?: AiConfig
  llm?: LlmConfig
}

export type RatingsSnapshot = {
  player: RatingEntry
  profiles: ProfileRating[]
  activeProfile: string
  autoMatch: boolean
  matchOffset: number
}

export type UserInfo = {
  id: string
  name: string
  createdAt: string
  dataDir: string
}

export type UsersSnapshot = {
  activeUser: string
  users: UserInfo[]
}

export type SelfPlayReport = {
  gamesPerPair: number
  totalGames: number
  completedGames: number
  stopped: boolean
}

export type SelfPlayProgress = {
  completed: number
  total: number
  percent: number
}

export type MainMenu = 'Game' | 'Rating' | 'AI' | 'Data' | 'Users'
export type LadderFilter = 'all' | 'user' | 'ai' | 'llm'

export type AppMode = 'welcome' | 'play' | 'settings'
export type SettingsPage = 'home' | 'profile' | 'ai' | 'data' | 'users'

export type LadderEntryKind = 'heuristic' | 'llm' | 'user'

export type LadderEntry = {
  id: string
  name: string
  kind: LadderEntryKind
  rating: number
  games: number
  wins: number
  draws: number
  losses: number
}

export const formatRecord = (wins: number, draws: number, losses: number) =>
  `${wins}-${draws}-${losses}`

export const formatWinRate = (wins: number, draws: number, losses: number) => {
  const total = wins + draws + losses
  if (total === 0) return '—'
  const score = (wins + draws * 0.5) / total
  return `${(score * 100).toFixed(1)}%`
}

export const emptyBoard = (size: number) => Array.from({ length: size * size }, () => null)

export const defaultGameMode: GameMode = { type: 'human_vs_ai', humanColor: 'B' }

export const BLACK_ADVANTAGE = 35

export const expectedScore = (ratingA: number, ratingB: number) =>
  1 / (1 + Math.pow(10, (ratingB - ratingA) / 400))
