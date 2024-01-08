use std::arch::x86_64::_lzcnt_u32; // for fast log2
// use chess::{Board, ChessMove as Move, MoveGen};
use std::collections::HashMap;
use crate::evaluate::evaluate_move;
use shakmaty::{Chess as Board, Move, Position, zobrist::{self, Zobrist64, ZobristHash}, EnPassantMode};
use std::rc::Rc;
use crate::time::Timer;

// fast inverse square root from Quake III Arena
// https://en.wikipedia.org/wiki/Fast_inverse_square_root
/*
fn fast_inv_sqrt(x: f64) -> f64 {
  let xhalf = 0.5 * x;
  let mut i = x.to_bits();
  i = 0x5f3759df - (i >> 1);
  let x = f64::from_bits(i);
  x * (1.5 - xhalf * x * x)
  // x * (1.5 - xhalf * x * x)
}
*/

pub struct Node {
  prior: f64,
  value_sum: f64,
  visits: u32, // 32 bits should be more than enough
  children: Option<Vec<(Move, Rc<Node>)>>,
}

type move_list = Vec<(Move, Rc<Node>)>;

impl Node {
  pub fn new(prior: f64, value_sum: f64, visits: u32, children: Option<Vec<(Move, Rc<Node>)>>) -> Node {
    Node {
      prior,
      value_sum,
      visits,
      children,
    }
  }

  fn has_unvisited_children(&self) -> bool {
    match self.children {
      Some(ref children) => children.iter().any(|(_, ref child)| child.visits == 0),
      None => false,
    }
  }

}

// w = winning score, n = visits, c = exploration constant, N = parent visits
// original UCB 1: w/n + c * sqrt(ln(N) / n)
// convert ln into log2: w/n + c * sqrt(log2(N) / n * log2(e))
// flip fraction to use fast inv sqrt: w/n + c * inv_sqrt(n / log2(N))
// note that we got rid of log2(e) because it only affects C, and that's a value we plan on playing around with anyway
pub fn ucb1(visits: u32, score: f64, parentvisits: u32) -> f64 {
  let c = 1f64; // exploration constant, we can play around with this value later
  let n = visits as f64;
  #[allow(non_snake_case)] // I want my capital N
  let log2_N: f64 = unsafe { (63 - _lzcnt_u32(parentvisits)) as f64 }; // fast log2
  (score / n) + c * (log2_N / n).sqrt() // we never divide by 0 because visits >= 1
}

/* 
fn ucb1_inv(visits: u32, score: f64, parentvisits: u32) -> f64 {
  let c = 1f64; // exploration constant, we can play around with this value later
  let n = visits as f64;
  let log2_n = unsafe { (63 - _lzcnt_u64(parentvisits)) as f64 }; // fast log2
  (score / n) + c * fast_inv_sqrt(n / log2_n)
}
*/

pub struct Game {
  board: Board,
  trans_table: HashMap<zobrist::Zobrist64, Rc<Node>>,
}

impl Game {
  pub fn default() -> Game {
    Game {
      board: Board::default(),
      trans_table: HashMap::new(),
    }
  }
  
  fn zoby(board: &Board) -> Zobrist64 {
    board.zobrist_hash(EnPassantMode::Legal)
  }
  
  fn expand(&mut self) -> Vec<(Move, Rc<Node>)> {
    let moves = self.board.legal_moves();
    let evaluations: Vec<(f64, Move)> = moves
      .into_iter()
      .map(|m| {
        let m_clone = m.clone();
        (evaluate_move(&self.board, m_clone), m)
      })
      .collect();

    let min = evaluations.iter().map(|e_m| e_m.0).fold(f64::INFINITY, f64::min) - 0.01;
    let sum: f64 = evaluations.iter().map(|e_m| e_m.0 - min).sum();

    evaluations.iter().map(|e_m| {
      let mut temp_board = self.board.clone();
      temp_board.play_unchecked(&e_m.1);
      let key = Game::zoby(&temp_board);
      
      // get node from key in table or insert new node if it doesn't exist
      let node: Rc<Node> = self.trans_table.entry(key).or_insert_with(|| {
        Rc::new(Node::new((e_m.0 - min) / sum, 0f64, 0, None))
      }).clone();

      (e_m.1.clone(), node)
    }).collect()
  }

  fn selection(node: &mut Node, path: &mut move_list) { // modifies 
    while node.visits > 0 && node.has_unvisited_children() {

    }
  }

  fn expansion(&mut self, node: &mut Node) {
    if !self.board.legal_moves().is_empty()  {
      node.children = Some(self.expand());
      
    }
  }

  fn backpropagation(&mut self) {}

  pub fn mcts(&mut self, root: &mut Node) -> Move { // returns best move
    let node = root;
    let mut path: Vec<(Move, Node)> = Vec::new();

    root.children[0]
  }
}