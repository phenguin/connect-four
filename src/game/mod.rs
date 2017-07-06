pub mod connectfour;

#[derive(Hash, Clone)]
pub struct ValidMove<G: Game> {
    valid_move: G::Move,
    valid_for: G,
}

pub struct ValidMoveMut<'gs, G: Game + 'gs> {
    valid_move: G::Move,
    valid_for: &'gs mut G,
}

impl<G: Game> ValidMove<G> {
    pub fn valid_move(&self) -> &G::Move {
        &self.valid_move
    }

    pub fn apply(mut self) -> G {
        self.valid_for.apply(self.valid_move);
        self.valid_for
    }
}

impl<'gs, G: Game> ValidMoveMut<'gs, G> {
    pub fn valid_move(&self) -> &G::Move {
        &self.valid_move
    }

    #[allow(dead_code)]
    fn apply(self) {
        self.valid_for.apply(self.valid_move);
    }
}

pub type Score = i32;

pub trait ParseGame: Game {
    fn parse_move(&self, &str) -> Option<Self::Move>;
}

use rand::Rng;
pub trait RandGame: Game + Clone {
    fn random_move<R: Rng>(&mut self, rng: &mut R) -> Option<ValidMoveMut<Self>> {
        let mut moves = self.possible_moves();
        rng.shuffle(&mut moves);
        moves.into_iter().next().map(move |m| {
            ValidMoveMut {
                valid_move: m.valid_move,
                valid_for: self,
            }
        })
    }

    fn random_outcome<R: Rng>(&self, rng: &mut R) -> Option<Self::Agent> {
        let mut game = self.clone();

        while !game.has_winner() {
            match game.random_move(rng) {
                None => return None,
                Some(m) => m.apply(),
            }
            if game.has_winner() {
                return game.winner();
            }
        }
        None
    }

    fn monte_carlo<R: Rng>(&self, rng: &mut R, trials: u32) -> u32 {
        let ref_player = self.ref_player();
        (0..trials)
            .flat_map(move |_| self.random_outcome(rng))
            .filter(move |c| *c == ref_player)
            .map(|_| 1)
            .sum()
    }
}

pub trait Game: Clone + Send {
    type Move: Clone + Copy + Send + Ord;
    type Agent: PartialEq + Clone + Copy + Send + Ord;
    fn to_act(&self) -> Self::Agent;
    fn player_weight(&self, &Self::Agent) -> Score;
    fn winner(&self) -> Option<Self::Agent>;
    fn agent_id(&self, &Self::Agent) -> u32;

    fn ref_player(&self) -> Self::Agent;
    fn new(&Self::Agent) -> Self;
    fn possible_moves(&self) -> Vec<ValidMove<Self>>;

    fn try_move_mut(&mut self, m: Self::Move) -> bool {
        self.verify_move_mut(m).map(|m| m.apply()).is_some()
    }
    fn verify_move_mut(&mut self, m: Self::Move) -> Option<ValidMoveMut<Self>> {
        if !self.move_valid(&m) {
            return None;
        }

        Some(ValidMoveMut {
            valid_move: m,
            valid_for: self,
        })
    }

    fn verify_move(self: Self, m: Self::Move) -> Option<ValidMove<Self>> {
        if !self.move_valid(&m) {
            return None;
        }

        if self.has_winner() {
            return None;
        }

        Some(ValidMove {
            valid_move: m,
            valid_for: self,
        })
    }

    fn move_valid(&self, &Self::Move) -> bool;

    fn has_won(&self, agent: &Self::Agent) -> bool;

    fn has_winner(&self) -> bool {
        self.winner().is_some()
    }

    fn apply(&mut self, Self::Move);

    fn try_move(&mut self, m: Self::Move) -> bool {
        let x = self.clone().verify_move(m);
        x.map(move |valid_move| {
                let new_board = valid_move.apply();
                *self = new_board;
            })
            .is_some()
    }

    fn try_moves<I>(&mut self, moves: I) -> Vec<bool>
        where I: Iterator<Item = Self::Move>
    {
        moves.map(move |m| self.try_move(m))
            .collect()
    }
}

