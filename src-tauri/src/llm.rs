use std::collections::HashSet;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

use crate::ai;
use crate::engine::Board;
use crate::types::{Coord, LlmConfig, Move, Player};

const COLS: &str = "ABCDEFGHIJKLMNO";
const MAX_RETRIES: u32 = 3;
const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Serialize)]
struct ChatMessage {
  role: String,
  content: String,
}

#[derive(Serialize)]
struct ChatRequest {
  model: String,
  messages: Vec<ChatMessage>,
  temperature: f64,
  top_p: f64,
  max_tokens: u32,
}

#[derive(Deserialize)]
struct ChatChoice {
  message: ChatMessageResponse,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
  content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
  choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ErrorDetail {
  message: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
  error: ErrorDetail,
}

lazy_static::lazy_static! {
  static ref HTTP_CLIENT: Client = Client::builder()
    .timeout(Duration::from_secs(60))
    .build()
    .expect("Failed to create HTTP client");
}

pub fn choose_move(
  board: &Board,
  player: Player,
  config: &LlmConfig,
  api_key: &str,
  moves: &[Move],
) -> Result<Coord, String> {
  // Use tokio runtime for async operation
  let rt = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .map_err(|e| format!("Failed to create async runtime: {e}"))?;

  rt.block_on(choose_move_async(board, player, config, api_key, moves))
}

pub async fn choose_move_async(
  board: &Board,
  player: Player,
  config: &LlmConfig,
  api_key: &str,
  moves: &[Move],
) -> Result<Coord, String> {
  if api_key.trim().is_empty() {
    return Err("Missing API key for LLM profile".to_string());
  }

  let candidates = ai::candidate_moves_for_llm(board, player, config.candidate_limit);
  if candidates.is_empty() {
    return Err("No valid moves".to_string());
  }

  let candidate_list: Vec<String> = candidates.iter().map(|c| coord_to_label(*c)).collect();
  let candidate_set: HashSet<String> = candidate_list.iter().cloned().collect();
  let (system, user) = build_prompt(board, player, moves, &candidate_list);

  let mut last_error = String::new();

  for attempt in 1..=MAX_RETRIES {
    match try_llm_call_async(config, api_key, &system, &user, &candidate_set).await {
      Ok(coord) => return Ok(coord),
      Err(e) => {
        last_error = e;
        if attempt < MAX_RETRIES {
          tokio::time::sleep(Duration::from_millis(500)).await;
        }
      }
    }
  }

  Err(format!("LLM failed after {} attempts: {}", MAX_RETRIES, last_error))
}

async fn try_llm_call_async(
  config: &LlmConfig,
  api_key: &str,
  system: &str,
  user: &str,
  candidate_set: &HashSet<String>,
) -> Result<Coord, String> {
  let response = call_llm_api(config, api_key, system, user).await?;
  let coord = parse_response(&response)?;
  let coord_label = coord_to_label(coord);
  if !candidate_set.contains(&coord_label) {
    return Err(format!("LLM returned move {} outside candidate list", coord_label));
  }
  Ok(coord)
}

async fn call_llm_api(
  config: &LlmConfig,
  api_key: &str,
  system: &str,
  user: &str,
) -> Result<String, String> {
  let base_url = if config.base_url.trim().is_empty() {
    DEFAULT_BASE_URL.to_string()
  } else {
    // Ensure URL ends with /chat/completions if it's just a base URL
    let url = config.base_url.trim_end_matches('/');
    if url.ends_with("/chat/completions") {
      url.to_string()
    } else if url.ends_with("/v1") {
      format!("{}/chat/completions", url)
    } else {
      format!("{}/v1/chat/completions", url)
    }
  };

  let request_body = ChatRequest {
    model: config.model.clone(),
    messages: vec![
      ChatMessage {
        role: "system".to_string(),
        content: system.to_string(),
      },
      ChatMessage {
        role: "user".to_string(),
        content: user.to_string(),
      },
    ],
    temperature: config.temperature as f64,
    top_p: config.top_p as f64,
    max_tokens: config.max_tokens,
  };

  let request_timeout = Duration::from_millis(config.timeout_ms as u64);

  let response = timeout(
    request_timeout,
    HTTP_CLIENT
      .post(&base_url)
      .header("Authorization", format!("Bearer {}", api_key))
      .header("Content-Type", "application/json")
      .json(&request_body)
      .send(),
  )
  .await
  .map_err(|_| "Request timed out".to_string())?
  .map_err(|e| format!("Request failed: {e}"))?;

  let status = response.status();
  let body = response
    .text()
    .await
    .map_err(|e| format!("Failed to read response: {e}"))?;

  if !status.is_success() {
    // Try to parse error response
    if let Ok(error_resp) = serde_json::from_str::<ErrorResponse>(&body) {
      return Err(format!("API error ({}): {}", status, error_resp.error.message));
    }
    return Err(format!("API error ({}): {}", status, truncate_for_error(&body)));
  }

  let chat_response: ChatResponse = serde_json::from_str(&body)
    .map_err(|e| format!("Failed to parse response: {e}"))?;

  let content = chat_response
    .choices
    .first()
    .map(|c| c.message.content.clone())
    .ok_or_else(|| "Empty response from LLM".to_string())?;

  Ok(content)
}

fn build_prompt(
  board: &Board,
  player: Player,
  moves: &[Move],
  candidates: &[String],
) -> (String, String) {
  let system = "You are a Gomoku player. Board size 15x15.\n\
Use coordinates A–O (columns) and 1–15 (rows).\n\
You must choose a move from the provided candidates list.\n\
Priority: (1) if you can win immediately, choose that move; (2) if the opponent can win immediately, block it; (3) otherwise choose the strongest candidate.\n\
Respond only with JSON: {\"move\":\"H8\"} where move is in candidates.\n\
If no move possible, respond {\"move\":\"pass\"}."
    .to_string();

  let to_move = match player {
    Player::B => "Black",
    Player::W => "White",
  };
  let (black_stones, white_stones) = list_stones(board);
  let history = format_move_history(moves);
  let board_str = render_board(board);
  let user = format!(
    "To move: {to_move}\nBlack stones: {black_stones}\nWhite stones: {white_stones}\nMove history: {history}\nCandidates: {candidates}\nBoard (row 15 at top):\n{board_str}",
    candidates = candidates.join(", ")
  );
  (system, user)
}

fn format_move_history(moves: &[Move]) -> String {
  if moves.is_empty() {
    return "None (opening move)".to_string();
  }
  moves
    .iter()
    .enumerate()
    .map(|(i, mv)| {
      let coord = coord_to_label(Coord { x: mv.x, y: mv.y });
      let player = player_label(mv.player);
      format!("{}. {}({})", i + 1, coord, player)
    })
    .collect::<Vec<_>>()
    .join(", ")
}

fn render_board(board: &Board) -> String {
  let size = board.size();
  let mut out = String::new();
  out.push_str("   ");
  for c in COLS.chars().take(size) {
    out.push(c);
    out.push(' ');
  }
  out.push('\n');

  for row in (0..size).rev() {
    out.push_str(&format!("{:>2} ", row + 1));
    for col in 0..size {
      let ch = match board.get(col, row) {
        None => '.',
        Some(Player::B) => 'B',
        Some(Player::W) => 'W',
      };
      out.push(ch);
      out.push(' ');
    }
    out.push('\n');
  }
  out
}

fn list_stones(board: &Board) -> (String, String) {
  let size = board.size();
  let mut black = Vec::new();
  let mut white = Vec::new();
  for y in 0..size {
    for x in 0..size {
      match board.get(x, y) {
        Some(Player::B) => black.push(coord_to_label(Coord { x, y })),
        Some(Player::W) => white.push(coord_to_label(Coord { x, y })),
        None => {}
      }
    }
  }
  (
    if black.is_empty() { "none".to_string() } else { black.join(", ") },
    if white.is_empty() { "none".to_string() } else { white.join(", ") },
  )
}

fn player_label(player: Player) -> &'static str {
  match player {
    Player::B => "B",
    Player::W => "W",
  }
}

fn coord_to_label(coord: Coord) -> String {
  let col = COLS
    .chars()
    .nth(coord.x)
    .unwrap_or('A');
  format!("{}{}", col, coord.y + 1)
}

fn parse_label(label: &str) -> Option<Coord> {
  if label.len() < 2 {
    return None;
  }
  let mut chars = label.chars();
  let col = chars.next()?.to_ascii_uppercase();
  let col_idx = COLS.find(col)?;
  let row_str: String = chars.collect();
  let row: usize = row_str.parse().ok()?;
  if row == 0 {
    return None;
  }
  Some(Coord {
    x: col_idx,
    y: row - 1,
  })
}

fn parse_response(raw: &str) -> Result<Coord, String> {
  // Try multiple parsing strategies
  if let Some(coord) = try_parse_json(raw) {
    return Ok(coord);
  }
  if let Some(coord) = try_extract_json_from_text(raw) {
    return Ok(coord);
  }
  if let Some(coord) = try_extract_move_directly(raw) {
    return Ok(coord);
  }
  Err(format!("Failed to parse LLM response: {}", truncate_for_error(raw)))
}

fn try_parse_json(raw: &str) -> Option<Coord> {
  let value: serde_json::Value = serde_json::from_str(raw).ok()?;
  extract_move_from_json(&value)
}

fn try_extract_json_from_text(raw: &str) -> Option<Coord> {
  // Remove markdown code blocks: ```json ... ``` or ``` ... ```
  let cleaned = raw
    .trim()
    .strip_prefix("```json")
    .or_else(|| raw.trim().strip_prefix("```"))
    .and_then(|s| s.strip_suffix("```"))
    .map(|s| s.trim())
    .unwrap_or(raw);

  if let Some(coord) = try_parse_json(cleaned) {
    return Some(coord);
  }

  // Try to find JSON object in text: look for { ... }
  if let Some(start) = raw.find('{') {
    if let Some(end) = raw.rfind('}') {
      if end > start {
        let json_str = &raw[start..=end];
        if let Some(coord) = try_parse_json(json_str) {
          return Some(coord);
        }
      }
    }
  }

  None
}

fn try_extract_move_directly(raw: &str) -> Option<Coord> {
  // Try to find a coordinate pattern directly (e.g., "H8", "A15")
  // Pattern: letter A-O followed by 1-15
  let raw_upper = raw.to_uppercase();
  for word in raw_upper.split(|c: char| !c.is_alphanumeric()) {
    let word = word.trim();
    if word.len() >= 2 && word.len() <= 3 {
      if let Some(first_char) = word.chars().next() {
        if ('A'..='O').contains(&first_char) {
          let rest: String = word.chars().skip(1).collect();
          if let Ok(num) = rest.parse::<usize>() {
            if (1..=15).contains(&num) {
              return parse_label(word);
            }
          }
        }
      }
    }
  }
  None
}

fn extract_move_from_json(value: &serde_json::Value) -> Option<Coord> {
  let move_str = value.get("move").and_then(|v| v.as_str())?;
  if move_str.eq_ignore_ascii_case("pass") {
    return None;
  }
  parse_label(move_str)
}

fn truncate_for_error(s: &str) -> String {
  if s.len() > 100 {
    format!("{}...", &s[..100])
  } else {
    s.to_string()
  }
}
