use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Player {
  B,
  W,
}

impl Player {
  pub fn other(self) -> Self {
    match self {
      Player::B => Player::W,
      Player::W => Player::B,
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleSetKind {
  Standard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GameResult {
  BWin,
  WWin,
  Draw,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Move {
  pub x: usize,
  pub y: usize,
  pub player: Player,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub t: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Players {
  pub black: String,
  pub white: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
  #[serde(default)]
  pub created_at: i64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub updated_at: Option<i64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub game_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameRecord {
  pub version: String,
  pub board_size: usize,
  pub rule_set: RuleSetKind,
  pub players: Players,
  pub result: Option<GameResult>,
  pub moves: Vec<Move>,
  pub meta: Meta,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSnapshot {
  pub board_size: usize,
  pub board: Vec<Option<Player>>,
  pub rule_set: RuleSetKind,
  pub to_move: Player,
  pub result: Option<GameResult>,
  pub moves: Vec<Move>,
  pub mode: GameMode,
  pub can_human_move: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
  pub depth: u8,
  pub max_candidates: usize,
  pub randomness: u8,
  pub max_nodes: u32,
  pub defense_weight: i32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProfileKind {
  Heuristic,
  Llm,
}

impl Default for ProfileKind {
  fn default() -> Self {
    ProfileKind::Heuristic
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
  #[serde(default)]
  pub base_url: String,
  pub model: String,
  #[serde(default = "default_temperature")]
  pub temperature: f32,
  #[serde(default = "default_top_p")]
  pub top_p: f32,
  #[serde(default = "default_max_tokens")]
  pub max_tokens: u32,
  #[serde(default = "default_timeout_ms")]
  pub timeout_ms: u64,
  #[serde(default = "default_candidate_limit")]
  pub candidate_limit: usize,
  #[serde(default)]
  pub api_key_set: bool,
}

fn default_temperature() -> f32 {
  0.4
}

fn default_top_p() -> f32 {
  1.0
}

fn default_max_tokens() -> u32 {
  128
}

fn default_timeout_ms() -> u64 {
  20000
}

fn default_candidate_limit() -> usize {
  12
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Coord {
  pub x: usize,
  pub y: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GameMode {
  #[serde(rename_all = "camelCase")]
  HumanVsAi { human_color: Player },
  #[serde(rename_all = "camelCase")]
  AiVsAi { black_id: String, white_id: String },
  HumanVsHuman,
}

impl Default for GameMode {
  fn default() -> Self {
    GameMode::HumanVsAi {
      human_color: Player::B,
    }
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingSample {
  pub board_size: usize,
  pub board: Vec<Option<Player>>,
  pub to_move: Player,
  pub legal_moves: Vec<Coord>,
  pub played_move: Option<Coord>,
  pub result: Option<GameResult>,
  pub ply: usize,
}
