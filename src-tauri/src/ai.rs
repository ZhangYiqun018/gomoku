use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use rand::seq::SliceRandom;
use rayon::prelude::*;

use crate::engine::Board;
use crate::rules::{rules_for, RuleSet};
use crate::types::{AiConfig, Coord, Move, Player, RuleSetKind};

const WIN_SCORE: i32 = 1_000_000;
const SCORE_FIVE: i32 = 200_000;
const SCORE_OPEN_FOUR: i32 = 50_000;
const SCORE_SEMI_FOUR: i32 = 10_000;
const SCORE_BROKEN_FOUR: i32 = 7_000;
const SCORE_OPEN_THREE: i32 = 3_000;
const SCORE_BROKEN_THREE: i32 = 1_200;
const SCORE_OPEN_TWO: i32 = 300;
const SCORE_SEMI_THREE: i32 = 400;
const SCORE_SEMI_TWO: i32 = 80;
const SCORE_OPEN_ONE: i32 = 6;

const MAX_KILLER_DEPTH: usize = 16; // Maximum depth for killer move tracking
const KILLERS_PER_DEPTH: usize = 2; // Number of killer moves to store per depth

#[derive(Default)]
struct ScoreBreakdown {
  score: i32,
  open_threes: i32,
  open_fours: i32,
}

struct SearchContext {
  nodes: u32,
  candidate_set: HashSet<(usize, usize)>,
  killer_moves: [[Option<Coord>; KILLERS_PER_DEPTH]; MAX_KILLER_DEPTH], // Killer moves per depth
  history: [[u32; 15]; 15], // History heuristic: counts of beta cutoffs per position
}

pub fn candidate_moves_for_llm(board: &Board, player: Player, max_candidates: usize) -> Vec<Coord> {
  let mut work_board = board.clone();
  let mut ctx = SearchContext {
    nodes: 0,
    candidate_set: HashSet::new(),
    killer_moves: [[None; KILLERS_PER_DEPTH]; MAX_KILLER_DEPTH],
    history: [[0; 15]; 15],
  };
  candidate_moves(&mut work_board, player, max_candidates, &mut ctx, 0)
}

pub fn tactical_move(board: &Board, rule_set: RuleSetKind, player: Player) -> Option<Coord> {
  let rules = rules_for(rule_set);
  let mut work_board = board.clone();
  let mut ctx = SearchContext {
    nodes: 0,
    candidate_set: HashSet::new(),
    killer_moves: [[None; KILLERS_PER_DEPTH]; MAX_KILLER_DEPTH],
    history: [[0; 15]; 15],
  };
  let candidates = candidate_moves(&mut work_board, player, usize::MAX, &mut ctx, 0);
  if candidates.is_empty() {
    return None;
  }

  let winning = immediate_wins(&mut work_board, player, &candidates, rules.as_ref());
  if !winning.is_empty() {
    return Some(winning[0]);
  }

  let blocks = immediate_wins(&mut work_board, player.other(), &candidates, rules.as_ref());
  if !blocks.is_empty() {
    return Some(blocks[0]);
  }

  None
}

// Shared context for parallel search with atomic node counter
struct SharedSearchContext {
  nodes: AtomicU32,
  max_nodes: u32,
}

// Transposition table entry flag
#[derive(Clone, Copy, PartialEq)]
enum TTFlag {
  Exact,      // Exact score
  LowerBound, // Alpha cutoff (score >= beta)
  UpperBound, // Beta cutoff (score <= alpha)
}

// Transposition table entry
#[derive(Clone, Copy)]
struct TTEntry {
  hash: u64,
  depth: u8,
  score: i32,
  flag: TTFlag,
}

// Fixed-size transposition table with replacement
struct TranspositionTable {
  entries: Vec<Option<TTEntry>>,
  size: usize,
}

impl TranspositionTable {
  fn new(size: usize) -> Self {
    Self {
      entries: vec![None; size],
      size,
    }
  }

  fn probe(&self, hash: u64, depth: u8) -> Option<(i32, TTFlag)> {
    let index = (hash as usize) % self.size;
    if let Some(entry) = &self.entries[index] {
      if entry.hash == hash && entry.depth >= depth {
        return Some((entry.score, entry.flag));
      }
    }
    None
  }

  fn store(&mut self, hash: u64, depth: u8, score: i32, flag: TTFlag) {
    let index = (hash as usize) % self.size;
    // Replace if slot is empty or new entry has greater/equal depth
    let should_replace = match &self.entries[index] {
      None => true,
      Some(existing) => depth >= existing.depth,
    };
    if should_replace {
      self.entries[index] = Some(TTEntry {
        hash,
        depth,
        score,
        flag,
      });
    }
  }
}

pub fn choose_move(
  board: &Board,
  rule_set: RuleSetKind,
  player: Player,
  config: AiConfig,
) -> Option<Coord> {
  let rules = rules_for(rule_set);
  // 只克隆一次，整个函数复用
  let mut work_board = board.clone();
  // 创建搜索上下文，复用HashSet
  let mut ctx = SearchContext {
    nodes: 0,
    candidate_set: HashSet::new(),
    killer_moves: [[None; KILLERS_PER_DEPTH]; MAX_KILLER_DEPTH],
    history: [[0; 15]; 15],
  };

  let mut candidates = candidate_moves(&mut work_board, player, config.max_candidates, &mut ctx, 0);
  if candidates.is_empty() {
    return None;
  }

  let winning = immediate_wins(&mut work_board, player, &candidates, rules.as_ref());
  if !winning.is_empty() {
    return pick_best(&mut work_board, player, &winning, config);
  }

  let blocks = immediate_wins(&mut work_board, player.other(), &candidates, rules.as_ref());
  if !blocks.is_empty() {
    candidates = blocks;
  }

  // Use parallel evaluation for candidates with iterative deepening
  let shared_ctx = Arc::new(SharedSearchContext {
    nodes: AtomicU32::new(0),
    max_nodes: config.max_nodes.max(1),
  });

  // Transposition table size: ~64K entries, should be enough for typical searches
  const TT_SIZE: usize = 65536;

  let mut best_move: Option<Coord> = None;
  let mut best_score = -WIN_SCORE;

  // Aspiration window constants
  const ASPIRATION_WINDOW: i32 = 50;
  let mut aspiration_guess: Option<i32> = None;

  // Iterative deepening: search from depth 1 to config.depth
  // This fills the transposition table progressively and allows early termination
  let start_depth = if config.depth <= 2 { config.depth } else { 1 };

  for current_depth in start_depth..=config.depth {
    // Check if we've exceeded node budget before starting new iteration
    if shared_ctx.nodes.load(Ordering::Relaxed) >= shared_ctx.max_nodes {
      break;
    }

    // Order candidates: put best move from previous iteration first
    if let Some(prev_best) = best_move {
      if let Some(pos) = candidates.iter().position(|&c| c == prev_best) {
        candidates.remove(pos);
        candidates.insert(0, prev_best);
      }
    }

    // Aspiration window: use previous score as guess for narrower search
    let (mut alpha, mut beta) = match aspiration_guess {
      Some(guess) if current_depth > 1 => {
        (guess - ASPIRATION_WINDOW, guess + ASPIRATION_WINDOW)
      }
      _ => (-WIN_SCORE, WIN_SCORE),
    };

    let mut iteration_complete = false;

    while !iteration_complete {
      let scored: Vec<(i32, Coord)> = candidates
        .par_iter()
        .map(|&coord| {
          // Each thread gets its own rules, board clone, local context, and transposition table
          let local_rules = rules_for(rule_set);
          let mut local_board = board.clone();
          let mut local_ctx = SearchContext {
            nodes: 0,
            candidate_set: HashSet::new(),
            killer_moves: [[None; KILLERS_PER_DEPTH]; MAX_KILLER_DEPTH],
            history: [[0; 15]; 15],
          };
          let mut local_tt = TranspositionTable::new(TT_SIZE);

          local_board.set(coord.x, coord.y, player);
          let mv = Move {
            x: coord.x,
            y: coord.y,
            player,
            t: None,
          };

          let score = if local_rules.check_win(&local_board, &mv).is_some() {
            WIN_SCORE
          } else {
            -negamax_parallel(
              &mut local_board,
              player.other(),
              current_depth.saturating_sub(1),
              -beta,  // Note: negated for negamax
              -alpha,
              local_rules.as_ref(),
              config.defense_weight,
              config.max_candidates,
              &mut local_ctx,
              &shared_ctx,
              &mut local_tt,
              1,    // Start at depth level 1 since we've already made one move
              true, // All root moves are treated as PV for parallel search
            )
          };

          // Accumulate local nodes to shared counter
          shared_ctx.nodes.fetch_add(local_ctx.nodes, Ordering::Relaxed);

          (score, coord)
        })
        .collect();

      // Find best score from this search
      let iter_best_score = scored.iter().map(|(s, _)| *s).max().unwrap_or(-WIN_SCORE);

      // Check if we need to re-search with wider window (aspiration fail)
      if iter_best_score <= alpha && alpha > -WIN_SCORE {
        // Fail-low: score is worse than expected, widen lower bound
        alpha = -WIN_SCORE;
        continue;
      }
      if iter_best_score >= beta && beta < WIN_SCORE {
        // Fail-high: score is better than expected, widen upper bound
        beta = WIN_SCORE;
        continue;
      }

      // Search succeeded within window
      iteration_complete = true;

      // Find best move from this iteration
      if let Some(&(score, coord)) = scored.iter().max_by_key(|(s, _)| *s) {
        best_move = Some(coord);
        best_score = score;
        aspiration_guess = Some(score);

        // Early termination if we found a winning move
        if score >= WIN_SCORE - 100 {
          // Signal to break outer loop
          candidates = scored.into_iter().map(|(_, c)| c).collect();
          break;
        }
      }

      // Re-sort candidates based on scores for next iteration
      let mut sorted_scored = scored;
      sorted_scored.sort_by(|a, b| b.0.cmp(&a.0));
      candidates = sorted_scored.into_iter().map(|(_, c)| c).collect();
    }

    // Check for winning move found
    if best_score >= WIN_SCORE - 100 {
      break;
    }
  }

  // Apply randomness to final selection from the best candidates
  if config.randomness > 0 {
    // Re-score at final depth for randomness selection
    let final_scored: Vec<(i32, Coord)> = candidates.iter().take(config.randomness as usize + 1)
      .map(|&c| {
        // Quick estimate based on previous search
        let idx = candidates.iter().position(|&x| x == c).unwrap_or(0);
        let score = best_score - (idx as i32 * 100); // Rough ordering
        (score, c)
      })
      .collect();
    pick_with_randomness(&final_scored, config.randomness)
  } else {
    best_move
  }
}

// PVS (Principal Variation Search) with negamax, transposition table, and killer moves
fn negamax_parallel(
  board: &mut Board,
  player: Player,
  depth: u8,
  mut alpha: i32,
  beta: i32,
  rules: &dyn RuleSet,
  defense_weight: i32,
  max_candidates: usize,
  ctx: &mut SearchContext,
  shared_ctx: &Arc<SharedSearchContext>,
  tt: &mut TranspositionTable,
  depth_level: usize, // Track current depth for killer move indexing
  is_pv_node: bool,   // Whether this is a Principal Variation node
) -> i32 {
  ctx.nodes += 1;

  // Check both local and shared node limits
  let total_nodes = shared_ctx.nodes.load(Ordering::Relaxed) + ctx.nodes;
  if depth == 0 || board.is_full() || total_nodes >= shared_ctx.max_nodes {
    return evaluate_board(board, player, defense_weight);
  }

  // Check transposition table
  let hash = board.zobrist_hash();
  let original_alpha = alpha;

  if let Some((tt_score, tt_flag)) = tt.probe(hash, depth) {
    match tt_flag {
      TTFlag::Exact => return tt_score,
      TTFlag::LowerBound => alpha = alpha.max(tt_score),
      TTFlag::UpperBound => {
        if tt_score < beta {
          return tt_score;
        }
      }
    }
    if alpha >= beta {
      return tt_score;
    }
  }

  let candidates = candidate_moves(board, player, max_candidates, ctx, depth_level);
  if candidates.is_empty() {
    return 0;
  }

  let mut best = -WIN_SCORE;
  let mut first_move = true;

  for coord in candidates {
    let mv = Move {
      x: coord.x,
      y: coord.y,
      player,
      t: None,
    };
    board.set(coord.x, coord.y, player);

    let score = if rules.check_win(board, &mv).is_some() {
      WIN_SCORE - depth as i32
    } else if first_move || !is_pv_node {
      // First move or non-PV node: full window search
      -negamax_parallel(
        board,
        player.other(),
        depth - 1,
        -beta,
        -alpha,
        rules,
        defense_weight,
        max_candidates,
        ctx,
        shared_ctx,
        tt,
        depth_level + 1,
        first_move && is_pv_node, // Only first move in PV is PV
      )
    } else {
      // PVS: Zero-window search for non-first moves
      let mut score = -negamax_parallel(
        board,
        player.other(),
        depth - 1,
        -alpha - 1, // Zero window: [alpha, alpha+1]
        -alpha,
        rules,
        defense_weight,
        max_candidates,
        ctx,
        shared_ctx,
        tt,
        depth_level + 1,
        false, // Zero-window search is never PV
      );

      // If zero-window search fails high, re-search with full window
      if score > alpha && score < beta {
        score = -negamax_parallel(
          board,
          player.other(),
          depth - 1,
          -beta,
          -alpha,
          rules,
          defense_weight,
          max_candidates,
          ctx,
          shared_ctx,
          tt,
          depth_level + 1,
          true, // Re-search is PV
        );
      }
      score
    };

    board.clear(coord.x, coord.y);
    first_move = false;

    if score > best {
      best = score;
    }
    if score > alpha {
      alpha = score;
    }
    if alpha >= beta {
      // Beta cutoff - record this move for move ordering (killer + history)
      record_cutoff(ctx, depth_level, depth, coord);
      break;
    }
  }

  // Store in transposition table
  let flag = if best <= original_alpha {
    TTFlag::UpperBound
  } else if best >= beta {
    TTFlag::LowerBound
  } else {
    TTFlag::Exact
  };
  tt.store(hash, depth, best, flag);

  best
}

fn candidate_moves(
  board: &mut Board,
  player: Player,
  max_candidates: usize,
  ctx: &mut SearchContext,
  depth: usize,
) -> Vec<Coord> {
  let size = board.size();
  let mut has_stones = false;
  // 复用HashSet，避免重复分配
  ctx.candidate_set.clear();
  let radius: i32 = 2;

  for y in 0..size {
    for x in 0..size {
      if board.get(x, y).is_some() {
        has_stones = true;
        for dy in -radius..=radius {
          for dx in -radius..=radius {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && ny >= 0 {
              let ux = nx as usize;
              let uy = ny as usize;
              if board.in_bounds(ux, uy) && board.get(ux, uy).is_none() {
                ctx.candidate_set.insert((ux, uy));
              }
            }
          }
        }
      }
    }
  }

  let mut candidates: Vec<Coord> = if !has_stones {
    vec![Coord {
      x: size / 2,
      y: size / 2,
    }]
  } else {
    ctx.candidate_set
      .iter()
      .map(|&(x, y)| Coord { x, y })
      .collect()
  };

  if candidates.len() > max_candidates {
    candidates = rank_candidates_with_killers(board, player, candidates, max_candidates, ctx, depth);
  } else if candidates.len() > 1 && depth < MAX_KILLER_DEPTH {
    // Sort by killer move and history priority even for small candidate sets
    sort_by_killer_and_history(&mut candidates, ctx, depth);
  }

  candidates
}

fn rank_candidates(
  board: &mut Board,
  player: Player,
  candidates: Vec<Coord>,
  max_candidates: usize,
) -> Vec<Coord> {
  let mut scored = Vec::with_capacity(candidates.len());
  for coord in candidates {
    board.set(coord.x, coord.y, player);
    let score = evaluate_board(board, player, 11);
    scored.push((score, coord));
    board.clear(coord.x, coord.y);
  }

  scored.sort_by(|a, b| b.0.cmp(&a.0));
  scored.truncate(max_candidates);
  scored.into_iter().map(|(_, coord)| coord).collect()
}

fn rank_candidates_with_killers(
  board: &mut Board,
  player: Player,
  candidates: Vec<Coord>,
  max_candidates: usize,
  ctx: &SearchContext,
  depth: usize,
) -> Vec<Coord> {
  let mut scored = Vec::with_capacity(candidates.len());
  for coord in candidates {
    board.set(coord.x, coord.y, player);
    let base_score = evaluate_board(board, player, 11);
    // Boost killer moves to prioritize them in the search order
    let killer_bonus = killer_priority(&coord, ctx, depth) * 100_000;
    // Add history heuristic bonus (scaled to not overpower killer moves)
    let history_bonus = ctx.history[coord.y][coord.x] as i32;
    scored.push((base_score + killer_bonus + history_bonus, coord));
    board.clear(coord.x, coord.y);
  }

  scored.sort_by(|a, b| b.0.cmp(&a.0));
  scored.truncate(max_candidates);
  scored.into_iter().map(|(_, coord)| coord).collect()
}

fn sort_by_killer_and_history(candidates: &mut Vec<Coord>, ctx: &SearchContext, depth: usize) {
  candidates.sort_by(|a, b| {
    // Primary: killer moves have highest priority
    let killer_a = killer_priority(a, ctx, depth) * 1_000_000;
    let killer_b = killer_priority(b, ctx, depth) * 1_000_000;
    // Secondary: history heuristic
    let history_a = ctx.history[a.y][a.x] as i32;
    let history_b = ctx.history[b.y][b.x] as i32;
    (killer_b + history_b).cmp(&(killer_a + history_a))
  });
}

fn killer_priority(coord: &Coord, ctx: &SearchContext, depth: usize) -> i32 {
  if depth >= MAX_KILLER_DEPTH {
    return 0;
  }
  let killers = &ctx.killer_moves[depth];
  if killers[0] == Some(*coord) {
    return 2; // Primary killer has highest priority
  }
  if killers[1] == Some(*coord) {
    return 1; // Secondary killer
  }
  0
}

fn record_cutoff(ctx: &mut SearchContext, depth_level: usize, remaining_depth: u8, coord: Coord) {
  // Record history heuristic (depth^2 bonus - deeper cutoffs are more valuable)
  let history_bonus = (remaining_depth as u32) * (remaining_depth as u32);
  ctx.history[coord.y][coord.x] = ctx.history[coord.y][coord.x].saturating_add(history_bonus);

  // Record killer move
  if depth_level >= MAX_KILLER_DEPTH {
    return;
  }
  // Don't store duplicate killers
  if ctx.killer_moves[depth_level][0] == Some(coord) {
    return;
  }
  // Shift and store new killer
  ctx.killer_moves[depth_level][1] = ctx.killer_moves[depth_level][0];
  ctx.killer_moves[depth_level][0] = Some(coord);
}

fn immediate_wins(
  board: &mut Board,
  player: Player,
  candidates: &[Coord],
  rules: &dyn RuleSet,
) -> Vec<Coord> {
  let mut wins = Vec::new();

  for coord in candidates.iter() {
    let mv = Move {
      x: coord.x,
      y: coord.y,
      player,
      t: None,
    };
    board.set(coord.x, coord.y, player);
    if rules.check_win(board, &mv).is_some() {
      wins.push(*coord);
    }
    board.clear(coord.x, coord.y);
  }

  wins
}

fn pick_best(board: &mut Board, player: Player, candidates: &[Coord], config: AiConfig) -> Option<Coord> {
  let mut scored = Vec::new();
  for coord in candidates.iter() {
    board.set(coord.x, coord.y, player);
    let score = evaluate_board(board, player, config.defense_weight);
    scored.push((score, *coord));
    board.clear(coord.x, coord.y);
  }

  scored.sort_by(|a, b| b.0.cmp(&a.0));
  pick_with_randomness(&scored, config.randomness)
}

fn pick_with_randomness(scored: &[(i32, Coord)], randomness: u8) -> Option<Coord> {
  if scored.is_empty() {
    return None;
  }

  if randomness == 0 {
    return Some(scored[0].1);
  }

  let bucket = usize::min(scored.len(), randomness as usize + 1);
  let mut rng = rand::thread_rng();
  scored[..bucket].choose(&mut rng).map(|(_, coord)| *coord)
}

fn evaluate_board(board: &Board, player: Player, defense_weight: i32) -> i32 {
  let my = score_for_player(board, player);
  let opp = score_for_player(board, player.other());
  let defense = (opp.score * defense_weight) / 10;
  my.score - defense
}

fn score_for_player(board: &Board, player: Player) -> ScoreBreakdown {
  let size = board.size();
  let mut total = ScoreBreakdown::default();

  // 横向扫描
  for y in 0..size {
    let scored = score_line_direct(board, player, 0, y, 1, 0, size);
    total.score += scored.score;
    total.open_threes += scored.open_threes;
    total.open_fours += scored.open_fours;
  }

  // 纵向扫描
  for x in 0..size {
    let scored = score_line_direct(board, player, x, 0, 0, 1, size);
    total.score += scored.score;
    total.open_threes += scored.open_threes;
    total.open_fours += scored.open_fours;
  }

  // 主对角线方向 (左上到右下)
  for start_x in 0..size {
    let line_len = size - start_x;
    if line_len >= 5 {
      let scored = score_line_direct(board, player, start_x, 0, 1, 1, line_len);
      total.score += scored.score;
      total.open_threes += scored.open_threes;
      total.open_fours += scored.open_fours;
    }
  }
  for start_y in 1..size {
    let line_len = size - start_y;
    if line_len >= 5 {
      let scored = score_line_direct(board, player, 0, start_y, 1, 1, line_len);
      total.score += scored.score;
      total.open_threes += scored.open_threes;
      total.open_fours += scored.open_fours;
    }
  }

  // 副对角线方向 (右上到左下)
  for start_x in 0..size {
    let line_len = start_x + 1;
    if line_len >= 5 {
      let scored = score_line_direct(board, player, start_x, 0, -1, 1, line_len);
      total.score += scored.score;
      total.open_threes += scored.open_threes;
      total.open_fours += scored.open_fours;
    }
  }
  for start_y in 1..size {
    let line_len = size - start_y;
    if line_len >= 5 {
      let scored = score_line_direct(board, player, size - 1, start_y, -1, 1, line_len);
      total.score += scored.score;
      total.open_threes += scored.open_threes;
      total.open_fours += scored.open_fours;
    }
  }

  total.score += center_bonus(board, player);
  if total.open_fours > 0 {
    total.score += 10_000;
  }
  if total.open_threes >= 2 {
    total.score += 6_000;
  }

  total
}

/// 直接在board上评估一条线，避免分配Vec
fn score_line_direct(
  board: &Board,
  player: Player,
  start_x: usize,
  start_y: usize,
  dx: i32,
  dy: i32,
  len: usize,
) -> ScoreBreakdown {
  let mut out = ScoreBreakdown::default();

  // 使用固定大小数组作为滑动窗口 (最大支持6格窗口用于模式匹配)
  let mut window: [i8; 6] = [0; 6];
  let mut window_pos = 0usize;

  let mut x = start_x as i32;
  let mut y = start_y as i32;

  // 连续棋子序列追踪
  let mut run_start_idx: Option<usize> = None;
  let mut prev_val: i8 = -1; // 用于追踪左侧是否开放

  for i in 0..len {
    let val = cell_value(board, x as usize, y as usize, player);

    // 处理连续序列
    if val == 1 {
      if run_start_idx.is_none() {
        run_start_idx = Some(i);
      }
    } else if let Some(start) = run_start_idx {
      // 连续序列结束
      let run_len = (i - start) as i32;
      let left_open = start > 0 && prev_val == 0;
      let right_open = val == 0;
      let open_ends = left_open as i32 + right_open as i32;

      out.score += run_score(run_len, open_ends);
      if run_len == 4 && open_ends == 2 {
        out.open_fours += 1;
      }
      if run_len == 3 && open_ends == 2 {
        out.open_threes += 1;
      }
      run_start_idx = None;
    }

    // 更新前一个值 (用于下一个序列的左侧开放判断)
    if val != 1 {
      prev_val = val;
    }

    // 滑动窗口模式匹配
    window[window_pos % 6] = val;
    window_pos += 1;

    if window_pos >= 6 {
      // 重构窗口为正确顺序
      let w = [
        window[(window_pos + 0) % 6],
        window[(window_pos + 1) % 6],
        window[(window_pos + 2) % 6],
        window[(window_pos + 3) % 6],
        window[(window_pos + 4) % 6],
        window[(window_pos + 5) % 6],
      ];
      // 6格模式
      if w == [0, 1, 1, 0, 1, 0] || w == [0, 1, 0, 1, 1, 0] {
        out.score += SCORE_BROKEN_THREE;
        out.open_threes += 1;
      }
      if w == [0, 1, 1, 1, 0, 1] || w == [1, 0, 1, 1, 1, 0] {
        out.score += SCORE_BROKEN_FOUR;
      }
    }

    if window_pos >= 5 {
      // 5格模式
      let w5 = [
        window[(window_pos + 1) % 6],
        window[(window_pos + 2) % 6],
        window[(window_pos + 3) % 6],
        window[(window_pos + 4) % 6],
        window[(window_pos + 5) % 6],
      ];
      if w5 == [0, 1, 0, 1, 0] {
        out.score += SCORE_OPEN_TWO;
      }
      if w5 == [1, 1, 1, 0, 1] || w5 == [1, 0, 1, 1, 1] {
        out.score += SCORE_BROKEN_FOUR;
      }
    }

    x += dx;
    y += dy;
  }

  // 处理末尾的连续序列
  if let Some(start) = run_start_idx {
    let run_len = (len - start) as i32;
    let left_open = start > 0 && prev_val == 0;
    let right_open = false; // 到达边界
    let open_ends = left_open as i32 + right_open as i32;

    out.score += run_score(run_len, open_ends);
    if run_len == 4 && open_ends == 2 {
      out.open_fours += 1;
    }
    if run_len == 3 && open_ends == 2 {
      out.open_threes += 1;
    }
  }

  out
}

fn cell_value(board: &Board, x: usize, y: usize, player: Player) -> i8 {
  match board.get(x, y) {
    None => 0,
    Some(p) if p == player => 1,
    _ => 2,
  }
}

fn run_score(len: i32, open_ends: i32) -> i32 {
  match (len, open_ends) {
    (5..=i32::MAX, _) => SCORE_FIVE,
    (4, 2) => SCORE_OPEN_FOUR,
    (4, 1) => SCORE_SEMI_FOUR,
    (3, 2) => SCORE_OPEN_THREE,
    (3, 1) => SCORE_SEMI_THREE,
    (2, 2) => SCORE_OPEN_TWO,
    (2, 1) => SCORE_SEMI_TWO,
    (1, 2) => SCORE_OPEN_ONE,
    _ => 0,
  }
}

fn center_bonus(board: &Board, player: Player) -> i32 {
  let size = board.size() as i32;
  let center = (size - 1) / 2;
  let mut score = 0;

  for y in 0..size {
    for x in 0..size {
      if board.get(x as usize, y as usize) == Some(player) {
        let dist = (x - center).abs() + (y - center).abs();
        score += (size - dist) / 3;
      }
    }
  }

  score
}
