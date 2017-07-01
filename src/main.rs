#![feature(plugin)]
#![plugin(clippy)]
#![allow(dead_code)]

use std::fmt;
use std::mem::transmute;
use std::marker::PhantomData;

const HEIGHT: usize = 6;
const WIDTH: usize = 7;
const NEEDED: usize = 4;

#[derive(Clone, Copy, PartialEq)]
enum Color {
    R,
    B,
}

impl Color {
    fn flip(&self) -> Color {
        use Color::*;
        match *self {
            R => B,
            B => R,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
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
            board: self.board,
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

    fn winner(&self) -> Option<Color> {
        if self.has_won(Color::R) {
            Some(Color::R)
        } else if self.has_won(Color::B) {
            Some(Color::B)
        } else {
            None
        }
    }

    fn possible_moves_for_color(&self, color: Color) -> Vec<ValidMove> {
        let mut moves = vec![];
        for j in 0..WIDTH {
            if let Slot::Empty = self.get(HEIGHT - 1, j) {
                moves.push(ValidMove {
                    index: j,
                    color: color,
                    valid_for: self.clone().dirtied(),
                })
            }
        }
        moves
    }

    fn has_won(&self, color: Color) -> bool {
        // horizontalCheck
        let color = Slot::Full(color);

        for j in 0..HEIGHT - NEEDED {
            for i in 0..WIDTH {
                if self.get(i, j) == color && self.get(i, j + 1) == color &&
                    self.get(i, j + 2) == color && self.get(i, j + 3) == color
                {
                    return true;
                }
            }
        }
        // verticalCheck
        for i in 0..WIDTH - (NEEDED - 1) {
            for j in 0..HEIGHT {
                if self.get(i, j) == color && self.get(i + 1, j) == color &&
                    self.get(i + 2, j) == color && self.get(i + 3, j) == color
                {
                    return true;
                }
            }
        }

        // ascending diagonal check
        for i in 3..WIDTH {
            for j in 0..HEIGHT - (NEEDED - 1) {
                if self.get(i, j) == color && self.get(i - 1, j + 1) == color &&
                    self.get(i - 2, j + 2) == color &&
                    self.get(i - 3, j + 3) == color
                {
                    return true;
                }
            }
        }
        // // descendingDiagonalCheck
        for i in 3..WIDTH {
            for j in 3..HEIGHT {
                if self.get(i, j) == color && self.get(i - 1, j - 1) == color &&
                    self.get(i - 2, j - 2) == color &&
                    self.get(i - 3, j - 3) == color
                {
                    return true;
                }
            }
        }
        false
    }


    fn verify_move(self: Board<Clean>, col: usize, color: Color) -> Option<ValidMove> {
        // TODO : Is this fine?
        if let Slot::Full(_) = self.board[HEIGHT - 1][col] {
            return None;
        }

        if col >= WIDTH {
            return None;
        }

        if self.winner().is_some() {
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

impl<T> Board<T> {
    fn get(&self, i: usize, j: usize) -> Slot {
        self.board[i][j]
    }

    fn set(&mut self, i: usize, j: usize, c: Color) {
        (*self).board[i][j] = Slot::Full(c);
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
            writeln!(f, "|")?;
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
