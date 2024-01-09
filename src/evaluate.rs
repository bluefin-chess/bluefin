use shakmaty::{Chess as Board, Move, Position};
use rand::Rng;
use std::arch::x86_64::_popcnt64;
// CNN evaluation function
// ideas for inputs: 64 squares w/ piece numbers and 2 more for enpassant and castle rights, basically a fen

pub fn evaluate_move(board: &Board, mov: Move) -> f64 {
  0f64
}

fn count_ones(n: u64) -> f64 {
  unsafe { _popcnt64(n as i64) as f64 }
}

pub fn evaluate(board: &Board) -> f64 {
  count_material(board) // + rand::thread_rng().gen_range(-0.1..0.1)
}

pub fn count_material(board: &Board) -> f64 {
  let mut white_score = 0f64;
  let mut black_score = 0f64;
  white_score += count_ones(board.board().pawns().0 & board.board().occupied().0 & board.board().white().0);
  white_score += count_ones(board.board().knights().0 & board.board().occupied().0 & board.board().white().0) * 3f64;
  white_score += count_ones(board.board().bishops().0 & board.board().occupied().0 & board.board().white().0) * 3f64;
  white_score += count_ones(board.board().rooks().0 & board.board().occupied().0 & board.board().white().0) * 5f64;
  white_score += count_ones(board.board().queens().0 & board.board().occupied().0 & board.board().white().0) * 9f64;

  black_score += count_ones(board.board().pawns().0 & board.board().occupied().0 & board.board().black().0);
  black_score += count_ones(board.board().knights().0 & board.board().occupied().0 & board.board().black().0) * 3f64;
  black_score += count_ones(board.board().bishops().0 & board.board().occupied().0 & board.board().black().0) * 3f64;
  black_score += count_ones(board.board().rooks().0 & board.board().occupied().0 & board.board().black().0) * 5f64;
  black_score += count_ones(board.board().queens().0 & board.board().occupied().0 & board.board().black().0) * 9f64;

  (white_score - black_score) * { if board.turn() == shakmaty::Color::White { 1f64 } else { -1f64 } }
}