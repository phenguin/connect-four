use game::{Game, Score};
use std::fmt;
use rand::{XorShiftRng, Rng};
use rand;
use std::marker::PhantomData;
use rayon::prelude::*;

#[derive(Clone)]
struct Node<G: Game> {
    game: G,
    attrs: NodeInfo<G>,
}

impl<G> fmt::Display for Node<G>
    where G: Game + fmt::Display,
          G::Agent: fmt::Display,
          G::Move: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.attrs)?;
        writeln!(f, "{}", self.game)
    }
}

#[derive(Clone)]
struct NodeInfo<G: Game> {
    preceding_move: Option<G::Move>,
    rng: XorShiftRng,
    max_depth: usize,
}

impl<G> fmt::Display for NodeInfo<G>
    where G: Game + fmt::Display,
          G::Agent: fmt::Display,
          G::Move: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self.preceding_move {
            None => String::from("<ROOT>"),
            Some(m) => format!("{}", m),
        };
        writeln!(f, "last move: {}", s)
    }
}

impl<G: Game> Default for NodeInfo<G> {
    fn default() -> Self {
        let seed = rand::random::<[u32; 4]>();
        Self::new_seeded(seed, 1)
    }
}

impl<G: Game> NodeInfo<G> {
    fn new_seeded(seed: [u32; 4], max_depth: usize) -> Self {
        let rng: XorShiftRng = rand::SeedableRng::from_seed(seed);

        NodeInfo {
            preceding_move: None,
            rng: rng,
            max_depth: max_depth,
        }
    }
}

pub trait Strategy<G: Game> {
    type Params;
    fn decide(&mut self, &G) -> G::Move;
    fn create(Self::Params) -> Self;
}

struct NegamaxState {}
pub struct NegamaxParams {
    pub max_depth: usize,
    pub trials: usize,
}

pub struct Negamax<G>
    where G: Game + Send,
          G::Agent: Send,
          G::Move: Send + Ord
{
    params: NegamaxParams,
    state: NegamaxState,
    _phantom: PhantomData<G>,
}



impl<G> Negamax<G>
    where G: Game + Send + fmt::Display,
          G::Agent: Send,
          G::Move: Send + Ord
{
    fn negamax(&mut self, game: &G) -> (Score, Option<G::Move>) {
        let NegamaxParams { trials, max_depth, .. } = self.params;
        let mut node = Node::new(game, max_depth);
        node.negamax(trials, 0)
    }
}

impl<G> Strategy<G> for Negamax<G>
    where G: Game + Send + fmt::Display,
          G::Agent: Send,
          G::Move: Send + Ord
{
    type Params = NegamaxParams;
    fn decide(&mut self, game: &G) -> G::Move {
        let (_, maybe_move) = self.negamax(game);
        maybe_move.expect("No moves available from start position.")
    }
    fn create(params: NegamaxParams) -> Self {
        Self {
            params: params,
            state: NegamaxState {},
            _phantom: PhantomData,
        }
    }
}


impl<G> Node<G>
    where G: Game + Send + fmt::Display,
          G::Agent: Send,
          G::Move: Send + Ord
{
    fn new_seeded(g: &G, seed: [u32; 4], max_depth: usize) -> Self {
        Node {
            game: g.clone(),
            attrs: NodeInfo::new_seeded(seed, max_depth),
        }
    }

    fn new(g: &G, max_depth: usize) -> Self {
        let seed = rand::random::<[u32; 4]>();
        Self::new_seeded(g, seed, max_depth)
    }

    fn max_depth(&self) -> usize {
        self.attrs.max_depth
    }

    fn rng(&mut self) -> &mut XorShiftRng {
        &mut self.attrs.rng
    }

    fn preceding_move(&self) -> Option<G::Move> {
        self.attrs.preceding_move
    }

    fn negamax(&mut self, trials: usize, depth: usize) -> (Score, Option<G::Move>) {
        let nexts = self.possibilities();
        if depth > self.max_depth() || nexts.is_empty() {
            return (self.heuristic(trials) as Score * self.game.player_weight(&self.game.to_act()),
                    None);
        }
        nexts.into_par_iter()
            .map(|mut node| {
                let (s, _) = node.negamax(trials, depth + 1);
                (-s, node.preceding_move())
            })
            .max()
            .unwrap()
    }

    // TODO Make this not super slow.
    fn random_outcome(&mut self) -> Option<G::Agent> {
        let mut game = (*self).game.clone();

        while !game.has_winner() {
            let mut moves = game.possible_moves();

            if moves.is_empty() {
                return None;
            }

            self.rng().shuffle(&mut moves);
            let m = moves[0].clone();
            game = m.apply();
            if game.has_winner() {
                return game.winner();
            }
        }
        None
    }

    fn monte_carlo(&mut self, trials: u32) -> u32 {
        let ref_color = self.game.ref_player();
        (0..trials)
            .flat_map(|_| self.random_outcome())
            .filter(move |c| *c == ref_color)
            .map(|_| 1)
            .sum()
    }

    fn heuristic(&mut self, trials: usize) -> usize {
        if let Some(c) = self.game.winner() {
            return if c == self.game.ref_player() {
                trials
            } else {
                0
            };
        };

        self.monte_carlo(trials as u32) as usize

    }

    fn possibilities(&mut self) -> Vec<Self> {
        let moves = self.game.possible_moves();
        // self.rng().shuffle(moves.as_mut_slice());
        moves.into_iter()
            .map(move |m| {
                let mut new_attrs = self.attrs.clone();
                new_attrs.preceding_move = Some(*m.valid_move());
                Node {
                    game: m.apply(),
                    attrs: new_attrs,
                }

            })
            .collect()
    }
}
