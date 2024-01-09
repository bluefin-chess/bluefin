use std::arch::x86_64::_lzcnt_u32; // for fast log2
// use chess::{Board, ChessMove as Move, MoveGen};
use std::collections::HashMap;
use crate::evaluate::{evaluate_move, evaluate};
use shakmaty::{Chess as Board, Move, Position, zobrist::{Zobrist64, ZobristHash}, EnPassantMode};
use std::rc::Rc;
use std::cell::RefCell;
use crate::time::Timer;

pub struct Node {
  prior: f64,
  value_sum: RefCell<f64>,
  visits: RefCell<u32>, // 32 bits should be more than enough
  children: Option<Vec<(Move, Rc<Node>)>>,
}

type MoveList = Vec<(Move, Rc<Node>)>;
type TranspositionTable = HashMap<Zobrist64, Rc<Node>>;

impl Node {
  pub fn new(prior: f64, value_sum: f64, visits: u32, children: Option<Vec<(Move, Rc<Node>)>>) -> Node {
    Node {
      prior,
      value_sum: RefCell::new(value_sum),
      visits: RefCell::new(visits),
      children,
    }
  }

  fn has_unvisited_children(&self) -> bool {
    match self.children {
      Some(ref children) => children.iter().any(|(_, ref child)| *child.visits.borrow() == 0),
      None => false,
    }
  }

  fn select_best_child(&self) -> (Move, Rc<Node>) {
    let children = self.children.as_ref().unwrap();
    if children.is_empty() { panic!("no children to select from") }

    let mut best_child_score = f64::NEG_INFINITY;
    let mut best_child: Option<(Move, Rc<Node>)> = None;

    // there's definitely a more idiomatic way to do this but I couldn't find one that wasn't confusing to read
    for (mov, node) in children.iter() {
      let score = ucb1(*node.visits.borrow(), *node.value_sum.borrow(), *self.visits.borrow());
      if score > best_child_score {
        best_child_score = score;
        best_child = Some((mov.clone(), Rc::clone(node)));
      }
    }

    best_child.expect("no best child found")
  }

  fn best_move(&self) -> Move {
    let mut best_move: Option<&Move> = None;
    let mut most_visits = 0;
    let children = self.children.as_ref().unwrap();
    println!("children: {}", children.len());

    if children.is_empty() { panic!("no children to select from") }

    for (mov, node) in children.iter() {
      print!("{}: {}, ", mov, *node.visits.borrow());
      let visits = *node.visits.borrow();
      if visits > most_visits {
        most_visits = visits;
        best_move = Some(mov);
      }
    }
    println!();

    best_move.expect("no best move found").clone()
  }

}

// w = winning score, n = visits, c = exploration constant, N = parent visits
// original UCB 1: w/n + c * sqrt(ln(N) / n)
// convert ln into log2: w/n + c * sqrt(log2(N) / n * log2(e))
// note that we got rid of log2(e) because it only affects C, and that's a value we plan on playing around with anyway
pub fn ucb1(visits: u32, score: f64, parentvisits: u32) -> f64 {
  let c = 1f64; // exploration constant, we can play around with this value later
  let n = visits as f64;
  #[allow(non_snake_case)] // I want my capital N
  let log2_N: f64 = unsafe { (63 - _lzcnt_u32(parentvisits)) as f64 }; // fast log2
  (score / n) + c * (log2_N / n).sqrt() // we never divide by 0 because visits >= 1
}

pub struct Game {
  pub board: Board,
  pub trans_table: TranspositionTable,
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
        Rc::new(Node::new((e_m.0 - min) / sum, 0f64, 1, None))
      }).clone();

      (e_m.1.clone(), node)
    }).collect()
  }

  fn selection(board: &mut Board, node: &mut Node, path: &mut MoveList) { 
    while *node.visits.borrow() > 0 && node.has_unvisited_children() {
      let (mov, child) = node.select_best_child();
      board.play_unchecked(&mov);
      path.push((mov, child));
    }
  }

  fn expansion(board: &mut Board, node: &mut Node, path: &mut MoveList, trans_table: &mut TranspositionTable) {
    if !board.legal_moves().is_empty() {
      node.children = Some(Game::expand(board, trans_table));
      let (mov, child) = node.select_best_child();
      board.play_unchecked(&mov);
      path.push((mov, child));
    }
  }

  fn backpropagation(board: &Board, path: &mut MoveList) {
    let mut val = {
      if board.is_game_over() {
        // draw if not checkmate, checkmate is always 1 because you can't mate yourself
        if board.is_checkmate() { 1f64 } else { 0f64 }
      } else {
        evaluate(board)
      }
    };

    for mov in path.iter_mut().rev() {
      print!("{}: {}, ", mov.0, val);
      *mov.1.value_sum.borrow_mut() += val;
      *mov.1.visits.borrow_mut() += 1;
      val = -val; // flip for side
    }
    println!();
  }

  pub fn mcts(&mut self, root: &mut Node, timer: Timer) -> Move { // returns best move
    let mut board = self.board.clone();
    let mut iterations = 0;

    while /*timer.is_time_remaining_5()*/ iterations < 10 {
      
      iterations += 1;
      let mut path: Vec<(Move, Rc<Node>)> = Vec::new();

      Game::selection(&mut board, root, &mut path);
      Game::expansion(&mut board, root, &mut path, &mut self.trans_table);
      print!("backprop: ");
      Game::backpropagation(&board, &mut path);

    }
    
    println!("iterations: {}", iterations);
    print!("self.board: ");
    for mov in self.board.legal_moves() {
      print!("{}, ", mov);
    }
    println!();
    println!("trans table size: {}", self.trans_table.len());

    root.best_move()
  }
}