#![feature(plugin)]
#![feature(conservative_impl_trait)]
#![feature(use_extern_macros)]
#![plugin(clippy)]
#![allow(dead_code)]

extern crate rand;
extern crate rayon;
extern crate clap;
extern crate cpuprofiler;

use cpuprofiler::PROFILER;


use Color::*;
use Slot::*;
use clap::{Arg, App, value_t};
use rand::{Rng, XorShiftRng};
use rayon::prelude::*;
use std::cmp::max;
use std::default::Default;
use std::fmt;
use std::io;
use std::marker::PhantomData;
use std::mem::transmute;

const HEIGHT: usize = 6;
const WIDTH: usize = 7;
const NEEDED: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
enum Color {
    R,
    B,
}

impl Color {
    fn flip(&self) -> Color {
        match *self {
            R => B,
            B => R,
        }
    }

    fn show(&self) -> &str {
        match *self {
            R => "X",
            B => "@",
        }
    }

    fn random() -> Color {
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
        write!(f,
               "{}",
               match *self {
                   Color::R => "X",
                   Color::B => "@",
               })
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

#[derive(Hash)]
struct Board<T> {
    board: [[Slot; WIDTH]; HEIGHT],
    winner: Option<Color>,
    _phantom: PhantomData<T>,
}
impl<T> std::clone::Clone for Board<T> {
    fn clone(&self) -> Board<T> {
        Board {
            board: self.board,
            winner: self.winner,
            _phantom: PhantomData,
        }
    }
}

#[derive(Hash)]
struct Dirty {}
#[derive(Hash)]
struct Clean {}

#[derive(Hash, Clone)]
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
        let mut res = self.valid_for;
        let n = self.index;
        for i in 0..HEIGHT {
            match res.board[i][n] {
                Full(_) => (),
                Empty => {
                    res.board[i][n] = Full(self.color);
                    let mut res = res.cleaned();
                    if res.has_won(self.color) {
                        res.winner = Some(self.color);
                    }
                    return (res, i);
                }
            }
        }
        panic!("This shouldn't happen for validated moves.");
    }
}

impl<'gs> ValidMoveMut<'gs> {
    fn apply(self) -> (usize) {
        let b = self.valid_for;
        let n = self.index;
        for i in 0..HEIGHT {
            match b.board[i][n] {
                Full(_) => (),
                Empty => {
                    (*b).board[i][n] = Full(self.color);
                    if b.has_won(self.color) {
                        (*b).winner = Some(self.color);
                    }
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
            winner: None,
            _phantom: PhantomData,
        }
    }

    fn winner(&self) -> Option<Color> {
        self.winner
    }

    fn has_winner(&self) -> bool {
        self.winner().is_some()
    }

    fn possible_moves(&self, color: Color) -> Vec<ValidMove> {
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
        let get = |i, j| self.get(j, i);

        for j in 0..HEIGHT - (NEEDED - 1) {
            for i in 0..WIDTH {
                if get(i, j) == color && get(i, j + 1) == color && get(i, j + 2) == color &&
                   get(i, j + 3) == color {
                    return true;
                }
            }
        }
        // verticalCheck
        for i in 0..WIDTH - (NEEDED - 1) {
            for j in 0..HEIGHT {
                if get(i, j) == color && get(i + 1, j) == color && get(i + 2, j) == color &&
                   get(i + 3, j) == color {
                    return true;
                }
            }
        }

        // ascending diagonal check
        for i in 3..WIDTH {
            for j in 0..HEIGHT - (NEEDED - 1) {
                if get(i, j) == color && get(i - 1, j + 1) == color &&
                   get(i - 2, j + 2) == color && get(i - 3, j + 3) == color {
                    return true;
                }
            }
        }
        // // descendingDiagonalCheck
        for i in 3..WIDTH {
            for j in 3..HEIGHT {
                if get(i, j) == color && get(i - 1, j - 1) == color &&
                   get(i - 2, j - 2) == color && get(i - 3, j - 3) == color {
                    return true;
                }
            }
        }
        false
    }

    fn move_valid(&self, col: usize) -> bool {
        // TODO : Is this fine?
        if let Slot::Full(_) = self.board[HEIGHT - 1][col] {
            return false;
        }

        if col >= WIDTH {
            return false;
        }

        true
    }

    fn verify_move(self: Board<Clean>, col: usize, color: Color) -> Option<ValidMove> {
        if !self.move_valid(col) {
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
        if !self.move_valid(col) {
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
        self.verify_move_mut(n, c).map(|m| m.apply())
    }

    fn try_moves<I>(&mut self, moves: I) -> Vec<Option<usize>>
        where I: Iterator<Item = (usize, Color)>
    {
        moves.map(move |(col, color)| self.try_move(col, color))
            .collect()
    }

    fn minimax(&self, to_act: Color, max_depth: usize, trials: usize) -> (i32, usize) {
        let game = Game {
            state: self.clone(),
            to_act: to_act,
            ref_color: to_act,
        };
        let mut node = Node::new(game, max_depth);
        node.negamax_ab(0, trials, -(trials as i32), trials as i32)
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

#[derive(Hash, Clone)]
struct Game {
    state: Board<Clean>,
    to_act: Color,
    ref_color: Color,
}

#[derive(Clone)]
struct Node {
    game: Game,
    attrs: NodeInfo,
}

#[derive(Clone)]
struct NodeInfo {
    preceding_move: Option<usize>,
    rng: XorShiftRng,
    max_depth: usize,
    index_alloc: [usize; WIDTH],
}

impl Default for NodeInfo {
    fn default() -> Self {
        let seed = rand::random::<[u32; 4]>();
        Self::new_seeded(seed, 1)
    }
}

impl NodeInfo {
    fn new_seeded(seed: [u32; 4], max_depth: usize) -> Self {
        let rng: XorShiftRng = rand::SeedableRng::from_seed(seed);
        let mut indices = [0; WIDTH];

        for i in 0..WIDTH {
            indices[i] = i;
        }

        NodeInfo {
            preceding_move: None,
            rng: rng,
            max_depth: max_depth,
            index_alloc: indices,
        }
    }

    fn shuffle_indices(&mut self) {
        // TODO(justincullen): Fix this
        let mut indices = self.index_alloc;
        self.rng.shuffle(&mut indices);
        self.index_alloc = indices;
    }
}



impl Node {
    fn new_seeded(g: Game, seed: [u32; 4], max_depth: usize) -> Self {
        Node {
            game: g,
            attrs: NodeInfo::new_seeded(seed, max_depth),
        }
    }

    fn new(g: Game, max_depth: usize) -> Self {
        let seed = rand::random::<[u32; 4]>();
        Self::new_seeded(g, seed, max_depth)
    }

    fn max_depth(&self) -> usize {
        self.attrs.max_depth
    }

    fn rng(&mut self) -> &mut XorShiftRng {
        &mut self.attrs.rng
    }

    fn preceding_move(&self) -> Option<usize> {
        self.attrs.preceding_move
    }

    fn negamax(&mut self, trials: usize, depth: usize) -> (i32, usize) {
        let nexts = self.possibilities();
        if depth > self.max_depth() || nexts.is_empty() {
            let mv = self.preceding_move();
            assert!(!mv.is_none());
            return (self.heuristic(trials) * self.game.color_weight(self.game.to_act), mv.unwrap());
        }
        nexts.into_par_iter()
            .map(|mut node| {
                let (s, m) = node.negamax(trials, depth + 1);
                (-s, m)
            })
            .max()
            .unwrap()
    }

    fn negamax_ab(&mut self, depth: usize, trials: usize, alpha: i32, beta: i32) -> (i32, usize) {


        let nexts = self.possibilities();
        if depth > self.max_depth() || nexts.is_empty() {
            let mv = self.preceding_move();
            assert!(!mv.is_none());
            return (self.heuristic(trials) * self.game.color_weight(self.game.to_act), mv.unwrap());
        }

        let mut best = i32::min_value();
        let mut best_move = None;
        let mut alpha = alpha;
        for mut node in nexts {
            let (s, m) = node.negamax_ab(depth + 1, trials, -beta, -alpha);
            let s = -s;
            if s >= best {
                best = s;
                best_move = Some(m);
            }

            alpha = max(alpha, best);
            if alpha >= beta {
                break;
            }
        }

        (best, best_move.unwrap())
    }

    fn random_outcome(&mut self, c: Color) -> Option<Color> {
        let mut game = (*self).game.clone();
        game.to_act = c;
        let mut game_over: bool = game.state.has_winner();
        while !game_over {
            self.attrs.shuffle_indices();
            let mut found_legal_move = false;
            for &i in &self.attrs.index_alloc {
                match game.state.try_move_mut(i, game.to_act) {
                    None => continue,
                    Some(_) => {
                        found_legal_move = true;
                        break;
                    }
                };
            }

            if !found_legal_move {
                return None;
            }

            game.to_act = game.to_act.flip();
            game_over = game.state.has_winner();
        }
        game.state.winner()
    }

    fn monte_carlo(&mut self, trials: u32) -> u32 {
        let ref_color = self.game.ref_color;
        let color = self.game.to_act;
        (0..trials)
            .flat_map(|_| self.random_outcome(color))
            .filter(move |c| *c == ref_color)
            .map(|_| 1)
            .sum()
    }

    fn heuristic(&mut self, trials: usize) -> i32 {
        if let Some(c) = self.game.state.winner() {
            if c == self.game.ref_color {
                return i32::max_value();
            } else {
                return i32::min_value();
            }
        } else {
            self.monte_carlo(trials as u32) as i32
        }
    }

    fn possibilities(&self) -> Vec<Node> {
        let moves = self.game.state.possible_moves(self.game.to_act);
        moves.into_iter()
            .map(move |m| {
                let mut new_attrs = self.attrs.clone();
                new_attrs.preceding_move = Some(m.index());
                Node {
                    game: Game {
                        state: m.apply().0,
                        to_act: self.game.to_act.flip(),
                        ref_color: self.game.ref_color,
                    },
                    attrs: new_attrs,
                }
            })
            .collect()
    }
}

impl Game {
    fn color_weight(&self, c: Color) -> i32 {
        if c == self.ref_color { 1 } else { -1 }
    }


    fn ref_color_to_act(&self) -> bool {
        self.ref_color == self.to_act
    }
}

trait Player {
    fn choose_move(&mut self, board: &Board<Clean>, color: Color) -> usize;
    fn display_name(&self) -> &str;
    fn player_type(&self) -> &str;
    fn full_name(&self) -> String {
        format!("{} ({})", self.display_name(), self.player_type())
    }
}

struct HumanPlayer {
    name: String,
}

impl HumanPlayer {
    fn new(name: &str) -> Self {
        HumanPlayer { name: String::from(name) }
    }
}

impl Player for HumanPlayer {
    fn display_name(&self) -> &str {
        self.name.as_str()
    }

    fn player_type(&self) -> &str {
        "Human"
    }

    fn choose_move(&mut self, board: &Board<Clean>, color: Color) -> usize {
        println!("{}'s move.", color.show());

        loop {
            println!("What is your move?");
            print!("Enter a number between {} and {}:", 1, WIDTH);
            let mut choice = String::new();
            io::stdin()
                .read_line(&mut choice)
                .expect("Failed to read line... something is mad broke.");
            println!("");

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
}
struct AIPlayer {
    name: String,
    search_depth: usize,
    trials: usize,
}

impl Player for AIPlayer {
    fn display_name(&self) -> &str {
        self.name.as_str()
    }

    fn player_type(&self) -> &str {
        "Computer"
    }

    fn choose_move(&mut self, board: &Board<Clean>, color: Color) -> usize {
        println!("Computer is thinking.....");
        let (s, m) = board.minimax(color, self.search_depth, self.trials);
        println!("score: {}", s);
        m
    }
}

impl AIPlayer {
    fn new(name: &str, search_depth: usize, trials: usize) -> Self {
        AIPlayer {
            name: String::from(name),
            search_depth: search_depth,
            trials: trials,
        }
    }
}

type Plr<'a> = &'a mut Player;

struct Runner<'a> {
    board: Board<Clean>,
    to_act: Color,
    players: (Plr<'a>, Plr<'a>),
    winner: Option<Color>,
}

impl<'a> Runner<'a> {
    fn new(p1: Plr<'a>, p2: Plr<'a>) -> Self {
        Self::new_with_first_to_act(Color::random(), p1, p2)
    }

    fn new_with_first_to_act(color: Color, p1: Plr<'a>, p2: Plr<'a>) -> Self {
        Runner {
            board: Board::new(),
            to_act: color,
            players: (p1, p2),
            winner: None,
        }
    }

    fn show_board(&self) {
        println!();
        println!("{}", self.board);
    }
    fn init(&mut self) {
        println!("CONNECT FOUR");
        println!("Player 1, {}, is {}'s.",
                 self.players.0.full_name(),
                 R.show());
        println!("Player 2, {}, is {}'s.",
                 self.players.1.full_name(),
                 B.show());
        println!("Flipping to see who starts...");

        (*self).to_act = Color::random();
        println!("{} goes first!", self.to_act.show());

        (*self).board = Board::new();
    }

    fn check_winner(&mut self) -> bool {
        (*self).winner = self.board.winner();
        self.winner.is_some()
    }

    fn step(&mut self) {
        let cloned_board = self.board.clone();
        self.show_board();
        match self.to_act {
            R => {
                let p1_move = (*self).players.0.choose_move(&cloned_board, R);
                self.board.try_move(p1_move, R);
            }
            B => {
                let p2_move = (*self).players.1.choose_move(&cloned_board, B);
                self.board.try_move(p2_move, B);
            }
        }
        (*self).to_act = self.to_act.flip();
    }

    fn game_loop(&mut self) {
        while !self.check_winner() {
            self.step()
        }

        let winner = self.winner.unwrap();
        println!("Winner is: {}", winner.show());
    }

    fn run<'b>(p1: Plr<'b>, p2: Plr<'b>) -> Option<Color> {
        let mut runner = Runner::new(p1, p2);
        runner.init();
        runner.game_loop();
        runner.winner
    }
}

fn do_main() {
    let matches = App::new("Connect Four")
        .version("0.1.0")
        .about("Simple project to play with while learning rust")
        .arg(Arg::with_name("depth")
            .short("d")
            .value_name("UINT")
            .long("search_depth")
            .help("Specifies how many game tree levels the AI will search before trying \
                   heuristics.")
            .takes_value(true))
        .arg(Arg::with_name("trials")
            .short("t")
            .value_name("UINT")
            .long("monte_carlo_trials")
            .help("How many monte carlo trials to run for the heuristic.")
            .takes_value(true))
        .get_matches();

    let mut human = HumanPlayer::new("Justin");
    let depth = value_t!(matches.value_of("depth"), usize).unwrap_or_else(|e| e.exit());
    let trials = value_t!(matches.value_of("trials"), usize).unwrap_or_else(|e| e.exit());
    let mut pc = AIPlayer::new("IRobot", depth, trials);
    Runner::run(&mut human, &mut pc);
}

fn main() {
    PROFILER.lock().unwrap().start("./my-prof2.profile");
    println!("{:?}", Board::new().minimax(R, 6, 100));
    PROFILER.lock().unwrap().stop();
}
