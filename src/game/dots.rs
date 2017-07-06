use self::DotsPlayer::*;
use rand;
use std::fmt;
use super::*;
use std::str::FromStr;


const HEIGHT: usize = 20;
const WIDTH: usize = 20;

pub fn test_board() -> DotsBoard {
    DotsBoard {
        horizontals: [[true; WIDTH - 1]; HEIGHT],
        verticals: [[true; WIDTH]; HEIGHT - 1],
        owners: [[Some(X); WIDTH - 1]; HEIGHT - 1],
    }
}

// DotsPlayer
#[derive(Clone, Copy, PartialOrd, PartialEq, Hash, Debug, Ord, Eq)]
pub enum DotsPlayer {
    X,
    O,
}
impl rand::Rand for DotsPlayer {
    fn rand<R: rand::Rng>(rng: &mut R) -> Self {
        if rng.gen() {
            return X;
        }
        O
    }
}

impl DotsPlayer {
    fn flip(&self) -> Self {
        match *self {
            O => X,
            X => O,
        }
    }
}

impl fmt::Display for DotsPlayer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                X => "X",
                O => "O",
            }
        )
    }
}

// DotsGame

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

impl fmt::Display for DotsBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for j in 0..(HEIGHT - 1) {
            self.draw_hrow(f, j)?;
            self.draw_vrow(f, j)?;
        }
        self.draw_hrow(f, HEIGHT - 1)
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
