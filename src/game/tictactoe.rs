use super::*;
use rand;
use std::default::Default;
use std::fmt;
pub const SIZE: usize = 3;
pub const REQ: usize = 3;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Marker {
    X,
    O,
}

impl Marker {
    fn flip(&self) -> Marker {
        match *self {
            Marker::X => Marker::O,
            Marker::O => Marker::X,
        }
    }
}

impl rand::Rand for Marker {
    fn rand<R: rand::Rng>(rng: &mut R) -> Marker {
        if rng.gen() {
            return Marker::X;
        }
        Marker::O
    }
}

impl fmt::Display for Marker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Marker::X => "X",
                Marker::O => "O",
            }
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Square(Option<Marker>);

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => write!(f, " "),
            Some(x) => write!(f, "{}", x),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Board {
    board: [Square; SIZE * SIZE],
}

impl Board {
    pub fn get(&self, i: usize, j: usize) -> Square {
        self.board[i * SIZE + j]
    }

    pub fn set(&mut self, i: usize, j: usize, x: Marker) {
        (*self).board[i * SIZE + j] = Square(Some(x));
    }
}

impl Default for Board {
    fn default() -> Self {
        Board { board: [Square(None); SIZE * SIZE] }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dashes: String = (0..SIZE * 3).map(|_| "-").collect();
        // for row in self.board.iter().rev() {
        writeln!(f, "|{}|", dashes.as_str())?;
        for i in 0..SIZE {
            write!(f, "|")?;
            for j in 0..SIZE {
                write!(f, " {} ", self.get(i, j))?;
            }
            writeln!(f, "|")?;
        }
        writeln!(f, "|{}|", dashes.as_str())?;
        write!(f, " ")?;
        for i in 0..SIZE {
            write!(f, " {} ", i + 1)?;
        }
        writeln!(f, " ")
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct TicTacToe {
    state: Board,
    to_act: Marker,
    ref_player: Marker,
    winner: Option<Marker>,
}

impl fmt::Display for TicTacToe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Acting: {}, Ref: {}", self.to_act, self.ref_player)?;
        writeln!(f, "{}", self.state)
    }
}

impl Game for TicTacToe {
    type Move = (usize, usize, Self::Agent);
    type Agent = Marker;

    fn agent_id(&self, &a: &Self::Agent) -> u32 {
        match a {
            Marker::X => 0,
            Marker::O => 1,
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
        !self.state.get(m.0, m.1).0.is_some()
    }

    fn new(&start: &Self::Agent) -> Self {
        let board = Board::default();
        TicTacToe {
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
        let marker = self.to_act();
        let mut moves = Vec::new();
        for i in 0..SIZE {
            for j in 0..SIZE {
                if !self.state.get(i, j).0.is_some() {
                    moves.push(ValidMove {
                        valid_move: (i, j, marker),
                        valid_for: self.clone(),
                    });
                }
            }
        }
        moves
    }


    fn apply(&mut self, m: Self::Move) {
        let (i, j, marker) = m;
        match self.state.get(i, j) {
            Square(Some(_)) => (),
            Square(None) => {
                self.state.set(i, j, marker);
                if self.has_won(&marker) {
                    self.winner = Some(marker);
                }
                self.to_act = self.to_act.flip();
                return;
            }
        }
        panic!("This shouldn't happen for validated moves.");
    }

    fn has_won(&self, &marker: &Marker) -> bool {
        // horizontalCheck
        let value = |i, j, m| {
            if i < SIZE && j < SIZE{
                if self.state.get(i,j) == m {
                    return 1;
                }
            }
            0
        };
        let marker = Square(Some(marker));
        let get = |i, j| self.state.get(j, i);
        for i in 0..SIZE {
            for j in 0..SIZE {
                if (0..REQ).map(|n| value(i, j + n, marker)).sum::<usize>() == REQ {
                    return true;
                }
                if (0..REQ).map(|n| value(i + n, j, marker)).sum::<usize>() == REQ {
                    return true;
                }
                if (0..REQ).map(|n| value(i + n, j + n, marker)).sum::<usize>() == REQ {
                    return true;
                }
                if (0..REQ).map(|n| value(i + n, j - n, marker)).sum::<usize>() == REQ {
                    return true;
                }
            }
        }
        false
    }
}

impl ParseGame for TicTacToe {
    fn parse_move(&self, input: &str) -> Option<Self::Move> {
        let mut words = input.split_whitespace();
        let mut it = Vec::new();
        for _ in 0..2 {
            it.push(
                words
                    .next()
                    .expect("invalid input.")
                    .parse::<usize>()
                    .expect("invalid input"),
            );
        }
        Some((it[0], it[1], self.to_act()))

    }
}

impl RandGame for TicTacToe {
    fn random_move<R: rand::Rng>(&mut self, rng: &mut R) -> Option<ValidMoveMut<Self>> {
        let mut moves = self.possible_moves();
        if moves.len() == 0 {
            None
        } else {
            rng.shuffle(&mut moves);
            Some(ValidMoveMut {
                valid_move: moves[0].valid_move,
                valid_for: self,
            })
        }
    }
}
