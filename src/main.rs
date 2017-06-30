use std::fmt;
use std::convert::From;

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

struct Board {
    board: [[Slot; 4]; 4],
}

impl Board {
    fn new() -> Self {
        // board[0] is the bottom row, board[3] is the top.
        Board { board: [[Slot::new(); 4]; 4] }
    }

    fn try_move(&mut self, n: Index, c: Color) -> Option<Index> {
        use Slot::*;
        let n: usize = usize::from(n);
        for i in 0..4 {
            match self.board[i][n] {
                Full(_) => (),
                Empty => {
                    self.board[i][n] = Full(c);
                    return Some(Index::from_usize(i).unwrap());
                }
            }
        }
        return None;
    }

    fn try_moves<T>(&mut self, moves: T) -> Vec<Option<Index>>
    where
        T: Iterator<Item = (Index, Color)>,
    {
        moves
            .map(move |(col, color)| self.try_move(col, color))
            .collect()
    }

    fn transposed(&self) -> Self {
        let mut board = Board::new();
        for i in 0..4 {
            for j in 0..4 {
                board.board[i][j] = self.board[j][i];
            }
        }
        board
    }
}

impl fmt::Display for Board {
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
    use Slot::*;
    use Index::*;
    let test_moves = vec![(One, R), (One, B), (Three, R), (Two, B)];
    let board = Board { board: [[Full(R); 4], [Empty; 4], [Empty; 4], [Empty; 4]] };
    // println!("{}", board);
    println!("{}", board);
    let mut board = board;
    board.try_moves(test_moves.into_iter());
    println!("{}", board);
    // println!("{}", board.transposed());
}
