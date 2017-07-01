#![feature(plugin)]
#![plugin(clippy)]

use std::fmt;
use std::mem::transmute;
use std::convert::From;
use std::marker::PhantomData;

const HEIGHT: usize = 4;
const WIDTH: usize = 7;

#[derive(Clone, Copy)]
enum Color {
    R,
    B,
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
    board: [[Slot; WIDTH]; HEIGHT],
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
    index: usize,
    color: Color,
    valid_for: Board<Dirty>,
}

impl ValidMove {
    pub fn board(&self) -> &Board<Dirty> {
        &self.valid_for
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn color(&self) -> Color {
        self.color
    }

    fn apply(self) -> (Board<Clean>, usize) {
        use Slot::*;
        let mut res = self.valid_for;
        let n = self.index;
        for i in 0..HEIGHT {
            match res.board[i][n] {
                Full(_) => (),
                Empty => {
                    res.board[i][n] = Full(self.color);
                    return (res.cleaned(), i);
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
            board: [[Slot::new(); WIDTH]; HEIGHT],
            _phantom: PhantomData,
        }
    }

    fn verify_move(self: Board<Clean>, col: usize, color: Color) -> Option<ValidMove> {
        // TODO : Is this fine?
        if let Slot::Full(_) = self.board[3][usize::from(col)] {
            return None;
        }

        if col >= WIDTH {
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

    fn try_move(&mut self, n: usize, c: Color) -> Option<usize> {
        let x = self.clone().verify_move(n, c);
        x.map(move |valid_move| {
            let (new_board, row) = valid_move.apply();
            *self = new_board;
            row
        })
    }

    fn try_moves<I>(&mut self, moves: I) -> Vec<Option<usize>>
    where
        I: Iterator<Item = (usize, Color)>,
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
        let dashes: String = (0..WIDTH).map(|_| "-").collect();
        writeln!(f, "|{}|", dashes.as_str())?;
        for row in self.board.iter().rev() {
            write!(f, "|")?;
            for slot in row.iter() {
                write!(f, "{}", slot)?;
            }
            writeln!(f, "|");
        }
        write!(f, "|{}|", dashes.as_str())
    }
}

fn main() {
    use Color::*;
    let test_moves = vec![(0, R), (0, B), (2, R), (1, B)];
    let board = Board::new();
    // println!("{}", board);
    println!("{}", board);
    let mut board = board;
    board.try_moves(test_moves.into_iter());
    println!("{}", board);
    // println!("{}", board.transposed());
}
