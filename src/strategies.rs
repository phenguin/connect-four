use game::{RandGame, Game, Score};
use std::fmt;
use rand::{XorShiftRng, Rng};
use rand;
use std::marker::PhantomData;
use rayon::prelude::*;

pub trait Strategy<G: Game> {
    type Params;
    fn decide(&mut self, &G) -> G::Move;
    fn create(Self::Params) -> Self;
}

struct NegamaxState {
    rng: XorShiftRng,
}
pub struct NegamaxParams {
    pub max_depth: usize,
    pub trials: usize,
}

pub struct Negamax<G> {
    pub params: NegamaxParams,
    state: NegamaxState,
    _phantom: PhantomData<G>,
}



impl<G> Negamax<G>
    where G: RandGame + Send + fmt::Display,
          G::Agent: Send,
          G::Move: Send + Ord
{
    fn heuristic(&mut self, game: &G) -> usize {
        let trials = self.params.trials;
        if let Some(c) = game.winner() {
            return if c == game.ref_player() { trials } else { 0 };
        };

        game.monte_carlo(&mut self.state.rng, trials as u32) as usize

    }

    pub fn negamax(&mut self, game: &G, depth: usize) -> (Score, Option<G::Move>) {
        let NegamaxParams { trials, max_depth, .. } = self.params;

        let nexts = game.possible_moves();
        if depth > max_depth || nexts.is_empty() {
            return (self.heuristic(game) as Score * game.player_weight(&game.to_act()), None);
        }
        nexts.into_iter()
            .map(|mv| {
                let m = mv.valid_move().clone();
                let new_game = mv.apply();
                let (s, _) = self.negamax(&new_game, depth + 1);
                (-s, Some(m))
            })
            .max()
            .unwrap()
    }
}

impl<G> Strategy<G> for Negamax<G>
    where G: RandGame + Send + fmt::Display,
          G::Agent: Send,
          G::Move: Send + Ord
{
    type Params = NegamaxParams;
    fn decide(&mut self, game: &G) -> G::Move {
        let (_, maybe_move) = self.negamax(game, 0);
        maybe_move.expect("No moves available from start position.")
    }
    fn create(params: NegamaxParams) -> Self {
        let seed = rand::random::<[u32; 4]>();
        let rng: XorShiftRng = rand::SeedableRng::from_seed(seed);
        Self {
            params: params,
            state: NegamaxState { rng: rng },
            _phantom: PhantomData,
        }
    }
}
