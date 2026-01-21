use crate::engine::Board;
use crate::types::{GameResult, Move, Player, RuleSetKind};

pub trait RuleSet {
  fn is_legal(&self, board: &Board, mv: &Move) -> bool;
  fn check_win(&self, board: &Board, mv: &Move) -> Option<GameResult>;
}

pub struct StandardRuleSet;

impl RuleSet for StandardRuleSet {
  fn is_legal(&self, board: &Board, mv: &Move) -> bool {
    board.in_bounds(mv.x, mv.y) && board.is_empty(mv.x, mv.y)
  }

  fn check_win(&self, board: &Board, mv: &Move) -> Option<GameResult> {
    let player = mv.player;
    let directions = [(1, 0), (0, 1), (1, 1), (1, -1)];

    for (dx, dy) in directions {
      let mut count = 1;
      count += count_dir(board, mv.x, mv.y, dx, dy, player);
      count += count_dir(board, mv.x, mv.y, -dx, -dy, player);

      if count >= 5 {
        return Some(match player {
          Player::B => GameResult::BWin,
          Player::W => GameResult::WWin,
        });
      }
    }

    None
  }
}

pub fn rules_for(kind: RuleSetKind) -> Box<dyn RuleSet> {
  match kind {
    RuleSetKind::Standard => Box::new(StandardRuleSet),
  }
}

fn count_dir(board: &Board, x: usize, y: usize, dx: i32, dy: i32, player: Player) -> usize {
  let mut count = 0;
  let mut cx = x as i32 + dx;
  let mut cy = y as i32 + dy;

  while cx >= 0 && cy >= 0 {
    let ux = cx as usize;
    let uy = cy as usize;
    if !board.in_bounds(ux, uy) {
      break;
    }
    if board.get(ux, uy) != Some(player) {
      break;
    }
    count += 1;
    cx += dx;
    cy += dy;
  }

  count
}
