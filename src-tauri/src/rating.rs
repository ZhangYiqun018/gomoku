use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::ai;
use crate::engine::GameState;
use crate::llm;
use crate::types::{AiConfig, GameMode, GameResult, LlmConfig, Player, Players, ProfileKind, RuleSetKind};

const RATINGS_VERSION: u32 = 1;
const DEFAULT_PLAYER_RATING: f64 = 1000.0;
const BLACK_ADVANTAGE: f64 = 35.0;
const BATCH_SAVE_SIZE: u32 = 10; // Save to disk every N games for better I/O efficiency

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingEntry {
  pub rating: f64,
  pub games: u32,
  #[serde(default)]
  pub wins: u32,
  #[serde(default)]
  pub draws: u32,
  #[serde(default)]
  pub losses: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRating {
  pub id: String,
  pub name: String,
  pub rating: f64,
  pub games: u32,
  #[serde(default)]
  pub wins: u32,
  #[serde(default)]
  pub draws: u32,
  #[serde(default)]
  pub losses: u32,
  #[serde(default)]
  pub kind: ProfileKind,
  #[serde(default)]
  pub config: Option<AiConfig>,
  #[serde(default)]
  pub llm: Option<LlmConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingStore {
  pub version: u32,
  pub player: RatingEntry,
  pub profiles: Vec<ProfileRating>,
  #[serde(default)]
  pub extras: Vec<ProfileRating>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatingsSnapshot {
  pub player: RatingEntry,
  pub profiles: Vec<ProfileRating>,
  pub active_profile: String,
  pub auto_match: bool,
  pub match_offset: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfPlayReport {
  pub games_per_pair: u32,
  pub total_games: u32,
  pub completed_games: u32,
  pub stopped: bool,
}

impl Default for RatingStore {
  fn default() -> Self {
    Self {
      version: RATINGS_VERSION,
      player: RatingEntry {
        rating: DEFAULT_PLAYER_RATING,
        games: 0,
        wins: 0,
        draws: 0,
        losses: 0,
      },
      profiles: default_profiles(),
      extras: Vec::new(),
    }
  }
}

impl RatingStore {
  pub fn load_or_default(path: &Path) -> Self {
    if let Ok(data) = fs::read_to_string(path) {
      if let Ok(mut store) = serde_json::from_str::<RatingStore>(&data) {
        store.ensure_profiles();
        return store;
      }
    }

    let mut store = RatingStore::default();
    store.ensure_profiles();
    store
  }

  pub fn load_or_default_user(path: &Path) -> Self {
    if let Ok(data) = fs::read_to_string(path) {
      if let Ok(mut store) = serde_json::from_str::<RatingStore>(&data) {
        store.ensure_profiles();
        return store;
      }
    }

    let mut store = RatingStore::default();
    store.player.rating = DEFAULT_PLAYER_RATING;
    store.player.games = 0;
    store.player.wins = 0;
    store.player.draws = 0;
    store.player.losses = 0;
    for profile in store.profiles.iter_mut() {
      profile.rating = 0.0;
      profile.games = 0;
      profile.wins = 0;
      profile.draws = 0;
      profile.losses = 0;
      profile.kind = ProfileKind::Heuristic;
      profile.llm = None;
    }
    store
  }

  pub fn save(&self, path: &Path) -> Result<(), String> {
    let data = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())
  }

  pub fn ensure_profiles(&mut self) {
    let defaults = default_profiles();
    let mut merged = Vec::new();

    for def in defaults.into_iter() {
      if let Some(existing) = self.profiles.iter().find(|p| p.id == def.id) {
        merged.push(ProfileRating {
          id: def.id,
          name: def.name,
          rating: existing.rating,
          games: existing.games,
          wins: existing.wins,
          draws: existing.draws,
          losses: existing.losses,
          kind: ProfileKind::Heuristic,
          config: def.config,
          llm: None,
        });
      } else {
        merged.push(def);
      }
    }

    self.profiles = merged;
  }

  pub fn get_profile(&self, id: &str) -> Option<&ProfileRating> {
    self.profiles.iter().find(|p| p.id == id)
  }

  pub fn get_profile_any(&self, id: &str) -> Option<&ProfileRating> {
    self
      .profiles
      .iter()
      .find(|p| p.id == id)
      .or_else(|| self.extras.iter().find(|p| p.id == id))
  }

  pub fn match_profile_id(&self, offset: i32) -> Option<String> {
    let target = self.player.rating + offset as f64;
    self
      .profiles
      .iter()
      .min_by(|a, b| {
        let da = (a.rating - target).abs();
        let db = (b.rating - target).abs();
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
      })
      .map(|p| p.id.clone())
  }

  pub fn update_player_vs_profile_user(
    &mut self,
    base: &RatingStore,
    profile_id: &str,
    result: GameResult,
    player_color: Player,
  ) -> Result<(), String> {
    let score_player = score_for_result(result, player_color);

    let idx = base
      .profiles
      .iter()
      .position(|p| p.id == profile_id)
      .ok_or_else(|| "Unknown profile".to_string())?;

    if idx >= self.profiles.len() {
      return Err("Profile index out of range".to_string());
    }

    let base_profile = &base.profiles[idx];
    let user_profile = &self.profiles[idx];

    let player_rating = self.player.rating;
    let player_games = self.player.games;
    let effective_profile_rating = base_profile.rating + user_profile.rating;
    let effective_profile_games = base_profile.games + user_profile.games;

    let (player_adjusted, profile_adjusted) =
      adjust_for_color(player_rating, effective_profile_rating, player_color);
    let expected_player = expected_score(player_adjusted, profile_adjusted);
    let expected_profile = 1.0 - expected_player;

    let k_player = k_factor(player_games);
    let k_profile = k_factor(effective_profile_games);

    let new_player = apply_rating(player_rating, score_player, expected_player, k_player);
    let new_profile = apply_rating(
      effective_profile_rating,
      1.0 - score_player,
      expected_profile,
      k_profile,
    );

    self.player.rating = new_player;
    self.player.games += 1;
    apply_result_to_entry(&mut self.player, result, player_color);
    let profile = &mut self.profiles[idx];
    profile.rating = new_profile - base_profile.rating;
    profile.games += 1;
    apply_result_to_profile(profile, result, player_color.other());

    Ok(())
  }

  pub fn update_player_vs_llm(
    &mut self,
    profile_id: &str,
    result: GameResult,
    player_color: Player,
  ) -> Result<(), String> {
    let idx = self
      .extras
      .iter()
      .position(|p| p.id == profile_id)
      .ok_or_else(|| "Unknown profile".to_string())?;
    let profile = &self.extras[idx];

    let player_rating = self.player.rating;
    let profile_rating = profile.rating;
    let (player_adjusted, profile_adjusted) =
      adjust_for_color(player_rating, profile_rating, player_color);
    let expected_player = expected_score(player_adjusted, profile_adjusted);
    let expected_profile = 1.0 - expected_player;

    let k_player = k_factor(self.player.games);
    let k_profile = k_factor(profile.games);

    let new_player = apply_rating(
      player_rating,
      score_for_result(result, player_color),
      expected_player,
      k_player,
    );
    let new_profile = apply_rating(
      profile_rating,
      1.0 - score_for_result(result, player_color),
      expected_profile,
      k_profile,
    );

    self.player.rating = new_player;
    self.player.games += 1;
    apply_result_to_entry(&mut self.player, result, player_color);
    let profile = &mut self.extras[idx];
    profile.rating = new_profile;
    profile.games += 1;
    apply_result_to_profile(profile, result, player_color.other());

    Ok(())
  }

  fn update_profile_by_index(
    &mut self,
    idx_a: usize,
    idx_b: usize,
    score_a: f64,
  ) -> Result<(), String> {
    if idx_a == idx_b {
      return Err("Profiles must be different".to_string());
    }
    if idx_a >= self.profiles.len() || idx_b >= self.profiles.len() {
      return Err("Profile index out of range".to_string());
    }

    let (rating_a, games_a) = {
      let profile = &self.profiles[idx_a];
      (profile.rating, profile.games)
    };
    let (rating_b, games_b) = {
      let profile = &self.profiles[idx_b];
      (profile.rating, profile.games)
    };

    let expected_a = expected_score(rating_a + BLACK_ADVANTAGE, rating_b);
    let expected_b = 1.0 - expected_a;

    let k_a = k_factor(games_a);
    let k_b = k_factor(games_b);

    let new_a = apply_rating(rating_a, score_a, expected_a, k_a);
    let new_b = apply_rating(rating_b, 1.0 - score_a, expected_b, k_b);

    if idx_a < idx_b {
      let (left, right) = self.profiles.split_at_mut(idx_b);
      let profile_a = &mut left[idx_a];
      let profile_b = &mut right[0];
      profile_a.rating = new_a;
      profile_b.rating = new_b;
      profile_a.games += 1;
      profile_b.games += 1;
      apply_score_to_profile(profile_a, score_a);
      apply_score_to_profile(profile_b, 1.0 - score_a);
    } else {
      let (left, right) = self.profiles.split_at_mut(idx_a);
      let profile_b = &mut left[idx_b];
      let profile_a = &mut right[0];
      profile_a.rating = new_a;
      profile_b.rating = new_b;
      profile_a.games += 1;
      profile_b.games += 1;
      apply_score_to_profile(profile_a, score_a);
      apply_score_to_profile(profile_b, 1.0 - score_a);
    }

    Ok(())
  }
}

pub fn ratings_base_path() -> PathBuf {
  let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  manifest_dir
    .parent()
    .unwrap_or(&manifest_dir)
    .join("ratings_base.json")
}

pub fn run_self_play(
  store: &mut RatingStore,
  save_path: &Path,
  games_per_pair: u32,
  parallelism: usize,
  stop_flag: Arc<AtomicBool>,
  mut on_progress: impl FnMut(u32, u32),
  min_level: u8,
  max_level: u8,
) -> Result<SelfPlayReport, String> {
  // Filter profiles by level range
  let filtered_indices: Vec<usize> = store
    .profiles
    .iter()
    .enumerate()
    .filter(|(_, p)| {
      p.id
        .strip_prefix('l')
        .and_then(|s| s.parse::<u8>().ok())
        .map(|level| level >= min_level && level <= max_level)
        .unwrap_or(false)
    })
    .map(|(idx, _)| idx)
    .collect();

  let profile_count = filtered_indices.len();
  if profile_count < 2 || games_per_pair == 0 {
    on_progress(0, 0);
    return Ok(SelfPlayReport {
      games_per_pair,
      total_games: 0,
      completed_games: 0,
      stopped: false,
    });
  }

  // Build pairs using local indices (0..profile_count), then map back to original indices
  let mut local_pairs = build_pairs(profile_count);
  let mut rng = StdRng::seed_from_u64(42);
  local_pairs.shuffle(&mut rng);
  let pairs: Vec<(usize, usize)> = local_pairs
    .into_iter()
    .map(|(a, b)| (filtered_indices[a], filtered_indices[b]))
    .collect();
  let total_games = pairs.len() as u32 * games_per_pair;
  on_progress(0, total_games);

  let configs: Vec<AiConfig> = store
    .profiles
    .iter()
    .map(|p| p.config.ok_or_else(|| "Missing AI config".to_string()))
    .collect::<Result<Vec<_>, _>>()?;
  let pair_list = std::sync::Arc::new(pairs);
  let config_list = std::sync::Arc::new(configs);
  let games_per_pair_usize = games_per_pair as usize;

  let (tx, rx) = std::sync::mpsc::channel::<Result<JobResult, String>>();
  let index = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
  let total_jobs = total_games as usize;
  let worker_count = usize::max(1, usize::min(parallelism, total_jobs));
  let mut handles = Vec::new();

  for _ in 0..worker_count {
    let tx = tx.clone();
    let pair_list = pair_list.clone();
    let config_list = config_list.clone();
    let index = index.clone();
    let total_pairs = pair_list.len();
    let stop_flag = stop_flag.clone();
    handles.push(std::thread::spawn(move || loop {
      if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
        break;
      }
      let idx = index.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
      if idx >= total_jobs {
        break;
      }
      let pair_idx = idx / games_per_pair_usize;
      let game_idx = idx % games_per_pair_usize;
      if pair_idx >= total_pairs {
        break;
      }
      let (a, b) = pair_list[pair_idx];
      let (black_idx, white_idx) = if game_idx % 2 == 0 { (a, b) } else { (b, a) };
      let black = config_list[black_idx];
      let white = config_list[white_idx];
      let result = play_ai_game(black, white).map(|result| JobResult {
        black_idx,
        white_idx,
        result,
      });
      let _ = tx.send(result);
    }));
  }
  drop(tx);

  let mut completed = 0u32;
  let mut pending_saves = 0u32;
  for msg in rx {
    let result = msg?;
    let score_black = score_for_result(result.result, Player::B);
    store.update_profile_by_index(result.black_idx, result.white_idx, score_black)?;
    completed += 1;
    pending_saves += 1;

    // Batch write: save every BATCH_SAVE_SIZE games instead of every game
    if pending_saves >= BATCH_SAVE_SIZE {
      store.save(save_path)?;
      pending_saves = 0;
    }
    on_progress(completed, total_games);
  }

  // Final save for any remaining games
  if pending_saves > 0 {
    store.save(save_path)?;
  }

  for handle in handles {
    let _ = handle.join();
  }

  let stopped = stop_flag.load(std::sync::atomic::Ordering::Relaxed) && completed < total_games;

  Ok(SelfPlayReport {
    games_per_pair,
    total_games,
    completed_games: completed,
    stopped,
  })
}

#[derive(Clone, Debug)]
enum MixedSide {
  Heuristic(usize),
  Llm(String),
}

#[derive(Clone, Debug)]
struct MixedEntry {
  side: MixedSide,
  rating: f64,
  games: u32,
  config: Option<AiConfig>,
  llm: Option<LlmConfig>,
}

#[derive(Clone, Debug)]
struct MixedJobResult {
  black_idx: usize,
  white_idx: usize,
  result: GameResult,
}

pub fn run_self_play_mixed(
  base: &RatingStore,
  user: &mut RatingStore,
  llm_keys: &std::collections::HashMap<String, String>,
  games_per_pair: u32,
  parallelism: usize,
  llm_ids: &[String],
  stop_flag: Arc<AtomicBool>,
  mut on_progress: impl FnMut(u32, u32),
  save_path: &Path,
  min_level: u8,
  max_level: u8,
) -> Result<SelfPlayReport, String> {
  let mut entries = Vec::new();
  for (idx, profile) in base.profiles.iter().enumerate() {
    // Filter heuristic profiles by level range
    let in_range = profile
      .id
      .strip_prefix('l')
      .and_then(|s| s.parse::<u8>().ok())
      .map(|level| level >= min_level && level <= max_level)
      .unwrap_or(false);
    if !in_range {
      continue;
    }
    let user_profile = user.profiles.get(idx);
    let delta_rating = user_profile.map(|p| p.rating).unwrap_or(0.0);
    let delta_games = user_profile.map(|p| p.games).unwrap_or(0);
    entries.push(MixedEntry {
      side: MixedSide::Heuristic(idx),
      rating: profile.rating + delta_rating,
      games: profile.games + delta_games,
      config: profile.config,
      llm: None,
    });
  }

  let llm_id_set: std::collections::HashSet<String> = llm_ids.iter().cloned().collect();
  for profile in user.extras.iter() {
    if profile.kind != ProfileKind::Llm {
      continue;
    }
    if !llm_id_set.contains(&profile.id) {
      continue;
    }
    if !llm_keys.contains_key(&profile.id) {
      return Err(format!("Missing API key for LLM profile {}", profile.name));
    }
    entries.push(MixedEntry {
      side: MixedSide::Llm(profile.id.clone()),
      rating: profile.rating,
      games: profile.games,
      config: None,
      llm: profile.llm.clone(),
    });
  }

  let profile_count = entries.len();
  if profile_count < 2 || games_per_pair == 0 {
    on_progress(0, 0);
    return Ok(SelfPlayReport {
      games_per_pair,
      total_games: 0,
      completed_games: 0,
      stopped: false,
    });
  }

  let mut pairs = build_pairs(profile_count);
  let mut rng = StdRng::seed_from_u64(42);
  pairs.shuffle(&mut rng);
  let total_games = pairs.len() as u32 * games_per_pair;
  on_progress(0, total_games);

  let fallback_map = build_llm_fallbacks(&entries)?;
  let pair_list = std::sync::Arc::new(pairs);
  let entry_list = std::sync::Arc::new(entries);
  let fallback_map = std::sync::Arc::new(fallback_map);
  let key_map = std::sync::Arc::new(llm_keys.clone());
  let games_per_pair_usize = games_per_pair as usize;

  let (tx, rx) = std::sync::mpsc::channel::<Result<MixedJobResult, String>>();
  let index = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
  let total_jobs = total_games as usize;
  let worker_count = usize::max(1, usize::min(parallelism, total_jobs));
  let mut handles = Vec::new();

  for _ in 0..worker_count {
    let tx = tx.clone();
    let pair_list = pair_list.clone();
    let entry_list = entry_list.clone();
    let fallback_map = fallback_map.clone();
    let key_map = key_map.clone();
    let index = index.clone();
    let total_pairs = pair_list.len();
    let stop_flag = stop_flag.clone();
    handles.push(std::thread::spawn(move || loop {
      if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
        break;
      }
      let idx = index.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
      if idx >= total_jobs {
        break;
      }
      let pair_idx = idx / games_per_pair_usize;
      let game_idx = idx % games_per_pair_usize;
      if pair_idx >= total_pairs {
        break;
      }
      let (a, b) = pair_list[pair_idx];
      let (black_idx, white_idx) = if game_idx % 2 == 0 { (a, b) } else { (b, a) };
      let result = play_mixed_game(
        &entry_list,
        black_idx,
        white_idx,
        &key_map,
        &fallback_map,
      );
      let _ = tx.send(result.map(|result| MixedJobResult {
        black_idx,
        white_idx,
        result,
      }));
    }));
  }
  drop(tx);

  let mut completed = 0u32;
  let mut pending_saves = 0u32;
  for msg in rx {
    let result = msg?;
    apply_mixed_result(
      base,
      user,
      entry_list.as_ref(),
      result.black_idx,
      result.white_idx,
      result.result,
    )?;
    completed += 1;
    pending_saves += 1;

    // Batch write: save every BATCH_SAVE_SIZE games instead of every game
    if pending_saves >= BATCH_SAVE_SIZE {
      user.save(save_path)?;
      pending_saves = 0;
    }
    on_progress(completed, total_games);
  }

  // Final save for any remaining games
  if pending_saves > 0 {
    user.save(save_path)?;
  }

  for handle in handles {
    let _ = handle.join();
  }

  let stopped = stop_flag.load(std::sync::atomic::Ordering::Relaxed) && completed < total_games;

  Ok(SelfPlayReport {
    games_per_pair,
    total_games,
    completed_games: completed,
    stopped,
  })
}

fn play_ai_game(black: AiConfig, white: AiConfig) -> Result<GameResult, String> {
  let players = Players {
    black: "AI".to_string(),
    white: "AI".to_string(),
  };
  let mode = GameMode::AiVsAi {
    black_id: "self_play_black".to_string(),
    white_id: "self_play_white".to_string(),
  };
  let mut game = GameState::new(15, RuleSetKind::Standard, players, mode);

  while game.result.is_none() {
    let config = if game.to_move == Player::B { black } else { white };
    let coord = ai::choose_move(&game.board, RuleSetKind::Standard, game.to_move, config);
    let Some(coord) = coord else {
      break;
    };
    game.apply_move(coord.x, coord.y)?;
  }

  Ok(game.result.unwrap_or(GameResult::Draw))
}

fn play_mixed_game(
  entries: &[MixedEntry],
  black_idx: usize,
  white_idx: usize,
  llm_keys: &std::collections::HashMap<String, String>,
  fallback_map: &std::collections::HashMap<String, AiConfig>,
) -> Result<GameResult, String> {
  let players = Players {
    black: "Self-play".to_string(),
    white: "Self-play".to_string(),
  };
  let mode = GameMode::AiVsAi {
    black_id: "self_play_black".to_string(),
    white_id: "self_play_white".to_string(),
  };
  let mut game = GameState::new(15, RuleSetKind::Standard, players, mode);
  let black_entry = entries.get(black_idx).ok_or_else(|| "Invalid black index".to_string())?;
  let white_entry = entries.get(white_idx).ok_or_else(|| "Invalid white index".to_string())?;

  while game.result.is_none() {
    let entry = if game.to_move == Player::B { black_entry } else { white_entry };
    let coord = match &entry.side {
      MixedSide::Heuristic(_) => {
        let config = entry.config.ok_or_else(|| "Missing AI config".to_string())?;
        ai::choose_move(&game.board, RuleSetKind::Standard, game.to_move, config)
      }
      MixedSide::Llm(id) => {
        if let Some(tactical) = ai::tactical_move(&game.board, RuleSetKind::Standard, game.to_move) {
          Some(tactical)
        } else {
          let api_key = llm_keys
            .get(id)
            .ok_or_else(|| "Missing API key for LLM profile".to_string())?;
          let config = entry.llm.clone().ok_or_else(|| "Missing LLM config".to_string())?;
          match llm::choose_move(&game.board, game.to_move, &config, api_key, &game.moves) {
            Ok(coord) => Some(coord),
            Err(_) => fallback_map
              .get(id)
              .and_then(|fallback| ai::choose_move(&game.board, RuleSetKind::Standard, game.to_move, *fallback)),
          }
        }
      }
    };

    let Some(coord) = coord else {
      break;
    };
    if let Err(_) = game.apply_move(coord.x, coord.y) {
      break;
    }
  }

  Ok(game.result.unwrap_or(GameResult::Draw))
}

fn expected_score(rating_a: f64, rating_b: f64) -> f64 {
  1.0 / (1.0 + 10f64.powf((rating_b - rating_a) / 400.0))
}

fn apply_rating(rating: f64, score: f64, expected: f64, k: f64) -> f64 {
  rating + k * (score - expected)
}

fn k_factor(games: u32) -> f64 {
  if games < 20 {
    32.0
  } else if games < 80 {
    24.0
  } else {
    16.0
  }
}

fn score_for_result(result: GameResult, player: Player) -> f64 {
  match (result, player) {
    (GameResult::BWin, Player::B) => 1.0,
    (GameResult::WWin, Player::W) => 1.0,
    (GameResult::Draw, _) => 0.5,
    _ => 0.0,
  }
}

fn adjust_for_color(player_rating: f64, opp_rating: f64, player_color: Player) -> (f64, f64) {
  match player_color {
    Player::B => (player_rating + BLACK_ADVANTAGE, opp_rating),
    Player::W => (player_rating, opp_rating + BLACK_ADVANTAGE),
  }
}

fn apply_result_to_entry(entry: &mut RatingEntry, result: GameResult, player: Player) {
  match (result, player) {
    (GameResult::BWin, Player::B) | (GameResult::WWin, Player::W) => entry.wins += 1,
    (GameResult::Draw, _) => entry.draws += 1,
    _ => entry.losses += 1,
  }
}

fn apply_result_to_profile(profile: &mut ProfileRating, result: GameResult, player: Player) {
  match (result, player) {
    (GameResult::BWin, Player::B) | (GameResult::WWin, Player::W) => profile.wins += 1,
    (GameResult::Draw, _) => profile.draws += 1,
    _ => profile.losses += 1,
  }
}

fn apply_score_to_profile(profile: &mut ProfileRating, score: f64) {
  if (score - 1.0).abs() < f64::EPSILON {
    profile.wins += 1;
  } else if (score - 0.5).abs() < f64::EPSILON {
    profile.draws += 1;
  } else {
    profile.losses += 1;
  }
}

fn build_llm_fallbacks(entries: &[MixedEntry]) -> Result<std::collections::HashMap<String, AiConfig>, String> {
  let mut heuristics = Vec::new();
  for entry in entries {
    if let MixedSide::Heuristic(_) = entry.side {
      if let Some(config) = entry.config {
        heuristics.push((entry.rating, config));
      }
    }
  }
  if heuristics.is_empty() {
    return Err("No heuristic profiles available for LLM fallback".to_string());
  }

  let mut map = std::collections::HashMap::new();
  for entry in entries {
    if let MixedSide::Llm(id) = &entry.side {
      let mut best = heuristics[0];
      let mut best_delta = (heuristics[0].0 - entry.rating).abs();
      for &(rating, config) in heuristics.iter().skip(1) {
        let delta = (rating - entry.rating).abs();
        if delta < best_delta {
          best = (rating, config);
          best_delta = delta;
        }
      }
      map.insert(id.clone(), best.1);
    }
  }
  Ok(map)
}

fn apply_mixed_result(
  base: &RatingStore,
  user: &mut RatingStore,
  entries: &[MixedEntry],
  black_idx: usize,
  white_idx: usize,
  result: GameResult,
) -> Result<(), String> {
  let black_entry = entries.get(black_idx).ok_or_else(|| "Invalid black index".to_string())?;
  let white_entry = entries.get(white_idx).ok_or_else(|| "Invalid white index".to_string())?;

  let (rating_black, games_black) = effective_for_side(base, user, black_entry)?;
  let (rating_white, games_white) = effective_for_side(base, user, white_entry)?;
  let (adj_black, adj_white) = adjust_for_color(rating_black, rating_white, Player::B);
  let expected_black = expected_score(adj_black, adj_white);
  let expected_white = 1.0 - expected_black;
  let score_black = score_for_result(result, Player::B);

  let new_black = apply_rating(rating_black, score_black, expected_black, k_factor(games_black));
  let new_white = apply_rating(rating_white, 1.0 - score_black, expected_white, k_factor(games_white));

  match (&black_entry.side, &white_entry.side) {
    (MixedSide::Heuristic(idx_a), MixedSide::Heuristic(idx_b)) => {
      update_user_profiles_with_base(base, user, *idx_a, *idx_b, new_black, new_white, result)?;
    }
    (MixedSide::Heuristic(idx), MixedSide::Llm(id)) => {
      update_user_profile_with_base(base, user, *idx, new_black, result, Player::B)?;
      update_llm_profile(user, id, new_white, result, Player::W)?;
    }
    (MixedSide::Llm(id), MixedSide::Heuristic(idx)) => {
      update_llm_profile(user, id, new_black, result, Player::B)?;
      update_user_profile_with_base(base, user, *idx, new_white, result, Player::W)?;
    }
    (MixedSide::Llm(id_a), MixedSide::Llm(id_b)) => {
      update_llm_profile(user, id_a, new_black, result, Player::B)?;
      update_llm_profile(user, id_b, new_white, result, Player::W)?;
    }
  }

  Ok(())
}

fn effective_for_side(
  base: &RatingStore,
  user: &RatingStore,
  entry: &MixedEntry,
) -> Result<(f64, u32), String> {
  match &entry.side {
    MixedSide::Heuristic(idx) => {
      let base_profile = base.profiles.get(*idx).ok_or_else(|| "Base profile missing".to_string())?;
      let user_profile = user.profiles.get(*idx).ok_or_else(|| "User profile missing".to_string())?;
      Ok((
        base_profile.rating + user_profile.rating,
        base_profile.games + user_profile.games,
      ))
    }
    MixedSide::Llm(id) => {
      let profile = user
        .extras
        .iter()
        .find(|p| p.id == *id)
        .ok_or_else(|| "LLM profile missing".to_string())?;
      Ok((profile.rating, profile.games))
    }
  }
}

fn update_user_profiles_with_base(
  base: &RatingStore,
  user: &mut RatingStore,
  idx_a: usize,
  idx_b: usize,
  new_a: f64,
  new_b: f64,
  result: GameResult,
) -> Result<(), String> {
  if idx_a == idx_b {
    return Err("Profiles must be different".to_string());
  }
  let base_a = base.profiles.get(idx_a).ok_or_else(|| "Base profile missing".to_string())?;
  let base_b = base.profiles.get(idx_b).ok_or_else(|| "Base profile missing".to_string())?;
  if idx_a < idx_b {
    let (left, right) = user.profiles.split_at_mut(idx_b);
    let profile_a = left.get_mut(idx_a).ok_or_else(|| "User profile missing".to_string())?;
    let profile_b = right.get_mut(0).ok_or_else(|| "User profile missing".to_string())?;
    profile_a.rating = new_a - base_a.rating;
    profile_b.rating = new_b - base_b.rating;
    profile_a.games += 1;
    profile_b.games += 1;
    apply_result_to_profile(profile_a, result, Player::B);
    apply_result_to_profile(profile_b, result, Player::W);
  } else {
    let (left, right) = user.profiles.split_at_mut(idx_a);
    let profile_b = left.get_mut(idx_b).ok_or_else(|| "User profile missing".to_string())?;
    let profile_a = right.get_mut(0).ok_or_else(|| "User profile missing".to_string())?;
    profile_a.rating = new_a - base_a.rating;
    profile_b.rating = new_b - base_b.rating;
    profile_a.games += 1;
    profile_b.games += 1;
    apply_result_to_profile(profile_a, result, Player::B);
    apply_result_to_profile(profile_b, result, Player::W);
  }
  Ok(())
}

fn update_user_profile_with_base(
  base: &RatingStore,
  user: &mut RatingStore,
  idx: usize,
  new_rating: f64,
  result: GameResult,
  player: Player,
) -> Result<(), String> {
  let base_profile = base.profiles.get(idx).ok_or_else(|| "Base profile missing".to_string())?;
  let profile = user.profiles.get_mut(idx).ok_or_else(|| "User profile missing".to_string())?;
  profile.rating = new_rating - base_profile.rating;
  profile.games += 1;
  apply_result_to_profile(profile, result, player);
  Ok(())
}

fn update_llm_profile(
  user: &mut RatingStore,
  id: &str,
  new_rating: f64,
  result: GameResult,
  player: Player,
) -> Result<(), String> {
  let profile = user
    .extras
    .iter_mut()
    .find(|p| p.id == id)
    .ok_or_else(|| "LLM profile missing".to_string())?;
  profile.rating = new_rating;
  profile.games += 1;
  apply_result_to_profile(profile, result, player);
  Ok(())
}

#[derive(Clone, Debug)]
struct JobResult {
  black_idx: usize,
  white_idx: usize,
  result: GameResult,
}

fn build_pairs(count: usize) -> Vec<(usize, usize)> {
  let mut pairs = Vec::new();
  for i in 0..count {
    for j in (i + 1)..count {
      pairs.push((i, j));
    }
  }
  pairs
}

fn default_profiles() -> Vec<ProfileRating> {
  vec![
    ProfileRating {
      id: "l01".to_string(),
      name: "Level 01".to_string(),
      rating: 600.0,
      games: 0,
      wins: 0,
      draws: 0,
      losses: 0,
      kind: ProfileKind::Heuristic,
      config: Some(AiConfig {
        depth: 1,
        max_candidates: 6,
        randomness: 5,
        max_nodes: 800,
        defense_weight: 9,
      }),
      llm: None,
    },
    ProfileRating {
      id: "l02".to_string(),
      name: "Level 02".to_string(),
      rating: 700.0,
      games: 0,
      wins: 0,
      draws: 0,
      losses: 0,
      kind: ProfileKind::Heuristic,
      config: Some(AiConfig {
        depth: 2,
        max_candidates: 8,
        randomness: 4,
        max_nodes: 1500,
        defense_weight: 10,
      }),
      llm: None,
    },
    ProfileRating {
      id: "l03".to_string(),
      name: "Level 03".to_string(),
      rating: 800.0,
      games: 0,
      wins: 0,
      draws: 0,
      losses: 0,
      kind: ProfileKind::Heuristic,
      config: Some(AiConfig {
        depth: 2,
        max_candidates: 10,
        randomness: 3,
        max_nodes: 2500,
        defense_weight: 11,
      }),
      llm: None,
    },
    ProfileRating {
      id: "l04".to_string(),
      name: "Level 04".to_string(),
      rating: 900.0,
      games: 0,
      wins: 0,
      draws: 0,
      losses: 0,
      kind: ProfileKind::Heuristic,
      config: Some(AiConfig {
        depth: 3,
        max_candidates: 10,
        randomness: 2,
        max_nodes: 4000,
        defense_weight: 11,
      }),
      llm: None,
    },
    ProfileRating {
      id: "l05".to_string(),
      name: "Level 05".to_string(),
      rating: 1000.0,
      games: 0,
      wins: 0,
      draws: 0,
      losses: 0,
      kind: ProfileKind::Heuristic,
      config: Some(AiConfig {
        depth: 3,
        max_candidates: 12,
        randomness: 2,
        max_nodes: 6500,
        defense_weight: 12,
      }),
      llm: None,
    },
    ProfileRating {
      id: "l06".to_string(),
      name: "Level 06".to_string(),
      rating: 1100.0,
      games: 0,
      wins: 0,
      draws: 0,
      losses: 0,
      kind: ProfileKind::Heuristic,
      config: Some(AiConfig {
        depth: 3,
        max_candidates: 14,
        randomness: 1,
        max_nodes: 9000,
        defense_weight: 12,
      }),
      llm: None,
    },
    ProfileRating {
      id: "l07".to_string(),
      name: "Level 07".to_string(),
      rating: 1200.0,
      games: 0,
      wins: 0,
      draws: 0,
      losses: 0,
      kind: ProfileKind::Heuristic,
      config: Some(AiConfig {
        depth: 4,
        max_candidates: 14,
        randomness: 1,
        max_nodes: 12000,
        defense_weight: 12,
      }),
      llm: None,
    },
    ProfileRating {
      id: "l08".to_string(),
      name: "Level 08".to_string(),
      rating: 1300.0,
      games: 0,
      wins: 0,
      draws: 0,
      losses: 0,
      kind: ProfileKind::Heuristic,
      config: Some(AiConfig {
        depth: 4,
        max_candidates: 16,
        randomness: 1,
        max_nodes: 18000,
        defense_weight: 13,
      }),
      llm: None,
    },
    ProfileRating {
      id: "l09".to_string(),
      name: "Level 09".to_string(),
      rating: 1400.0,
      games: 0,
      wins: 0,
      draws: 0,
      losses: 0,
      kind: ProfileKind::Heuristic,
      config: Some(AiConfig {
        depth: 4,
        max_candidates: 18,
        randomness: 0,
        max_nodes: 26000,
        defense_weight: 13,
      }),
      llm: None,
    },
    ProfileRating {
      id: "l10".to_string(),
      name: "Level 10".to_string(),
      rating: 1500.0,
      games: 0,
      wins: 0,
      draws: 0,
      losses: 0,
      kind: ProfileKind::Heuristic,
      config: Some(AiConfig {
        depth: 5,
        max_candidates: 18,
        randomness: 0,
        max_nodes: 35000,
        defense_weight: 13,
      }),
      llm: None,
    },
    ProfileRating {
      id: "l11".to_string(),
      name: "Level 11".to_string(),
      rating: 1600.0,
      games: 0,
      wins: 0,
      draws: 0,
      losses: 0,
      kind: ProfileKind::Heuristic,
      config: Some(AiConfig {
        depth: 5,
        max_candidates: 20,
        randomness: 0,
        max_nodes: 45000,
        defense_weight: 14,
      }),
      llm: None,
    },
    ProfileRating {
      id: "l12".to_string(),
      name: "Level 12".to_string(),
      rating: 1750.0,
      games: 0,
      wins: 0,
      draws: 0,
      losses: 0,
      kind: ProfileKind::Heuristic,
      config: Some(AiConfig {
        depth: 6,
        max_candidates: 20,
        randomness: 0,
        max_nodes: 60000,
        defense_weight: 14,
      }),
      llm: None,
    },
  ]
}
