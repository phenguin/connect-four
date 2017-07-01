#![feature(plugin)]
#![plugin(clippy)]

use std::fmt;
use std::mem::transmute;
use std::convert::From;
use std::marker::PhantomData;

#[derive(Clone, Copy)]
enum Color {
    R,
    B,
}

#[derive(Clone, Copy)]
enum Index {
    One,
    Two,
    Three,
    Four,
}

impl Index {
    fn from_usize(n: usize) -> Option<Self> {
        use Index::*;
        match n {
            0 => Some(One),
            1 => Some(Two),
            2 => Some(Three),
            3 => Some(Four),
            _ => None,
        }
    }
}

impl From<Index> for usize {
    fn from(i: Index) -> Self {
        use Index::*;
        match i {
            One => 0,
            Two => 1,
            Three => 2,
            Four => 3,
        }
    }
}

#[derive(Clone, Copy)]
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
                Color::R => "R",
                Color::B => "B",
            }
        )
    }
}

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Slot::*;
        match *self {
            Empty => write!(f, " "),
            Full(c) => write!(f, "{}", c),
        }
    }
}

struct Board<T> {
    board: [[Slot; 4]; 4],
    _phantom: PhantomData<T>,
}
impl<T> std::clone::Clone for Board<T> {
    fn clone(&self) -> Board<T> {
        Board {
            board: self.board.clone(),
            _phantom: PhantomData,
        }
    }
}

struct Dirty {}
struct Clean {}

struct ValidMove {
    index: Index,
    color: Color,
    valid_for: Board<Dirty>,
}

impl ValidMove {
    pub fn board(&self) -> &Board<Dirty> {
        &self.valid_for
    }

    pub fn index(&self) -> Index {
        self.index
    }

    pub fn color(&self) -> Color {
        self.color
    }

    fn apply(self) -> (Board<Clean>, Index) {
        use Slot::*;
        let mut res = self.valid_for;
        let n: usize = usize::from(self.index);
        for i in 0..4 {
            match res.board[i][n] {
                Full(_) => (),
                Empty => {
                    res.board[i][n] = Full(self.color);
                    return (res.cleaned(), Index::from_usize(i).unwrap());
                }
            }
        }
        panic!("This shouldn't happen for validated moves.");
    }
}

impl Board<Clean> {
    fn new() -> Self {
        // board[0] is the bottom row, board[3] is the top.
        Board {
            board: [[Slot::new(); 4]; 4],
            _phantom: PhantomData,
        }
    }

    fn verify_move(self: Board<Clean>, col: Index, color: Color) -> Option<ValidMove> {
        // TODO : Is this fine?
        if let Slot::Full(_) = self.board[3][usize::from(col)] {
            return None;
        }

        Some(ValidMove {
            index: col,
            color: color,
            valid_for: self.dirtied(),
        })
    }

    fn dirtied(self) -> Board<Dirty> {
        unsafe { transmute(self) }
    }

    fn try_move(&mut self, n: Index, c: Color) -> Option<Index> {
        let x = self.clone().verify_move(n, c);
        x.map(move |valid_move| {
            let (new_board, row) = valid_move.apply();
            *self = new_board;
            row
        })
    }

    fn try_moves<I>(&mut self, moves: I) -> Vec<Option<Index>>
    where
        I: Iterator<Item = (Index, Color)>,
    {
        moves
            .map(move |(col, color)| self.try_move(col, color))
            .collect()
    }
}

impl Board<Dirty> {
    fn cleaned(self) -> Board<Clean> {
        unsafe { transmute(self) }
    }
}

impl<T> fmt::Display for Board<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "|----|")?;
        for row in self.board.iter().rev() {
            writeln!(f, "|{}{}{}{}|", row[0], row[1], row[2], row[3])?;
        }
        write!(f, "|----|")
    }
}

fn main() {
    use Color::*;
    use Index::*;
    let test_moves = vec![(One, R), (One, B), (Three, R), (Two, B)];
    let board = Board::new();
    // println!("{}", board);
    println!("{}", board);
    let mut board = board;
    board.try_moves(test_moves.into_iter());
    println!("{}", board);
    // println!("{}", board.transposed());
}
