#![feature(plugin)]
#![feature(conservative_impl_trait)]
#![feature(use_extern_macros)]
#![plugin(clippy)]
#![allow(dead_code)]
extern crate rand;
extern crate rayon;

use std::io;
use std::cmp::max;
use rand::Rng;
use std::hash::{Hash,Hasher};
use std::collections::hash_map::DefaultHasher;
use rand::XorShiftRng;
use std::fmt;
use std::mem::transmute;
use std::marker::PhantomData;
use rayon::prelude::*;

const HEIGHT: usize = 6;
const WIDTH: usize = 7;
const NEEDED: usize = 4;
const SEARCH_DEPTH: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
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

    fn show(&self) -> &str {
        use Color::*;
        match *self {
            R => "Red",
            B => "Black",
        }
    }

    fn random() -> Color {
        use Color::*;
        if rand::random() {
            return R;
        }
        B
    }
}

#[derive(Clone, Copy, PartialEq, Hash)]
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

#[derive(Hash)]
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

#[derive(Hash)]
struct Dirty {}
#[derive(Hash)]
struct Clean {}

#[derive(Hash,Clone)]
struct ValidMove {
    index: usize,
    color: Color,
    valid_for: Board<Dirty>,
}

struct ValidMoveMut<'gs> {
    index: usize,
    color: Color,
    valid_for: &'gs mut Board<Clean>,
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

impl <'gs> ValidMoveMut<'gs> {
    fn apply(self) -> (usize) {
        use Slot::*;
        let b = self.valid_for;
        let n = self.index;
        for i in 0..HEIGHT {
            match b.board[i][n] {
                Full(_) => (),
                Empty => {
                    (*b).board[i][n] = Full(self.color);
                    return i;
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

    fn has_winner(&self) -> bool {
        self.winner().is_some()
    }

    fn possible_moves(&self, color: Color) -> Vec<ValidMove> {
        let mut moves = vec![];

        if self.winner().is_some() {
            return moves;
        }

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
        let get = |i, j| self.get(j, i);

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

    fn verify_move_mut(&mut self, col: usize, color: Color) -> Option<ValidMoveMut> {
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

        Some(ValidMoveMut {
            index: col,
            color: color,
            valid_for: self,
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

    fn try_move_mut(&mut self, n: usize, c: Color) -> Option<usize> {
        self.verify_move_mut(n, c).map(|m| {
            m.apply()
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

    fn minimax(&self, to_act: Color) -> (i32, usize) {
        let node = Node {
            game: Game {
                state: self.clone(),
                to_act: to_act,
                ref_color: to_act,
            },
            preceding_move: None,
        };
        node.negamax_ab(0, i32::min_value(), i32::max_value())
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
        for row in self.board.iter().rev() {
            write!(f, "|")?;
            for slot in row.iter() {
                write!(f, "{}", slot)?;
            }
            writeln!(f, "|")?;
        }
        writeln!(f, "|{}|", dashes.as_str())?;
        write!(f, " ")?;
        for i in 0..WIDTH {
            write!(f, "{}", i + 1)?;
        }
        writeln!(f, " ")
    }
}

#[derive(Hash,Clone)]
struct Game {
    state: Board<Clean>,
    to_act: Color,
    ref_color: Color,
}

#[derive(Hash)]
struct Node {
    game: Game,
    preceding_move: Option<usize>,
}

impl Node {
    fn negamax(&self, depth: usize) -> (i32, usize) {
        let game = &self.game;


        let nexts = game.possibilities();
        if depth > SEARCH_DEPTH || nexts.is_empty() {
            assert!(!self.preceding_move.is_none());
            return (
                game.eval() * game.color_weight(),
                self.preceding_move.unwrap(),
            );
        }

        nexts
            .par_iter()
            .map(|node| {
                let (s, m) = node.negamax(depth + 1);
                (-s, m)
            })
            .max()
            .unwrap()


    }
    fn negamax_ab(&self, depth: usize, alpha: i32, beta: i32) -> (i32, usize) {
        let game = &self.game;


        let nexts = game.possibilities();
        if depth > SEARCH_DEPTH || nexts.is_empty() {
            assert!(!self.preceding_move.is_none());
            return (
                game.eval() * game.color_weight(),
                self.preceding_move.unwrap(),
            );
        }

        let mut best = i32::min_value();
        let mut best_move = None;
        let mut alpha = alpha;
        for node in nexts {
            let (s, m) = node.negamax_ab(depth + 1, -beta, -alpha);
            let s = -s;
            if s >= best {
                best = s;
                best_move = Some(m);
            }

            alpha = max(alpha, best);
            if alpha >= beta {
                break
            }
        }

        (best, best_move.unwrap())
    }
}

fn calculate_hash<T: Hash>(t: &T) -> [u32; 4] {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    let x = s.finish();
    let low: u32 = x as u32;
    let high: u32 = (x >> 32) as u32;
    [low, high, low, high]
        
}

impl Game {
    fn eval(&self) -> i32 {
        if let Some(c) = self.state.winner() {
            if c == self.ref_color {
                return i32::max_value();
            } else {
                return i32::min_value();
            }
        } else {
            self.monte_carlo(100) as i32
        }
    }

    fn random_outcome(&self) -> Option<Color> {
        // let seed = calculate_hash(self);
        let seed = rand::random::<[u32; 4]>();
        let mut rng : XorShiftRng = rand::SeedableRng::from_seed(seed);
        
        let mut game = (*self).clone();
        while !game.state.has_winner() {
            let moves : Vec<usize> = game.state.possible_moves(game.to_act).into_iter().map(|m| m.index()).collect();
            let next_move : Option<&usize> = rng.choose(moves.as_ref());
            if next_move.is_none() { return None };
            let index = *next_move.unwrap();
            game.state.try_move_mut(index, game.to_act);
            game.to_act = game.to_act.flip();
        }
       game.state.winner()
    }

    fn monte_carlo(&self, trials: u32)  -> u32 {
        (0..trials).flat_map(|_| {
            self.random_outcome()
        }).filter(|c| *c == self.ref_color).map(|_| 1).sum()
    }

    fn color_weight(&self) -> i32 {
        if self.to_act == self.ref_color { 1 } else { -1 }
    }


    fn ref_color_to_act(&self) -> bool {
        self.ref_color == self.to_act
    }

    fn possibilities(&self) -> Vec<Node> {
        let moves = self.state.possible_moves(self.to_act);
        moves
            .into_iter()
            .map(move |m| {
                Node {
                    preceding_move: Some(m.index()),
                    game: Game {
                        state: m.apply().0,
                        to_act: self.to_act.flip(),
                        ref_color: self.ref_color,
                    },
                }
            })
            .collect()
    }
}

fn test() {
    let board = Board::new();
    // println!("{}", board);
    let mut board = board;
    let mut color = Color::random();
    let (s, m) = board.minimax(color);
    println!("move: {}, score: {}", m, s);
    for _ in 1..20 {
        if board.winner().is_some() {
            break;
        };
        board.try_move(rand::random::<usize>() % WIDTH, color);
        color = color.flip();
    }
    if let Some(winner) = board.winner() {
        println!("Winner: {}", winner);
    }

    println!("{}", board);
    let (s, m) = board.minimax(color);
    println!("move: {}, score: {}", m, s);
    // println!("{}", board.transposed());
}

fn get_human_move(board: &Board<Clean>, color: Color) -> usize {
    println!("{}'s move! Here is the board:\n", color.show());
    println!("{}", board);

    loop {
        println!("What is your move?");
        println!("Enter a number between {} and {}", 1, WIDTH);
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).expect(
            "Failed to read line... something is mad broke.",
        );

        let choice: usize = match choice.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        if !(1 <= choice && choice <= WIDTH) {
            println!("Try a better number...");
            continue;
        }

        if board.clone().verify_move(choice - 1, color).is_none() {
            println!("Invalid move..");
            continue;
        }

        return choice - 1;
    }
}

fn get_computer_move(board: &Board<Clean>, color: Color) -> usize {
    println!("Computer is thinking.....");
    let (s, m) = board.minimax(color);
    println!("score: {}", s);
    m
}

fn run_game() {
    use Color::*;
    println!("Connect four time..");
    println!("You are red, and the computer is black..");
    println!("Flipping to see who starts...");

    let mut to_act = Color::random();
    println!(
        "{} goes first!",
        to_act.show()
    );

    let mut board = Board::new();
    while !board.has_winner() {
        let cloned_board = board.clone();
        match to_act {
            R => {
                let hmove = get_human_move(&cloned_board, R);
                board.try_move(hmove, R);
            }
            B => {
                board.try_move(get_computer_move(&cloned_board, B), B);
            }
        }
        to_act = to_act.flip();
    }

    let winner = board.winner().unwrap();
    println!("Winner is: {}", winner.show());
}

fn main() {
    run_game();
}
