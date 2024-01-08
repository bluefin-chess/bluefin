mod mcts;
mod evaluate;
mod time;
use shakmaty::Move;

fn main() {
  let mut node = mcts::Node::new(0f64, 0f64, 1, None);
  let mut game = mcts::Game::default();

  let best_move: Move = game.mcts(&mut node, time::Timer::new(std::time::Duration::from_secs(3)));

  println!("best move: {}", best_move);
}