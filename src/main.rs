use std::fmt;

#[derive(Clone, Copy)]
enum Color {
    Red,
    Black,
}

#[derive(Clone, Copy)]
enum Slot {
    Empty,
    Full(Color),
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Color::Red => "R",
                Color::Black => "B",
            }
        )
    }
}

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Slot::*;
        use Color::*;
        match *self {
            Empty => write!(f, " "),
            Full(c) => write!(f, "{}", c),
        }
    }
}

struct Board {
    board: [[Slot; 4]; 4],
}
fn main() {}
