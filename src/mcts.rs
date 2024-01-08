use std::arch::x86_64::_lzcnt_u32; // for fast log2
// use chess::{Board, ChessMove as Move, MoveGen};
use std::collections::HashMap;
use crate::evaluate::{evaluate_move, evaluate};
use shakmaty::{Chess as Board, Move, Position, zobrist::{Zobrist64, ZobristHash}, EnPassantMode};
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

type MoveList = Vec<(Move, Rc<Node>)>;
type TranspositionTable = HashMap<Zobrist64, Rc<Node>>;

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

  fn select_best_child(&self) -> (Move, Rc<Node>) {
    let children = self.children.as_ref().unwrap();
    let mut best_child_score = f64::NEG_INFINITY;
    let mut best_child: (Move, Rc<Node>);

    // there's definitely a more idiomatic way to do this but I couldn't find one that wasn't confusing to read
    for (mov, node) in children.iter() {
      let score = ucb1(node.visits, node.value_sum, self.visits);
      if score > best_child_score {
        best_child_score = score;
        best_child = (mov.clone(), Rc::clone(node));
      }
    }

    best_child
  }

  fn best_move(&self) -> Move {
    let mut best_move;
    let mut most_visits = 0;

    for (mov, node) in self.children.as_ref().unwrap().iter() {
      if node.visits > most_visits {
        most_visits = node.visits;
        best_move = mov.clone();
      }
    }

    best_move
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
  trans_table: TranspositionTable,
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

  fn expand(board: &mut Board, trans_table: &mut TranspositionTable) -> Vec<(Move, Rc<Node>)> {
    let moves = board.legal_moves();
    let evaluations: Vec<(f64, Move)> = moves
      .into_iter()
      .map(|m| {
        let m_clone = m.clone();
        (evaluate_move(board, m_clone), m)
      })
      .collect();

    let min = evaluations.iter().map(|e_m| e_m.0).fold(f64::INFINITY, f64::min) - 0.01;
    let sum: f64 = evaluations.iter().map(|e_m| e_m.0 - min).sum();

    evaluations.iter().map(|e_m| {
      let mut temp_board = board.clone();
      temp_board.play_unchecked(&e_m.1);
      let key = Game::zoby(&temp_board);
      
      // get node from key in table or insert new node if it doesn't exist
      let node: Rc<Node> = trans_table.entry(key).or_insert_with(|| {
        Rc::new(Node::new((e_m.0 - min) / sum, 0f64, 0, None))
      }).clone();

      (e_m.1.clone(), node)
    }).collect()
  }

  fn selection(board: &mut Board, node: &mut Node, path: &mut MoveList) { 
    while node.visits > 0 && node.has_unvisited_children() {
      let (mov, child) = node.select_best_child();
      board.play_unchecked(&mov);
      path.push((mov, child));
    }
  }

  fn expansion(board: &mut Board, node: &mut Node, path: &mut MoveList, trans_table: &mut TranspositionTable) {
    if !board.legal_moves().is_empty()  {
      node.children = Some(Game::expand(board, trans_table));
      let (mov, child) = node.select_best_child();
      board.play_unchecked(&mov);
      path.push((mov, child));
    }
  }

  fn backpropagation(board: &Board, path: &mut MoveList) {
    let mut val = {
      if board.is_game_over() {
        // draw if not checkmate
        if board.is_checkmate() { 1f64 } else { 0f64  }
      } else {
        evaluate(&board)
      }
    };

    for mov in path.iter_mut().rev() {
      mov.1.value_sum += val;
      mov.1.visits += 1;
      val = -val; // flip for side
    }
  }

  pub fn mcts(&mut self, root: &mut Node, timer: Timer) -> Move { // returns best move
    let mut board = self.board.clone();

    while timer.is_time_remaining_5() {
      let mut path: Vec<(Move, Rc<Node>)> = Vec::new();

      Game::selection(&mut board, root, &mut path);
      let leaf: Rc<Node> = path.last_mut().unwrap().1;
      Game::expansion(&mut board, root, &mut path, &mut self.trans_table);
      Game::backpropagation(&board, &mut path);
    }
    
    root.best_move()
  }
}