use game::Game;

pub trait Strategy<G: Game> {
    type Params;
    fn decide(&mut self, &G) -> G::Move;
    fn create(Self::Params) -> Self;
}

pub mod negamax;
pub mod mcts;
pub mod mcts_parallel;
pub mod mcts_rayon;
