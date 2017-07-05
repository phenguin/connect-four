
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

    fn random_outcome<R: Rng>(&mut self, rng: &mut R) -> Option<Self::Agent> {
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

    fn monte_carlo<R: Rng>(&mut self, rng: &mut R, trials: u32) -> u32 {
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

pub mod connectfour {
    use self::Color::*;
    use self::Slot::*;
    use rand;
    use std::fmt;
    use std::clone::Clone;
    use super::{Game, ParseGame, Score, ValidMove};
    use std::str::FromStr;

    const HEIGHT: usize = 6;
    const WIDTH: usize = 7;
    const NEEDED: usize = 4;

    #[derive(Clone, Copy, PartialOrd, PartialEq, Hash, Debug, Ord, Eq)]
    pub enum Color {
        R,
        B,
    }

    impl rand::Rand for Color {
        fn rand<R: rand::Rng>(rng: &mut R) -> Color {
            if rng.gen() {
                return R;
            }
            B
        }
    }

    impl Color {
        fn flip(&self) -> Color {
            match *self {
                R => B,
                B => R,
            }
        }

        #[allow(dead_code)]
        fn show(&self) -> &str {
            match *self {
                R => "X",
                B => "@",
            }
        }
    }

    #[derive(Clone, Copy, PartialEq, Hash, Debug)]
    enum Slot {
        Empty,
        Full(Color),
    }

    impl Slot {
        fn new() -> Self {
            Slot::Empty
        }
    }

    impl fmt::Display for Color {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f,
                   "{}",
                   match *self {
                       Color::R => "X",
                       Color::B => "@",
                   })
        }
    }

    impl fmt::Display for Slot {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                Empty => write!(f, " "),
                Full(c) => write!(f, "{}", c),
            }
        }
    }

    #[derive(Hash, Debug)]
    struct C4Board {
        board: [[Slot; WIDTH]; HEIGHT],
    }
    impl Clone for C4Board {
        fn clone(&self) -> C4Board {
            C4Board { board: self.board }
        }
    }

    impl C4Board {
        fn get(&self, i: usize, j: usize) -> Slot {
            self.board[i][j]
        }

        fn set(&mut self, i: usize, j: usize, c: Color) {
            (*self).board[i][j] = Slot::Full(c);
        }
    }

    impl fmt::Display for C4Board {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let dashes: String = (0..WIDTH * 3).map(|_| "-").collect();
            for row in self.board.iter().rev() {
                write!(f, "|")?;
                for slot in row.iter() {
                    write!(f, " {} ", slot)?;
                }
                writeln!(f, "|")?;
            }
            writeln!(f, "|{}|", dashes.as_str())?;
            write!(f, " ")?;
            for i in 0..WIDTH {
                write!(f, " {} ", i + 1)?;
            }
            writeln!(f, " ")
        }
    }

    #[derive(Hash, Clone, Debug)]
    pub struct ConnectFour {
        state: C4Board,
        to_act: Color,
        ref_player: Color,
        winner: Option<Color>,
    }

    impl fmt::Display for ConnectFour {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            writeln!(f, "Acting: {}, Ref: {}", self.to_act, self.ref_player)?;
            writeln!(f, "{}", self.state)
        }
    }

    impl ParseGame for ConnectFour {
        fn parse_move(&self, input: &str) -> Option<Self::Move> {
            usize::from_str(input).ok().map(|n| (n, self.to_act()))
        }
    }

    impl Game for ConnectFour {
        type Move = (usize, Self::Agent);
        type Agent = Color;

        fn agent_id(&self, &a: &Self::Agent) -> u32 {
            match a {
                R => 0,
                B => 1,
            }
        }

        fn winner(&self) -> Option<Self::Agent> {
            self.winner
        }

        fn ref_player(&self) -> Self::Agent {
            self.ref_player
        }

        fn move_valid(&self, &m: &Self::Move) -> bool {
            // TODO : Is this fine?
            let state = &self.state;
            let (col, color) = m;
            if let Slot::Full(_) = state.board[HEIGHT - 1][col] {
                return false;
            }

            if col >= WIDTH || color != self.to_act {
                return false;
            }

            true
        }

        fn new(&start: &Self::Agent) -> Self {
            let board = C4Board { board: [[Slot::new(); WIDTH]; HEIGHT] };
            ConnectFour {
                to_act: start,
                ref_player: start,
                state: board,
                winner: None,
            }
        }

        fn to_act(&self) -> Self::Agent {
            self.to_act
        }

        fn player_weight(&self, &p: &Self::Agent) -> Score {
            if p == self.ref_player() {
                return 1;
            }

            -1
        }

        fn possible_moves(&self) -> Vec<ValidMove<Self>> {
            let mut moves = vec![];
            let color = self.to_act();

            if self.has_winner() {
                return moves;
            }

            for j in 0..WIDTH {
                if let Slot::Empty = self.state.get(HEIGHT - 1, j) {
                    moves.push(ValidMove {
                        valid_move: (j, color),
                        valid_for: self.clone(),
                    })
                }
            }
            moves
        }


        fn apply(&mut self, m: Self::Move) {
            let (n, color) = m;
            for i in 0..HEIGHT {
                match self.state.board[i][n] {
                    Full(_) => (),
                    Empty => {
                        self.state.set(i, n, color);
                        if self.has_won(&color) {
                            self.winner = Some(color);
                        }
                        self.to_act = self.to_act.flip();
                        return;
                    }
                }
            }
            panic!("This shouldn't happen for validated moves.");
        }

        fn has_won(&self, &color: &Color) -> bool {
            // horizontalCheck
            let color = Slot::Full(color);
            let get = |i, j| self.state.get(j, i);

            for j in 0..HEIGHT - (NEEDED - 1) {
                for i in 0..WIDTH {
                    if get(i, j) == color && get(i, j + 1) == color && get(i, j + 2) == color &&
                       get(i, j + 3) == color {
                        return true;
                    }
                }
            }
            // verticalCheck
            for i in 0..WIDTH - (NEEDED - 1) {
                for j in 0..HEIGHT {
                    if get(i, j) == color && get(i + 1, j) == color && get(i + 2, j) == color &&
                       get(i + 3, j) == color {
                        return true;
                    }
                }
            }

            // ascending diagonal check
            for i in 3..WIDTH {
                for j in 0..HEIGHT - (NEEDED - 1) {
                    if get(i, j) == color && get(i - 1, j + 1) == color &&
                       get(i - 2, j + 2) == color &&
                       get(i - 3, j + 3) == color {
                        return true;
                    }
                }
            }
            // // descendingDiagonalCheck
            for i in 3..WIDTH {
                for j in 3..HEIGHT {
                    if get(i, j) == color && get(i - 1, j - 1) == color &&
                       get(i - 2, j - 2) == color &&
                       get(i - 3, j - 3) == color {
                        return true;
                    }
                }
            }
            false
        }
    }
}
