use std::arch::x86_64::_lzcnt_u32; // for fast log2
// use chess::{Board, ChessMove as Move, MoveGen};
use std::collections::HashMap;
use crate::evaluate::evaluate_move;
use shakmaty::{Chess, Move, Position, zobrist::{ZobristHash, Zobrist64, ZobristValue}, EnPassantMode};

// fast inverse square root from Quake III Arena
// https://en.wikipedia.org/wiki/Fast_inverse_square_root
/*
fn fast_inv_sqrt(x: f32) -> f32 {
  let xhalf = 0.5 * x;
  let mut i = x.to_bits();
  i = 0x5f3759df - (i >> 1);
  let x = f32::from_bits(i);
  x * (1.5 - xhalf * x * x)
  // x * (1.5 - xhalf * x * x)
}
*/

struct Node {
  prior: f64,
  value_sum: f64,
  visits: u32, // 32 bits should be more than enough
  children: Option<Vec<(Move, Node)>>,
}

impl Node {
  fn new(prior: f64, value_sum: f64, visits: u32, children: Option<Vec<(Move, Node)>>) -> Node {
    Node {
      prior,
      value_sum,
      visits,
      children,
    }
  }
}

// w = winning score, n = visits, c = exploration constant, N = parent visits
// original UCB 1: w/n + c * sqrt(ln(N) / n)
// convert ln into log2: w/n + c * sqrt(log2(N) / n * log2(e))
// flip fraction to use fast inv sqrt: w/n + c * inv_sqrt(n / log2(N))
// note that we got rid of log2(e) because it only affects C, and that's a value we plan on playing around with anyway
pub fn ucb1(visits: u32, score: f32, parentvisits: u32) -> f32 {
  let c = 1f32; // exploration constant, we can play around with this value later
  let n = visits as f32;
  #[allow(non_snake_case)] // I want my capital N
  let log2_N: f32 = unsafe { (63 - _lzcnt_u32(parentvisits)) as f32 }; // fast log2
  (score / n) + c * (log2_N / n).sqrt() // we never divide by 0 because visits >= 1
}

/* 
fn ucb1_inv(visits: u32, score: f32, parentvisits: u32) -> f32 {
  let c = 1f32; // exploration constant, we can play around with this value later
  let n = visits as f32;
  let log2_n = unsafe { (63 - _lzcnt_u64(parentvisits)) as f32 }; // fast log2
  (score / n) + c * fast_inv_sqrt(n / log2_n)
}
*/

struct Game {
  board: Position,
  trans_table: HashMap<ZobristValue, Node>,
}

impl Game {
  fn zoby(&self) -> ZobristValue {
    ZobristHash::zobrist_hash(&self.board, EnPassantMode::Legal)
  }
  
  pub fn expand(&mut self) -> Vec<(Move, Node)> {
    let moves = self.board.legal_moves();
    let evaluations: Vec<(f32, Move)> = moves
      .into_iter()
      .map(|m| (self.evaluate_move(m), m.clone()))
      .collect();
    
    let min = evaluations.iter().map(|e_m| e_m.0).fold(f32::INFINITY, f32::min) - 0.01; // get val slightly smaller than min
    let sum = evaluations.iter().map(|e_m| e_m.0 - min).sum(); // sum of all values - min

    evaluations.iter().map(|e_m| {
      let mut temp_board = self.board.clone();
      temp_board.play_unchecked(e_m.1); // don't need to check legality because we're only playing from legal moves
      let key = zoby(&temp_board);
      if let Some(node) = self.trans_table.get(&key) {
        (e_m.1, node)
      } else {
        let mut new_node = Node::new(
          (e_m.1 - min) / sum,
          0f64,
          0,
          None,
        );
        self.trans_table.insert(key, new_node);
        (e_m.1, new_node)
      }
    }).collect()
  }
}