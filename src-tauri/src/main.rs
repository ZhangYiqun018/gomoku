use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{State, Window};
use serde::Serialize;

mod ai;
mod engine;
mod llm;
mod rating;
mod rules;
mod types;
mod users;

use engine::GameState;
use rating::{ratings_base_path, run_self_play, run_self_play_mixed, ProfileRating, RatingStore, RatingsSnapshot, SelfPlayReport};
use types::{GameMode, GameRecord, GameSnapshot, LlmConfig, Player, ProfileKind, RuleSetKind};
use users::{
  ensure_data_dirs, ensure_user_dir, llm_keys_path, new_user_id, now_timestamp, ratings_user_path,
  snapshot_from_store, user_dir, user_settings_path, users_path, LlmKeyStore, UserProfile,
  UserSettings, UserStore, UsersSnapshot,
};

struct AppState {
  game: Mutex<GameState>,
  rating_base: Arc<Mutex<RatingStore>>,
  rating_user: Arc<Mutex<RatingStore>>,
  users: Mutex<UserStore>,
  active_profile: Mutex<String>,
  current_profile: Mutex<String>,
  auto_match: Mutex<bool>,
  match_offset: Mutex<i32>,
  rating_applied: Mutex<bool>,
  self_play_running: Arc<Mutex<bool>>,
  self_play_stop: Arc<AtomicBool>,
}

#[tauri::command]
fn new_game(
  state: State<'_, AppState>,
  rule_set: RuleSetKind,
  mode: Option<GameMode>,
) -> Result<GameSnapshot, String> {
  let mut game = state.game.lock().map_err(|_| "Game state lock poisoned".to_string())?;
  let mode = mode.unwrap_or_default();

  let (players, active_profile_id) = match &mode {
    GameMode::HumanVsAi { human_color } => {
      let active_profile = resolve_active_profile(&state)?;
      let profile_name = profile_name_for(&state, &active_profile)?;
      let profile_label = profile_label_for(&state, &active_profile)?;
      let ai_name = format!("{} ({})", profile_label, profile_name);
      let players = if *human_color == Player::B {
        types::Players {
          black: "Human".to_string(),
          white: ai_name,
        }
      } else {
        types::Players {
          black: ai_name,
          white: "Human".to_string(),
        }
      };
      (players, Some(active_profile))
    }
    GameMode::AiVsAi { black_id, white_id } => {
      let black_name = profile_name_for(&state, black_id)?;
      let black_label = profile_label_for(&state, black_id)?;
      let white_name = profile_name_for(&state, white_id)?;
      let white_label = profile_label_for(&state, white_id)?;
      let players = types::Players {
        black: format!("{} ({})", black_label, black_name),
        white: format!("{} ({})", white_label, white_name),
      };
      (players, None)
    }
    GameMode::HumanVsHuman => {
      let players = types::Players {
        black: "Human (Black)".to_string(),
        white: "Human (White)".to_string(),
      };
      (players, None)
    }
  };

  if let Some(ref profile_id) = active_profile_id {
    let mut current = state
      .current_profile
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    *current = profile_id.clone();
  }

  *game = GameState::new(15, rule_set, players, mode);
  let mut applied = state
    .rating_applied
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  *applied = false;
  Ok(game.snapshot())
}

#[tauri::command]
fn get_state(state: State<'_, AppState>) -> Result<GameSnapshot, String> {
  let game = state.game.lock().map_err(|_| "Game state lock poisoned".to_string())?;
  Ok(game.snapshot())
}

#[tauri::command]
fn make_move(state: State<'_, AppState>, x: usize, y: usize) -> Result<GameSnapshot, String> {
  let mut game = state.game.lock().map_err(|_| "Game state lock poisoned".to_string())?;

  if !game.can_human_move() {
    return Err("It's not your turn".to_string());
  }

  let human_color = match &game.mode {
    GameMode::HumanVsAi { human_color } => *human_color,
    GameMode::HumanVsHuman => game.to_move,
    GameMode::AiVsAi { .. } => return Err("Cannot make human move in AI vs AI mode".to_string()),
  };

  game.apply_move(x, y)?;
  maybe_apply_rating(&state, &game, human_color)?;
  Ok(game.snapshot())
}

#[tauri::command]
fn ai_move(state: State<'_, AppState>) -> Result<GameSnapshot, String> {
  let mut game = state.game.lock().map_err(|_| "Game state lock poisoned".to_string())?;
  if game.result.is_some() {
    return Err("Game is already finished".to_string());
  }

  // Determine which profile to use based on game mode
  let profile_id = match &game.mode {
    GameMode::HumanVsAi { .. } => {
      // Use the current_profile for human vs AI
      state
        .current_profile
        .lock()
        .map_err(|_| "Rating lock poisoned".to_string())?
        .clone()
    }
    GameMode::AiVsAi { black_id, white_id } => {
      // Use the appropriate profile based on whose turn it is
      if game.to_move == Player::B {
        black_id.clone()
      } else {
        white_id.clone()
      }
    }
    GameMode::HumanVsHuman => {
      return Err("AI moves are not available in human vs human mode".to_string());
    }
  };

  let base = state
    .rating_base
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  let user = state
    .rating_user
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  let selection = select_profile(&base, &user, &profile_id)?;
  drop(user);
  drop(base);

  let choice = match selection {
    SelectedProfile::Heuristic { config } => {
      ai::choose_move(&game.board, game.rule_set, game.to_move, config)
        .ok_or_else(|| "No valid moves".to_string())?
    }
    SelectedProfile::Llm { id, config } => {
      if let Some(tactical) = ai::tactical_move(&game.board, game.rule_set, game.to_move) {
        tactical
      } else {
        let user_id = active_user_id(&state)?;
        let keys = load_llm_keys(&user_id)?;
        let api_key = keys
          .keys
          .get(&id)
          .ok_or_else(|| "Missing API key for LLM profile".to_string())?
          .clone();
        llm::choose_move(&game.board, game.to_move, &config, &api_key, &game.moves)?
      }
    }
  };

  // Determine player color for rating purposes
  let human_color = match &game.mode {
    GameMode::HumanVsAi { human_color } => *human_color,
    _ => Player::B, // Default, won't actually be used for rating in AI vs AI mode
  };

  game.apply_move(choice.x, choice.y)?;

  // Only apply rating changes for human vs AI mode
  if matches!(game.mode, GameMode::HumanVsAi { .. }) {
    maybe_apply_rating(&state, &game, human_color)?;
  }

  Ok(game.snapshot())
}

#[tauri::command]
fn save_game(state: State<'_, AppState>, path: String) -> Result<(), String> {
  let game = state.game.lock().map_err(|_| "Game state lock poisoned".to_string())?;
  let record = game.to_record();
  let json = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?;
  std::fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_game(state: State<'_, AppState>, path: String) -> Result<GameSnapshot, String> {
  let data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
  let record: GameRecord = serde_json::from_str(&data).map_err(|e| e.to_string())?;
  let game = GameState::from_record(record)?;
  let mut guard = state.game.lock().map_err(|_| "Game state lock poisoned".to_string())?;
  *guard = game;
  if guard.moves.is_empty() || guard.result.is_some() {
    let active_profile = resolve_active_profile(&state)?;
    let mut current = state
      .current_profile
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    *current = active_profile;
  }
  let mut applied = state
    .rating_applied
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  *applied = guard.result.is_some();
  Ok(guard.snapshot())
}

#[tauri::command]
fn export_training(state: State<'_, AppState>, path: String) -> Result<(), String> {
  let game = state.game.lock().map_err(|_| "Game state lock poisoned".to_string())?;
  let samples = game.training_samples();
  let json = serde_json::to_string_pretty(&samples).map_err(|e| e.to_string())?;
  std::fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_ratings(state: State<'_, AppState>) -> Result<RatingsSnapshot, String> {
  let user_id = active_user_id(&state)?;
  let (player, profiles) = {
    let base = state
      .rating_base
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    let user = state
      .rating_user
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    (user.player.clone(), effective_profiles(&base, &user, &user_id)?)
  };
  let active_profile = state
    .active_profile
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?
    .clone();
  let auto_match = *state
    .auto_match
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  let match_offset = *state
    .match_offset
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;

  Ok(RatingsSnapshot {
    player,
    profiles,
    active_profile,
    auto_match,
    match_offset,
  })
}

#[tauri::command]
fn get_users(state: State<'_, AppState>) -> Result<UsersSnapshot, String> {
  let store = state
    .users
    .lock()
    .map_err(|_| "User store lock poisoned".to_string())?;
  Ok(snapshot_from_store(&store))
}

#[tauri::command]
fn create_user(state: State<'_, AppState>, name: String) -> Result<UsersSnapshot, String> {
  let trimmed = name.trim();
  if trimmed.is_empty() {
    return Err("User name cannot be empty".to_string());
  }

  {
    let game = state.game.lock().map_err(|_| "Game state lock poisoned".to_string())?;
    if !game.moves.is_empty() && game.result.is_none() {
      return Err("Finish the current game before switching users".to_string());
    }
  }

  let id = new_user_id();
  ensure_user_dir(&id)?;

  let base = state
    .rating_base
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  let user_store = RatingStore::load_or_default_user(&ratings_user_path(&id));
  let settings = default_user_settings(&base, &user_store, &id);
  user_store.save(&ratings_user_path(&id))?;
  settings.save(&user_settings_path(&id))?;

  let user_profile = UserProfile {
    id: id.clone(),
    name: trimmed.to_string(),
    created_at: now_timestamp(),
  };

  let snapshot = {
    let mut store = state
      .users
      .lock()
      .map_err(|_| "User store lock poisoned".to_string())?;
    store.users.push(user_profile);
    store.active_user = id.clone();
    store.save(&users_path())?;
    snapshot_from_store(&store)
  };

  apply_user_context(&state, &id, user_store, settings)?;
  Ok(snapshot)
}

#[tauri::command]
fn set_active_user(state: State<'_, AppState>, id: String) -> Result<UsersSnapshot, String> {
  {
    let game = state.game.lock().map_err(|_| "Game state lock poisoned".to_string())?;
    if !game.moves.is_empty() && game.result.is_none() {
      return Err("Finish the current game before switching users".to_string());
    }
  }

  let snapshot = {
    let mut store = state
      .users
      .lock()
      .map_err(|_| "User store lock poisoned".to_string())?;
    if !store.users.iter().any(|user| user.id == id) {
      return Err("Unknown user".to_string());
    }
    store.active_user = id.clone();
    store.save(&users_path())?;
    snapshot_from_store(&store)
  };

  let base = state
    .rating_base
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  ensure_user_dir(&id)?;
  let user_ratings_path = ratings_user_path(&id);
  let user_store = RatingStore::load_or_default_user(&user_ratings_path);
  if !user_ratings_path.exists() {
    let _ = user_store.save(&user_ratings_path);
  }
  let settings = load_or_default_settings(&base, &user_store, &id)?;
  apply_user_context(&state, &id, user_store, settings)?;

  Ok(snapshot)
}

#[tauri::command]
fn delete_user(
  state: State<'_, AppState>,
  id: String,
  delete_data: bool,
) -> Result<UsersSnapshot, String> {
  {
    let game = state.game.lock().map_err(|_| "Game state lock poisoned".to_string())?;
    if !game.moves.is_empty() && game.result.is_none() {
      return Err("Finish the current game before switching users".to_string());
    }
  }

  let (snapshot, new_active) = {
    let mut store = state
      .users
      .lock()
      .map_err(|_| "User store lock poisoned".to_string())?;
    if store.users.len() <= 1 {
      return Err("Cannot delete the last user".to_string());
    }
    if !store.users.iter().any(|user| user.id == id) {
      return Err("Unknown user".to_string());
    }
    store.users.retain(|user| user.id != id);
    if store.active_user == id {
      store.active_user = store
        .users
        .first()
        .map(|user| user.id.clone())
        .unwrap_or_default();
    }
    let active = store.active_user.clone();
    store.save(&users_path())?;
    (snapshot_from_store(&store), active)
  };

  if delete_data {
    let _ = fs::remove_dir_all(user_dir(&id));
  }

  if new_active != id {
    let base = state
      .rating_base
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    ensure_user_dir(&new_active)?;
    let user_ratings_path = ratings_user_path(&new_active);
    let user_store = RatingStore::load_or_default_user(&user_ratings_path);
    if !user_ratings_path.exists() {
      let _ = user_store.save(&user_ratings_path);
    }
    let settings = load_or_default_settings(&base, &user_store, &new_active)?;
    apply_user_context(&state, &new_active, user_store, settings)?;
  }

  Ok(snapshot)
}

#[tauri::command]
fn update_user(state: State<'_, AppState>, id: String, name: String) -> Result<UsersSnapshot, String> {
  let trimmed = name.trim();
  if trimmed.is_empty() {
    return Err("User name cannot be empty".to_string());
  }

  let mut store = state
    .users
    .lock()
    .map_err(|_| "User store lock poisoned".to_string())?;
  let user = store
    .users
    .iter_mut()
    .find(|user| user.id == id)
    .ok_or_else(|| "Unknown user".to_string())?;
  user.name = trimmed.to_string();
  store.save(&users_path())?;
  Ok(snapshot_from_store(&store))
}

#[tauri::command]
fn create_llm_profile(
  state: State<'_, AppState>,
  name: String,
  config: LlmConfig,
  api_key: String,
) -> Result<RatingsSnapshot, String> {
  if name.trim().is_empty() {
    return Err("Profile name cannot be empty".to_string());
  }
  {
    let game = state.game.lock().map_err(|_| "Game state lock poisoned".to_string())?;
    if !game.moves.is_empty() && game.result.is_none() {
      return Err("Finish the current game before adding profiles".to_string());
    }
  }

  let user_id = active_user_id(&state)?;
  let mut user = state
    .rating_user
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;

  let id = new_llm_profile_id();
  if user.get_profile_any(&id).is_some() {
    return Err("Profile id collision".to_string());
  }

  let mut llm_config = normalize_llm_config(config)?;
  llm_config.api_key_set = !api_key.trim().is_empty();
  let profile = ProfileRating {
    id: id.clone(),
    name: name.trim().to_string(),
    rating: 1000.0,
    games: 0,
    wins: 0,
    draws: 0,
    losses: 0,
    kind: ProfileKind::Llm,
    config: None,
    llm: Some(llm_config),
  };
  user.extras.push(profile);
  user.save(&ratings_user_path(&user_id))?;
  drop(user);

  if !api_key.trim().is_empty() {
    let mut keys = load_llm_keys(&user_id)?;
    keys.keys.insert(id, api_key);
    save_llm_keys(&user_id, &keys)?;
  }

  get_ratings(state)
}

#[tauri::command]
fn update_llm_profile(
  state: State<'_, AppState>,
  id: String,
  name: String,
  config: LlmConfig,
  api_key: Option<String>,
) -> Result<RatingsSnapshot, String> {
  if name.trim().is_empty() {
    return Err("Profile name cannot be empty".to_string());
  }

  let user_id = active_user_id(&state)?;
  let mut user = state
    .rating_user
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;

  let profile = user
    .extras
    .iter_mut()
    .find(|p| p.id == id)
    .ok_or_else(|| "Unknown profile".to_string())?;
  if profile.kind != ProfileKind::Llm {
    return Err("Profile is not LLM".to_string());
  }

  let mut llm_config = normalize_llm_config(config)?;
  if let Some(ref key) = api_key {
    llm_config.api_key_set = !key.trim().is_empty();
  } else if let Some(existing) = profile.llm.as_ref() {
    llm_config.api_key_set = existing.api_key_set;
  }

  profile.name = name.trim().to_string();
  profile.llm = Some(llm_config);
  user.save(&ratings_user_path(&user_id))?;
  drop(user);

  if let Some(key) = api_key {
    let mut keys = load_llm_keys(&user_id)?;
    if key.trim().is_empty() {
      keys.keys.remove(&id);
    } else {
      keys.keys.insert(id.clone(), key);
    }
    save_llm_keys(&user_id, &keys)?;
  }

  get_ratings(state)
}

#[tauri::command]
fn delete_llm_profile(
  state: State<'_, AppState>,
  id: String,
  delete_key: bool,
) -> Result<RatingsSnapshot, String> {
  let user_id = active_user_id(&state)?;
  let mut user = state
    .rating_user
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;

  let before = user.extras.len();
  user.extras.retain(|p| p.id != id);
  if user.extras.len() == before {
    return Err("Unknown profile".to_string());
  }
  user.save(&ratings_user_path(&user_id))?;
  drop(user);

  if delete_key {
    let mut keys = load_llm_keys(&user_id)?;
    keys.keys.remove(&id);
    save_llm_keys(&user_id, &keys)?;
  }

  let active = state
    .active_profile
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?
    .clone();
  if active == id {
    let base = state
      .rating_base
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    let user = state
      .rating_user
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    let fallback = match_profile_id(&base, &user, &user_id, 0)?
      .unwrap_or_else(|| "l05".to_string());
    {
      let mut active_profile = state
        .active_profile
        .lock()
        .map_err(|_| "Rating lock poisoned".to_string())?;
      *active_profile = fallback.clone();
      let mut current_profile = state
        .current_profile
        .lock()
        .map_err(|_| "Rating lock poisoned".to_string())?;
      *current_profile = fallback.clone();
    }
    save_user_settings(&state, fallback, false, 0)?;
  }

  get_ratings(state)
}

#[tauri::command]
fn set_active_profile(state: State<'_, AppState>, id: String) -> Result<RatingsSnapshot, String> {
  {
    let game = state.game.lock().map_err(|_| "Game state lock poisoned".to_string())?;
    if !game.moves.is_empty() && game.result.is_none() {
      return Err("Cannot change AI profile mid-game".to_string());
    }
  }

  {
    let base = state
      .rating_base
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    let user = state
      .rating_user
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    if !profile_exists(&base, &user, &id) {
      return Err("Unknown profile".to_string());
    }
  }

  {
    let mut active = state
      .active_profile
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    *active = id.clone();
    let mut current = state
      .current_profile
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    *current = active.clone();
  }

  let mut auto = state
    .auto_match
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  *auto = false;
  drop(auto);

  let match_offset = *state
    .match_offset
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  save_user_settings(&state, id.clone(), false, match_offset)?;

  get_ratings(state)
}

#[tauri::command]
fn set_match_mode(
  state: State<'_, AppState>,
  auto_match: bool,
  match_offset: i32,
) -> Result<RatingsSnapshot, String> {
  {
    let mut auto = state
      .auto_match
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    *auto = auto_match;
  }
  {
    let mut offset = state
      .match_offset
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    *offset = match_offset;
  }

  if auto_match {
    let active_profile = resolve_active_profile(&state)?;
    let game = state.game.lock().map_err(|_| "Game state lock poisoned".to_string())?;
    if game.moves.is_empty() || game.result.is_some() {
      let mut current = state
        .current_profile
        .lock()
        .map_err(|_| "Rating lock poisoned".to_string())?;
      *current = active_profile;
    }
  }

  let active_profile = state
    .active_profile
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?
    .clone();
  save_user_settings(&state, active_profile, auto_match, match_offset)?;

  get_ratings(state)
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SelfPlayProgress {
  completed: u32,
  total: u32,
  percent: f32,
}

#[tauri::command]
fn start_self_play(
  state: State<'_, AppState>,
  window: Window,
  games_per_pair: u32,
  parallelism: u32,
  include_llm: bool,
  llm_ids: Vec<String>,
  min_level: Option<u8>,
  max_level: Option<u8>,
) -> Result<bool, String> {
  {
    let mut running = state
      .self_play_running
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    if *running {
      return Err("Self play already running".to_string());
    }
    *running = true;
  }
  state.self_play_stop.store(false, Ordering::Relaxed);

  if include_llm && llm_ids.is_empty() {
    return Err("Select at least one LLM profile".to_string());
  }

  let min_level = min_level.unwrap_or(1);
  let max_level = max_level.unwrap_or(12);
  if min_level < 1 || max_level > 12 || min_level > max_level {
    return Err("Invalid level range".to_string());
  }

  let rating_base = state.rating_base.clone();
  let rating_user = state.rating_user.clone();
  let running_flag = state.self_play_running.clone();
  let stop_flag = state.self_play_stop.clone();
  let progress_window = window.clone();
  let save_path = ratings_base_path();
  let user_id = if include_llm {
    Some(active_user_id(&state)?)
  } else {
    None
  };
  let user_save_path = user_id.as_ref().map(|id| ratings_user_path(id));
  let llm_keys = if let Some(ref id) = user_id {
    Some(load_llm_keys(id)?)
  } else {
    None
  };
  let llm_ids = if include_llm { llm_ids } else { Vec::new() };

  tauri::async_runtime::spawn_blocking(move || {
    let result = (|| -> Result<SelfPlayReport, String> {
      if include_llm {
        let base_store = {
          let rating = rating_base
            .lock()
            .map_err(|_| "Rating lock poisoned".to_string())?;
          rating.clone()
        };
        let mut user_store = {
          let rating = rating_user
            .lock()
            .map_err(|_| "Rating lock poisoned".to_string())?;
          rating.clone()
        };
        let key_store = llm_keys.ok_or_else(|| "Missing LLM keys".to_string())?;
        let save_path = user_save_path.ok_or_else(|| "Missing user path".to_string())?;
        let report = run_self_play_mixed(
          &base_store,
          &mut user_store,
          &key_store.keys,
          games_per_pair,
          usize::max(1, parallelism as usize),
          &llm_ids,
          stop_flag,
          |completed, total| {
            let percent = if total == 0 {
              100.0
            } else {
              (completed as f32 / total as f32) * 100.0
            };
            let _ = progress_window.emit(
              "self_play_progress",
              SelfPlayProgress {
                completed,
                total,
                percent,
              },
            );
          },
          &save_path,
          min_level,
          max_level,
        );
        if let Ok(mut rating) = rating_user.lock() {
          *rating = user_store;
        }
        report
      } else {
        let mut local_store = {
          let rating = rating_base
            .lock()
            .map_err(|_| "Rating lock poisoned".to_string())?;
          rating.clone()
        };
        let report = run_self_play(
          &mut local_store,
          &save_path,
          games_per_pair,
          usize::max(1, parallelism as usize),
          stop_flag,
          |completed, total| {
            let percent = if total == 0 {
              100.0
            } else {
              (completed as f32 / total as f32) * 100.0
            };
            let _ = progress_window.emit(
              "self_play_progress",
              SelfPlayProgress {
                completed,
                total,
                percent,
              },
            );
          },
          min_level,
          max_level,
        );
        if let Ok(mut rating) = rating_base.lock() {
          *rating = local_store;
        }
        report
      }
    })();

    match result {
      Ok(report) => {
        let _ = progress_window.emit("self_play_done", report);
      }
      Err(err) => {
        let _ = progress_window.emit("self_play_error", err);
      }
    }

    if let Ok(mut running) = running_flag.lock() {
      *running = false;
    }
  });

  Ok(true)
}

#[tauri::command]
fn stop_self_play(state: State<'_, AppState>) -> Result<(), String> {
  let running = state
    .self_play_running
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  if *running {
    state.self_play_stop.store(true, Ordering::Relaxed);
  }
  Ok(())
}

fn maybe_apply_rating(
  state: &State<'_, AppState>,
  game: &GameState,
  player_color: Player,
) -> Result<(), String> {
  if game.result.is_none() {
    return Ok(());
  }

  {
    let applied = state
      .rating_applied
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    if *applied {
      return Ok(());
    }
  }

  let profile_id = state
    .current_profile
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?
    .clone();
  let result = game.result.ok_or_else(|| "No result".to_string())?;
  let base = state
    .rating_base
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  let mut user = state
    .rating_user
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  let user_id = active_user_id(state)?;
  if base.get_profile(&profile_id).is_some() {
    user.update_player_vs_profile_user(&base, &profile_id, result, player_color)?;
  } else {
    user.update_player_vs_llm(&profile_id, result, player_color)?;
  }
  user.save(&ratings_user_path(&user_id))?;
  {
    let mut applied = state
      .rating_applied
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    *applied = true;
  }
  Ok(())
}

fn resolve_active_profile(state: &State<'_, AppState>) -> Result<String, String> {
  let auto_match = *state
    .auto_match
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  if !auto_match {
    return Ok(state
      .active_profile
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?
      .clone());
  }

  let offset = *state
    .match_offset
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  let matched = {
    let user_id = active_user_id(state)?;
    let base = state
      .rating_base
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    let user = state
      .rating_user
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    match_profile_id(&base, &user, &user_id, offset)?
  };
  let mut active = state
    .active_profile
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  if let Some(id) = matched {
    *active = id.clone();
    return Ok(id);
  }
  Ok(active.clone())
}

fn profile_name_for(state: &State<'_, AppState>, id: &str) -> Result<String, String> {
  let rating = state
    .rating_base
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  if let Some(profile) = rating.get_profile(id) {
    return Ok(profile.name.clone());
  }
  drop(rating);
  let user = state
    .rating_user
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  user
    .extras
    .iter()
    .find(|p| p.id == id)
    .map(|p| p.name.clone())
    .ok_or_else(|| "Unknown profile".to_string())
}

fn profile_label_for(state: &State<'_, AppState>, id: &str) -> Result<String, String> {
  let rating = state
    .rating_base
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  if rating.get_profile(id).is_some() {
    return Ok("AI".to_string());
  }
  drop(rating);
  let user = state
    .rating_user
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  if user.extras.iter().any(|p| p.id == id && p.kind == ProfileKind::Llm) {
    return Ok("LLM".to_string());
  }
  Ok("AI".to_string())
}

fn profile_exists(base: &RatingStore, user: &RatingStore, id: &str) -> bool {
  base.get_profile(id).is_some() || user.extras.iter().any(|p| p.id == id)
}

enum SelectedProfile {
  Heuristic { config: types::AiConfig },
  Llm { id: String, config: LlmConfig },
}

fn select_profile(
  base: &RatingStore,
  user: &RatingStore,
  id: &str,
) -> Result<SelectedProfile, String> {
  if let Some(profile) = base.get_profile(id) {
    let config = profile
      .config
      .ok_or_else(|| "Missing heuristic config".to_string())?;
    return Ok(SelectedProfile::Heuristic { config });
  }
  if let Some(profile) = user.extras.iter().find(|p| p.id == id) {
    if profile.kind != ProfileKind::Llm {
      return Err("Unsupported profile kind".to_string());
    }
    let config = profile
      .llm
      .clone()
      .ok_or_else(|| "Missing LLM config".to_string())?;
    return Ok(SelectedProfile::Llm {
      id: id.to_string(),
      config,
    });
  }
  Err("Unknown profile".to_string())
}

fn effective_profiles(
  base: &RatingStore,
  user: &RatingStore,
  user_id: &str,
) -> Result<Vec<ProfileRating>, String> {
  let mut combined: Vec<ProfileRating> = base
    .profiles
    .iter()
    .map(|profile| {
      let user_profile = user.profiles.iter().find(|p| p.id == profile.id);
      let (delta_rating, delta_games, delta_wins, delta_draws, delta_losses) = user_profile
        .map(|p| (p.rating, p.games, p.wins, p.draws, p.losses))
        .unwrap_or((0.0, 0, 0, 0, 0));
      ProfileRating {
        id: profile.id.clone(),
        name: profile.name.clone(),
        rating: profile.rating + delta_rating,
        games: profile.games + delta_games,
        wins: profile.wins + delta_wins,
        draws: profile.draws + delta_draws,
        losses: profile.losses + delta_losses,
        kind: ProfileKind::Heuristic,
        config: profile.config,
        llm: None,
      }
    })
    .collect();

  let key_store = load_llm_keys(user_id)?;
  for extra in user.extras.iter() {
    let mut profile = extra.clone();
    if profile.kind == ProfileKind::Llm {
      if let Some(mut llm_config) = profile.llm.clone() {
        llm_config.api_key_set = key_store.keys.contains_key(&profile.id);
        profile.llm = Some(llm_config);
      }
    }
    combined.push(profile);
  }

  Ok(combined)
}

fn match_profile_id(
  base: &RatingStore,
  user: &RatingStore,
  user_id: &str,
  offset: i32,
) -> Result<Option<String>, String> {
  let profiles = effective_profiles(base, user, user_id)?;
  let matcher = RatingStore {
    version: base.version,
    player: user.player.clone(),
    profiles,
    extras: Vec::new(),
  };
  Ok(matcher.match_profile_id(offset))
}

fn active_user_id(state: &State<'_, AppState>) -> Result<String, String> {
  let store = state
    .users
    .lock()
    .map_err(|_| "User store lock poisoned".to_string())?;
  if store.active_user.is_empty() {
    return Err("No active user".to_string());
  }
  Ok(store.active_user.clone())
}

fn load_llm_keys(user_id: &str) -> Result<LlmKeyStore, String> {
  ensure_user_dir(user_id)?;
  let path = llm_keys_path(user_id);
  Ok(LlmKeyStore::load_or_default(&path))
}

fn save_llm_keys(user_id: &str, store: &LlmKeyStore) -> Result<(), String> {
  ensure_user_dir(user_id)?;
  store.save(&llm_keys_path(user_id))
}

fn new_llm_profile_id() -> String {
  let rand_part: u32 = rand::random();
  format!("llm-{}-{:08x}", now_timestamp(), rand_part)
}

fn normalize_llm_config(mut config: LlmConfig) -> Result<LlmConfig, String> {
  if config.model.trim().is_empty() {
    return Err("Model name cannot be empty".to_string());
  }
  if config.max_tokens == 0 {
    config.max_tokens = 128;
  }
  if config.timeout_ms < 5000 {
    config.timeout_ms = 5000;
  }
  if config.candidate_limit == 0 {
    config.candidate_limit = 12;
  }
  Ok(config)
}

fn default_user_settings(base: &RatingStore, user: &RatingStore, user_id: &str) -> UserSettings {
  let active_profile = match_profile_id(base, user, user_id, 0)
    .ok()
    .flatten()
    .unwrap_or_else(|| "l05".to_string());
  UserSettings {
    version: 1,
    active_profile,
    auto_match: true,
    match_offset: 0,
  }
}

fn load_or_default_settings(
  base: &RatingStore,
  user: &RatingStore,
  user_id: &str,
) -> Result<UserSettings, String> {
  ensure_user_dir(user_id)?;
  let path = user_settings_path(user_id);
  if let Ok(data) = fs::read_to_string(&path) {
    if let Ok(settings) = serde_json::from_str::<UserSettings>(&data) {
      if base.get_profile(&settings.active_profile).is_some() {
        return Ok(settings);
      }
    }
  }
  let settings = default_user_settings(base, user, user_id);
  settings.save(&path)?;
  Ok(settings)
}

fn save_user_settings(
  state: &State<'_, AppState>,
  active_profile: String,
  auto_match: bool,
  match_offset: i32,
) -> Result<(), String> {
  let user_id = active_user_id(state)?;
  ensure_user_dir(&user_id)?;
  let settings = UserSettings {
    version: 1,
    active_profile,
    auto_match,
    match_offset,
  };
  settings.save(&user_settings_path(&user_id))
}

fn apply_user_context(
  state: &State<'_, AppState>,
  user_id: &str,
  user_store: RatingStore,
  settings: UserSettings,
) -> Result<(), String> {
  ensure_user_dir(user_id)?;
  {
    let mut user = state
      .rating_user
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    *user = user_store;
  }
  {
    let mut active = state
      .active_profile
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    *active = settings.active_profile.clone();
    let mut current = state
      .current_profile
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    *current = settings.active_profile.clone();
  }
  {
    let mut auto = state
      .auto_match
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    *auto = settings.auto_match;
  }
  {
    let mut offset = state
      .match_offset
      .lock()
      .map_err(|_| "Rating lock poisoned".to_string())?;
    *offset = settings.match_offset;
  }
  let game = state
    .game
    .lock()
    .map_err(|_| "Game state lock poisoned".to_string())?;
  let mut applied = state
    .rating_applied
    .lock()
    .map_err(|_| "Rating lock poisoned".to_string())?;
  *applied = game.result.is_some();
  Ok(())
}

fn main() {
  let players = types::Players {
    black: "Human".to_string(),
    white: "AI".to_string(),
  };
  let game = GameState::new(15, RuleSetKind::Standard, players, GameMode::default());
  let _ = ensure_data_dirs();

  let users_path = users_path();
  let mut users = UserStore::load_or_default(&users_path);
  if users.users.is_empty() {
    let id = new_user_id();
    let _ = ensure_user_dir(&id);
    users.users.push(UserProfile {
      id: id.clone(),
      name: "Player 1".to_string(),
      created_at: now_timestamp(),
    });
    users.active_user = id;
    let _ = users.save(&users_path);
  }
  if users.active_user.is_empty()
    || !users.users.iter().any(|user| user.id == users.active_user)
  {
    users.active_user = users
      .users
      .first()
      .map(|user| user.id.clone())
      .unwrap_or_default();
    let _ = users.save(&users_path);
  }
  let active_user = users.active_user.clone();
  let _ = ensure_user_dir(&active_user);

  let ratings_base_path = ratings_base_path();
  let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let legacy_base_path = manifest_dir
    .parent()
    .unwrap_or(&manifest_dir)
    .join("ratings.json");
  let legacy_user_path = manifest_dir
    .parent()
    .unwrap_or(&manifest_dir)
    .join("ratings_user.json");

  let base_store = if ratings_base_path.exists() {
    RatingStore::load_or_default(&ratings_base_path)
  } else if legacy_base_path.exists() {
    RatingStore::load_or_default(&legacy_base_path)
  } else {
    RatingStore::default()
  };
  if !ratings_base_path.exists() {
    let _ = base_store.save(&ratings_base_path);
  }

  let user_ratings_path = ratings_user_path(&active_user);
  let mut user_store = if user_ratings_path.exists() {
    RatingStore::load_or_default_user(&user_ratings_path)
  } else if legacy_user_path.exists() {
    let store = RatingStore::load_or_default_user(&legacy_user_path);
    let _ = store.save(&user_ratings_path);
    store
  } else {
    RatingStore::load_or_default_user(&user_ratings_path)
  };
  if !user_ratings_path.exists() && legacy_base_path.exists() {
    user_store.player = base_store.player.clone();
  }
  if !user_ratings_path.exists() {
    let _ = user_store.save(&user_ratings_path);
  }

  let user_settings = match load_or_default_settings(&base_store, &user_store, &active_user) {
    Ok(settings) => settings,
    Err(_) => {
      let settings = default_user_settings(&base_store, &user_store, &active_user);
      let _ = settings.save(&user_settings_path(&active_user));
      settings
    }
  };

  tauri::Builder::default()
    .manage(AppState {
      game: Mutex::new(game),
      rating_base: Arc::new(Mutex::new(base_store)),
      rating_user: Arc::new(Mutex::new(user_store)),
      users: Mutex::new(users),
      active_profile: Mutex::new(user_settings.active_profile.clone()),
      current_profile: Mutex::new(user_settings.active_profile),
      auto_match: Mutex::new(user_settings.auto_match),
      match_offset: Mutex::new(user_settings.match_offset),
      rating_applied: Mutex::new(false),
      self_play_running: Arc::new(Mutex::new(false)),
      self_play_stop: Arc::new(AtomicBool::new(false)),
    })
    .invoke_handler(tauri::generate_handler![
      new_game,
      get_state,
      make_move,
      ai_move,
      save_game,
      load_game,
      export_training,
      get_ratings,
      get_users,
      set_active_profile,
      set_match_mode,
      create_llm_profile,
      update_llm_profile,
      delete_llm_profile,
      create_user,
      update_user,
      set_active_user,
      delete_user,
      start_self_play,
      stop_self_play,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
