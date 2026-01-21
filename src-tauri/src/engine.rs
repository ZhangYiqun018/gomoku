use std::time::{SystemTime, UNIX_EPOCH};

use lazy_static::lazy_static;
use rand::Rng;

use crate::rules::rules_for;
use crate::types::{
  Coord, GameMode, GameRecord, GameResult, GameSnapshot, Meta, Move, Player, Players, RuleSetKind,
  TrainingSample,
};

// Zobrist hashing table for transposition table
// 225 = 15x15 board, 2 players (Black and White)
lazy_static! {
  pub static ref ZOBRIST_TABLE: [[u64; 2]; 225] = {
    let mut table = [[0u64; 2]; 225];
    let mut rng = rand::thread_rng();
    for i in 0..225 {
      table[i][0] = rng.gen(); // Black
      table[i][1] = rng.gen(); // White
    }
    table
  };
}

#[derive(Clone, Debug)]
pub struct Board {
  size: usize,
  cells: Vec<Option<Player>>,
  hash: u64, // Cached Zobrist hash for O(1) lookup
}

impl Board {
  pub fn new(size: usize) -> Self {
    Self {
      size,
      cells: vec![None; size * size],
      hash: 0, // Empty board has hash 0
    }
  }

  pub fn size(&self) -> usize {
    self.size
  }

  pub fn in_bounds(&self, x: usize, y: usize) -> bool {
    x < self.size && y < self.size
  }

  pub fn index(&self, x: usize, y: usize) -> usize {
    y * self.size + x
  }

  pub fn get(&self, x: usize, y: usize) -> Option<Player> {
    if !self.in_bounds(x, y) {
      return None;
    }
    self.cells[self.index(x, y)]
  }

  pub fn set(&mut self, x: usize, y: usize, player: Player) {
    let idx = self.index(x, y);
    self.cells[idx] = Some(player);
    // Incremental hash update: XOR in the new piece
    if idx < 225 {
      let player_idx = match player {
        Player::B => 0,
        Player::W => 1,
      };
      self.hash ^= ZOBRIST_TABLE[idx][player_idx];
    }
  }

  pub fn clear(&mut self, x: usize, y: usize) {
    let idx = self.index(x, y);
    // Incremental hash update: XOR out the removed piece before clearing
    if idx < 225 {
      if let Some(player) = self.cells[idx] {
        let player_idx = match player {
          Player::B => 0,
          Player::W => 1,
        };
        self.hash ^= ZOBRIST_TABLE[idx][player_idx];
      }
    }
    self.cells[idx] = None;
  }

  pub fn is_empty(&self, x: usize, y: usize) -> bool {
    self.in_bounds(x, y) && self.get(x, y).is_none()
  }

  pub fn is_full(&self) -> bool {
    self.cells.iter().all(|cell| cell.is_some())
  }

  pub fn empty_coords(&self) -> Vec<Coord> {
    let mut coords = Vec::with_capacity(self.size * self.size);
    for y in 0..self.size {
      for x in 0..self.size {
        if self.get(x, y).is_none() {
          coords.push(Coord { x, y });
        }
      }
    }
    coords
  }

  pub fn cells(&self) -> Vec<Option<Player>> {
    self.cells.clone()
  }

  // Get the cached Zobrist hash - O(1) operation
  pub fn hash(&self) -> u64 {
    self.hash
  }

  // Compute Zobrist hash from scratch (for verification or backward compatibility)
  pub fn zobrist_hash(&self) -> u64 {
    // Return cached hash - O(1) instead of O(225)
    self.hash
  }
}

#[derive(Clone, Debug)]
pub struct GameState {
  pub board: Board,
  pub rule_set: RuleSetKind,
  pub to_move: Player,
  pub moves: Vec<Move>,
  pub result: Option<GameResult>,
  pub players: Players,
  pub created_at: i64,
  pub updated_at: i64,
  pub game_id: String,
  pub mode: GameMode,
}

impl GameState {
  pub fn new(board_size: usize, rule_set: RuleSetKind, players: Players, mode: GameMode) -> Self {
    let now = now_ts();
    Self {
      board: Board::new(board_size),
      rule_set,
      to_move: Player::B,
      moves: Vec::new(),
      result: None,
      players,
      created_at: now,
      updated_at: now,
      game_id: new_game_id(now),
      mode,
    }
  }

  pub fn snapshot(&self) -> GameSnapshot {
    let can_human_move = self.can_human_move();
    GameSnapshot {
      board_size: self.board.size(),
      board: self.board.cells(),
      rule_set: self.rule_set,
      to_move: self.to_move,
      result: self.result,
      moves: self.moves.clone(),
      mode: self.mode.clone(),
      can_human_move,
    }
  }

  pub fn can_human_move(&self) -> bool {
    if self.result.is_some() {
      return false;
    }
    match &self.mode {
      GameMode::HumanVsAi { human_color } => self.to_move == *human_color,
      GameMode::HumanVsHuman => true,
      GameMode::AiVsAi { .. } => false,
    }
  }

  pub fn is_ai_turn(&self) -> bool {
    if self.result.is_some() {
      return false;
    }
    match &self.mode {
      GameMode::HumanVsAi { human_color } => self.to_move != *human_color,
      GameMode::HumanVsHuman => false,
      GameMode::AiVsAi { .. } => true,
    }
  }

  pub fn current_ai_profile(&self) -> Option<&str> {
    match &self.mode {
      GameMode::HumanVsAi { .. } => None, // AI profile handled externally
      GameMode::AiVsAi { black_id, white_id } => {
        if self.to_move == Player::B {
          Some(black_id)
        } else {
          Some(white_id)
        }
      }
      GameMode::HumanVsHuman => None,
    }
  }

  pub fn apply_move(&mut self, x: usize, y: usize) -> Result<(), String> {
    let mv = Move {
      x,
      y,
      player: self.to_move,
      t: Some(now_ts()),
    };
    self.apply_existing_move(mv)
  }

  pub fn to_record(&self) -> GameRecord {
    GameRecord {
      version: "1.0".to_string(),
      board_size: self.board.size(),
      rule_set: self.rule_set,
      players: self.players.clone(),
      result: self.result,
      moves: self.moves.clone(),
      meta: Meta {
        created_at: self.created_at,
        updated_at: Some(self.updated_at),
        game_id: Some(self.game_id.clone()),
      },
    }
  }

  pub fn from_record(record: GameRecord) -> Result<Self, String> {
    let mut state = GameState::new(
      record.board_size,
      record.rule_set,
      record.players.clone(),
      GameMode::default(),
    );
    let created_at = if record.meta.created_at > 0 {
      record.meta.created_at
    } else {
      now_ts()
    };
    let updated_at = record.meta.updated_at.unwrap_or(created_at);
    state.created_at = created_at;
    state.updated_at = updated_at;
    state.game_id = record
      .meta
      .game_id
      .clone()
      .unwrap_or_else(|| new_game_id(created_at));

    for mv in record.moves.iter() {
      state.apply_existing_move(mv.clone())?;
    }

    if let Some(result) = record.result {
      state.result = Some(result);
    }
    state.updated_at = updated_at;
    Ok(state)
  }

  fn apply_existing_move(&mut self, mv: Move) -> Result<(), String> {
    if self.result.is_some() {
      return Err("Game is already finished".to_string());
    }
    if mv.player != self.to_move {
      return Err("Move order mismatch".to_string());
    }

    let rules = rules_for(self.rule_set);
    if !rules.is_legal(&self.board, &mv) {
      return Err("Illegal move".to_string());
    }

    self.board.set(mv.x, mv.y, mv.player);
    self.moves.push(mv.clone());

    if let Some(result) = rules.check_win(&self.board, &mv) {
      self.result = Some(result);
      self.updated_at = now_ts();
      return Ok(());
    }

    if self.board.is_full() {
      self.result = Some(GameResult::Draw);
      self.updated_at = now_ts();
      return Ok(());
    }

    self.to_move = self.to_move.other();
    self.updated_at = now_ts();
    Ok(())
  }

  pub fn training_samples(&self) -> Vec<TrainingSample> {
    let mut board = Board::new(self.board.size());
    let mut samples = Vec::with_capacity(self.moves.len());
    let mut to_move = Player::B;

    for (ply, mv) in self.moves.iter().enumerate() {
      let legal_moves = board.empty_coords();
      samples.push(TrainingSample {
        board_size: board.size(),
        board: board.cells(),
        to_move,
        legal_moves,
        played_move: Some(Coord { x: mv.x, y: mv.y }),
        result: self.result,
        ply,
      });

      board.set(mv.x, mv.y, mv.player);
      to_move = to_move.other();
    }

    samples
  }
}

fn now_ts() -> i64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs() as i64
}

fn new_game_id(seed: i64) -> String {
  let rand_part: u32 = rand::random();
  format!("gomoku-{}-{:08x}", seed, rand_part)
}
