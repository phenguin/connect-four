use self::DotsPlayer::*;
use rand;
use std::fmt;
use super::*;

use std::collections::HashSet;
use std::hash::Hash;
use std::hash::Hasher;
use std::cmp::{Ord, Ordering};


const HEIGHT: usize = 3;
const WIDTH: usize = 3;

pub fn test_board() -> DotsBoard {
    DotsBoard {
        horizontals: [[true; WIDTH - 1]; HEIGHT],
        verticals: [[true; WIDTH]; HEIGHT - 1],
        owners: [[Some(A); WIDTH - 1]; HEIGHT - 1],
    }
}


// DotsPlayer
#[derive(Clone, Copy, PartialOrd, PartialEq, Hash, Debug, Ord, Eq)]
pub enum DotsPlayer {
    A,
    B,
}
impl rand::Rand for DotsPlayer {
    fn rand<R: rand::Rng>(rng: &mut R) -> Self {
        if rng.gen() {
            return A;
        }
        B
    }
}

impl DotsPlayer {
    fn flip(&self) -> Self {
        match *self {
            A => B,
            B => A,
        }
    }
}

impl fmt::Display for DotsPlayer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                A => "A",
                B => "B",
            }
        )
    }
}


#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub struct DotsBoard {
    horizontals: [[bool; WIDTH - 1]; HEIGHT],
    verticals: [[bool; WIDTH]; HEIGHT - 1],
    owners: [[Option<DotsPlayer>; WIDTH - 1]; HEIGHT - 1],
}

impl DotsBoard {
    fn draw_hrow(&self, f: &mut fmt::Formatter, j: usize) -> fmt::Result {
        assert!(j < HEIGHT);
        for i in 0..WIDTH - 1 {
            let c = if self.horizontals[j][i] { "---" } else { "   " };
            write!(f, "+{}", c)?;
        }
        writeln!(f, "+")
    }
    fn draw_vrow(&self, f: &mut fmt::Formatter, j: usize) -> fmt::Result {
        assert!(j < HEIGHT - 1);
        for i in 0..WIDTH - 1 {
            let c = if self.verticals[j][i] { "|" } else { " " };
            write!(f, "{}", c)?;
            write!(f, " ")?;
            if let Some(owner) = self.owners[j][i] {
                write!(f, "{} ", owner)?;
            } else {
                write!(f, "  ")?;
            }
        }
        let c = if self.verticals[j][WIDTH - 1] {
            "|"
        } else {
            " "
        };
        writeln!(f, "{}", c)
    }
}

impl RandGame for Dots {}

impl fmt::Display for DotsBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for j in 0..(HEIGHT - 1) {
            self.draw_hrow(f, j)?;
            self.draw_vrow(f, j)?;
        }
        self.draw_hrow(f, HEIGHT - 1)
    }
}

#[derive(Clone, Debug)]
pub struct Dots {
    board: DotsBoard,
    to_act: DotsPlayer,
    ref_player: DotsPlayer,
    winner: Option<DotsPlayer>,
    scores: (usize, usize),
    possible_moves: HashSet<DotsMove>,
}

impl Eq for Dots {}
impl PartialEq for Dots {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board && self.to_act == other.to_act
    }
}

impl fmt::Display for Dots {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Acting: {}, Scores: {:?}", self.to_act(), self.scores)?;
        writeln!(f, "{}", self.board)?;
        let mut moves: Vec<_> = self.possible_moves.iter().collect();
        moves.sort();
        // for m in &moves {
        //     writeln!(f, "Move: {:?}", m)?;
        // }
        writeln!(f, "Winner: {:?}", self.winner)

    }
}

impl Hash for Dots {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.board.hash(state);
        self.to_act.hash(state);
    }
}

#[derive(Hash, Clone, Debug, PartialEq, Eq, Copy, PartialOrd, Ord)]
pub enum DotsMove {
    H(usize, usize),
    V(usize, usize),
}

impl DotsMove {
    fn up(&self) -> Self {
        match *self {
            V(j, i) => V(j - 1, i),
            H(j, i) => H(j - 1, i),
        }
    }
    fn left(&self) -> Self {
        match *self {
            V(j, i) => V(j, i - 1),
            H(j, i) => H(j, i - 1),
        }
    }
    fn coords(&self) -> (usize, usize) {
        match *self {
            V(j, i) | H(j, i) => (j, i),
        }
    }
}

use self::DotsMove::*;

impl ParseGame for Dots {
    fn parse_move(&self, s: &str) -> Option<DotsMove> {
        let pieces: Vec<&str> = s.split(' ').collect();

        let mj = pieces[1].parse::<usize>().ok();
        let mi = pieces[2].parse::<usize>().ok();
        let pair = mj.and_then(|j| mi.map(|i| (j, i)));
        pair.and_then(|(j, i)| match pieces[0] {
            "H" | "h" => Some(H(j, i)),
            "V" | "v" => Some(V(j, i)),
            _ => None,
        })
    }
}

impl Game for Dots {
    type Agent = DotsPlayer;
    type Move = DotsMove;
    fn to_act(&self) -> Self::Agent {
        self.to_act
    }

    fn move_valid(&self, m: &Self::Move) -> bool {
        self.possible_moves.contains(m)
    }

    fn has_won(&self, &agent: &Self::Agent) -> bool {
        if let Some(winner) = self.winner {
            if winner == agent {
                return true;
            }
        }
        false
    }

    fn apply(&mut self, m: Self::Move) {
        let acting = self.to_act();

        match m {
            H(j, i) => {
                (*self).board.horizontals[j][i] = true;
            }
            V(j, i) => {

                (*self).board.verticals[j][i] = true;
            }
        }

        self.possible_moves.remove(&m);

        let complete = self.completes(m);
        let no_completions = complete.is_empty();
        for (j, i) in complete {
            (*self).board.owners[j][i] = Some(acting);
            (*self).inc_score(acting);
        }

        if self.possible_moves().is_empty() {
            let other = acting.flip();
            match self.get_score(acting).cmp(&self.get_score(other)) {
                Ordering::Greater => (*self).winner = Some(acting),
                Ordering::Less => (*self).winner = Some(other),
                Ordering::Equal => (),
            }
        }

        if no_completions {
            (*self).to_act = acting.flip()
        }

    }

    fn player_weight(&self, &a: &Self::Agent) -> Score {
        if self.ref_player() == a { 1 } else { -1 }
    }

    fn winner(&self) -> Option<Self::Agent> {
        self.winner
    }

    fn agent_id(&self, &agent: &Self::Agent) -> u32 {
        match agent {
            DotsPlayer::A => 0,
            DotsPlayer::B => 1,
        }
    }

    fn ref_player(&self) -> Self::Agent {
        self.ref_player
    }
    fn new(&start: &Self::Agent) -> Self {
        let mut moves = HashSet::new();

        for j in 0..HEIGHT {
            for i in 0..WIDTH - 1 {
                moves.insert(H(j, i));
            }
        }

        for j in 0..HEIGHT - 1 {
            for i in 0..WIDTH {
                moves.insert(V(j, i));
            }
        }

        Self {
            board: DotsBoard {
                horizontals: [[false; WIDTH - 1]; HEIGHT],
                verticals: [[false; WIDTH]; HEIGHT - 1],
                owners: [[None; WIDTH - 1]; HEIGHT - 1],
            },
            to_act: start,
            ref_player: start,
            winner: None,
            scores: (0, 0),
            possible_moves: moves,
        }


    }

    fn possible_moves(&self) -> Vec<ValidMove<Self>> {
        self.possible_moves
            .iter()
            .map(|&m| {
                let this = self.clone();
                ValidMove {
                    valid_move: m,
                    valid_for: this,
                }
            })
            .collect()
    }
}

fn box_bounds((j, i): (usize, usize), v: &mut Vec<DotsMove>) {
    v.push(V(j, i)); // L
    v.push(H(j, i)); // T
    v.push(V(j, i + 1)); // R
    v.push(H(j + 1, i)); // B
}

impl Dots {
    fn completes(&self, m: DotsMove) -> Vec<(usize, usize)> {
        // TODO Temp
        let mut res = Vec::new();
        let mut conds = Vec::new();
        match m {
            H(0, _) | V(_, 0) => {
                let mut v = Vec::new();
                box_bounds(m.coords(), &mut v);
                conds.push((v, m.coords()));
            }
            H(j, _) => {
                if j == HEIGHT - 1 {
                    let mut v = Vec::new();
                    box_bounds(m.up().coords(), &mut v);
                    conds.push((v, m.up().coords()));
                } else {
                    for b in &[m.coords(), m.up().coords()] {
                        let mut v = Vec::new();
                        box_bounds(*b, &mut v);
                        conds.push((v, *b));
                    }
                }
            }
            V(_, i) => {
                if i == WIDTH - 1 {
                    let mut v = Vec::new();
                    box_bounds(m.left().coords(), &mut v);
                    conds.push((v, m.left().coords()));
                } else {
                    for b in &[m.coords(), m.left().coords()] {
                        let mut v = Vec::new();
                        box_bounds(*b, &mut v);
                        conds.push((v, *b));
                    }
                }
            }
        };

        for (checks, square) in conds {
            let got_square = checks.into_iter().all(|mv| {
                let valid = !self.move_valid(&mv);
                // println!("square ({:?}) -- {:?}: {:?}", square, mv, valid);
                valid
            });
            if got_square {
                res.push(square);
            }
        }
        res

    }
    fn get_score(&self, a: DotsPlayer) -> usize {
        match a {
            DotsPlayer::A => self.scores.0,
            DotsPlayer::B => self.scores.1,
        }
    }
    fn inc_score(&mut self, a: DotsPlayer) {
        let it = match a {
            DotsPlayer::A => &mut self.scores.0,
            DotsPlayer::B => &mut self.scores.1,
        };
        *it += 1;
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_draw_hrow() {

        println!("{}", test_board());
    }
}
