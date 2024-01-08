mod mcts;
mod evaluate;
mod time;
use shakmaty::Move;

fn main() {
  let mut node = mcts::Node::new(0f64, 0f64, 1, None);
  let mut game = mcts::Game::default();

  let best_move = game.mcts();
}