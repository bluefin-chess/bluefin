mod mcts;
mod evaluate;
mod time;

use shakmaty::{Move, Position};

fn main() {
  let mut node = mcts::Node::new(1f64, 1, None);
  let mut game = mcts::Game::default();

  println!("default eval: {}", evaluate::evaluate(&game.board));

  let best_move: Move = game.mcts(&mut node, time::Timer::new(std::time::Duration::from_secs(5)));

  println!("best move: {}", best_move);
}