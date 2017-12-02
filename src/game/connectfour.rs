use self::Color::*;
use self::Slot::*;
use rand;
use std::fmt;
use std::clone::Clone;
use super::*;
use std::str::FromStr;

const HEIGHT: usize = 6;
const WIDTH: usize = 7;
const NEEDED: usize = 4;


#[derive(Serialize, Deserialize, Clone, Copy, PartialOrd, PartialEq, Hash, Debug, Ord, Eq)]
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

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
        write!(
            f,
            "{}",
            match *self {
                Color::R => "X",
                Color::B => "@",
            }
        )
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

#[derive(Serialize, Deserialize, Hash, Debug, PartialEq, Eq)]
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

    fn num_pieces(&self) -> usize {
        self.board
            .iter()
            .flat_map(|row| row.iter())
            .map(|s| match s {
                &Empty => 0,
                _ => 1,
            })
            .sum()
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

#[derive(Serialize, Deserialize, Hash, Clone, Debug, PartialEq, Eq)]
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
        usize::from_str(input).ok().map(|n| (n - 1, self.to_act()))
    }
}

impl RandGame for ConnectFour {
    fn random_move<R: rand::Rng>(&mut self, rng: &mut R) -> Option<ValidMoveMut<Self>> {
        let mut is = [0; WIDTH];
        for (i, r) in is.iter_mut().enumerate() {
            *r = i;
        }
        rng.shuffle(&mut is);

        let to_act = self.to_act();
        for &i in &is {
            let m = (i, to_act);
            if self.move_valid(&m) {
                return Some(ValidMoveMut {
                    valid_move: m,
                    valid_for: self,
                });
            }
        }
        None
    }
}

impl Game for ConnectFour {
    type Move = (usize, Self::Agent);
    type Agent = Color;

    fn reachable(&self, g: &Self) -> bool {
        g.state.num_pieces() > self.state.num_pieces()
    }

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

        if col >= WIDTH || color != self.to_act {
            return false;
        }

        if let Slot::Full(_) = state.board[HEIGHT - 1][col] {
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
                    get(i, j + 3) == color
                {
                    return true;
                }
            }
        }
        // verticalCheck
        for i in 0..WIDTH - (NEEDED - 1) {
            for j in 0..HEIGHT {
                if get(i, j) == color && get(i + 1, j) == color && get(i + 2, j) == color &&
                    get(i + 3, j) == color
                {
                    return true;
                }
            }
        }

        // ascending diagonal check
        for i in 3..WIDTH {
            for j in 0..HEIGHT - (NEEDED - 1) {
                if get(i, j) == color && get(i - 1, j + 1) == color &&
                    get(i - 2, j + 2) == color && get(i - 3, j + 3) == color
                {
                    return true;
                }
            }
        }
        // // descendingDiagonalCheck
        for i in 3..WIDTH {
            for j in 3..HEIGHT {
                if get(i, j) == color && get(i - 1, j - 1) == color &&
                    get(i - 2, j - 2) == color && get(i - 3, j - 3) == color
                {
                    return true;
                }
            }
        }
        false
    }
}
