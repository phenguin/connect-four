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
use std::cmp::{max, min};
use std::default::Default;
use std::fmt;
use std::io;
use std::marker::PhantomData;
use std::str::FromStr;
use std::cmp::Eq;
use std::mem::transmute;

const HEIGHT: usize = 6;
const WIDTH: usize = 7;
const NEEDED: usize = 4;

#[derive(Clone, Copy, PartialOrd, PartialEq, Hash, Debug, Ord, Eq)]
enum Color {
    R,
    B,
}

impl rand::Rand for Color {
    fn rand<R: rand::Rng>(rng: &mut R) -> Color {
        Color::random()
    }
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

#[derive(Clone, Copy, PartialEq, Hash, Debug)]
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

#[derive(Hash, Debug)]
struct C4Board {
    board: [[Slot; WIDTH]; HEIGHT],
}
impl std::clone::Clone for C4Board {
    fn clone(&self) -> C4Board {
        C4Board { board: self.board }
    }
}

trait ParseGame: Game {
    fn parse_move(&self, &str) -> Option<Self::Move>;
}

trait Game: Clone + Send {
    type Move: Clone + Copy + Send + Ord;
    type Agent: PartialEq + Clone + Copy + Send + Ord;
    fn to_act(&self) -> Self::Agent;
    fn player_weight(&self, &Self::Agent) -> Score;
    fn winner(&self) -> Option<Self::Agent>;
    fn agent_id(&self, &Self::Agent) -> u32;

    fn ref_player(&self) -> Self::Agent;
    fn new(&Self::Agent) -> Self;
    fn possible_moves(&self) -> Vec<ValidMove<Self>>;

    fn try_move_mut(&mut self, m: Self::Move) -> bool {
        self.verify_move_mut(m).map(|m| m.apply()).is_some()
    }
    fn verify_move_mut(&mut self, m: Self::Move) -> Option<ValidMoveMut<Self>> {
        if !self.move_valid(&m) {
            return None;
        }

        Some(ValidMoveMut {
            valid_move: m,
            valid_for: self,
        })
    }

    fn verify_move(self: Self, m: Self::Move) -> Option<ValidMove<Self>> {
        if !self.move_valid(&m) {
            return None;
        }

        if self.has_winner() {
            return None;
        }

        Some(ValidMove {
            valid_move: m,
            valid_for: self,
        })
    }

    fn move_valid(&self, &Self::Move) -> bool;

    fn has_won(&self, agent: &Self::Agent) -> bool;

    fn has_winner(&self) -> bool {
        self.winner().is_some()
    }

    fn apply(&mut self, Self::Move);

    fn try_move(&mut self, m: Self::Move) -> bool {
        let x = self.clone().verify_move(m);
        x.map(move |valid_move| {
                let new_board = valid_move.apply();
                *self = new_board;
            })
            .is_some()
    }

    fn try_moves<I>(&mut self, moves: I) -> Vec<bool>
        where I: Iterator<Item = Self::Move>
    {
        moves.map(move |m| self.try_move(m))
            .collect()
    }
}

#[derive(Hash, Debug)]
struct Dirty {}
#[derive(Hash, Debug)]
struct Clean {}

#[derive(Hash, Clone)]
struct ValidMove<G: Game> {
    valid_move: G::Move,
    valid_for: G,
}

struct ValidMoveMut<'gs, G: Game + 'gs> {
    valid_move: G::Move,
    valid_for: &'gs mut G,
}

impl<G: Game> ValidMove<G> {
    pub fn board(&self) -> &G {
        &self.valid_for
    }

    pub fn valid_move(&self) -> &G::Move {
        &self.valid_move
    }

    fn apply(mut self) -> G {
        self.valid_for.apply(self.valid_move);
        self.valid_for
    }
}

impl<'gs, G: Game> ValidMoveMut<'gs, G> {
    fn apply(self) {
        self.valid_for.apply(self.valid_move);
    }
}

impl C4Board {
    fn get(&self, i: usize, j: usize) -> Slot {
        self.board[i][j]
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

#[derive(Hash, Clone, Debug)]
struct ConnectFour {
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

#[derive(Clone)]
struct Node<G: Game> {
    game: G,
    attrs: NodeInfo<G>,
}

impl<G> fmt::Display for Node<G>
    where G: Game + fmt::Display,
          G::Agent: fmt::Display,
          G::Move: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.attrs)?;
        writeln!(f, "{}", self.game)
    }
}

#[derive(Clone)]
struct NodeInfo<G: Game> {
    preceding_move: Option<G::Move>,
    rng: XorShiftRng,
    max_depth: usize,
    index_alloc: [usize; WIDTH],
}

impl<G> fmt::Display for NodeInfo<G>
    where G: Game + fmt::Display,
          G::Agent: fmt::Display,
          G::Move: fmt::Display
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self.preceding_move {
            None => String::from("<ROOT>"),
            Some(m) => format!("{}", m),
        };
        writeln!(f, "last move: {}", s)
    }
}

impl<G: Game> Default for NodeInfo<G> {
    fn default() -> Self {
        let seed = rand::random::<[u32; 4]>();
        Self::new_seeded(seed, 1)
    }
}

impl<G: Game> NodeInfo<G> {
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

trait Strategy<G: Game> {
    type Params;
    fn decide(&mut self, &G) -> G::Move;
    fn create(Self::Params) -> Self;
}

struct NegamaxState {}
struct NegamaxParams {
    max_depth: usize,
    trials: usize,
}

struct Negamax<G>
    where G: Game + Send,
          G::Agent: Send,
          G::Move: Send + Ord
{
    params: NegamaxParams,
    state: NegamaxState,
    _phantom: PhantomData<G>,
}

type Score = i32;
impl ParseGame for ConnectFour {
    fn parse_move(&self, input: &str) -> Option<Self::Move> {
        usize::from_str(input).ok().map(|n| (n, self.to_act()))
    }
}

impl Game for ConnectFour {
    type Move = (usize, Self::Agent);
    type Agent = Color;

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
        if let Slot::Full(_) = state.board[HEIGHT - 1][col] {
            return false;
        }

        if col >= WIDTH || color != self.to_act {
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
                    self.state.board[i][n] = Full(color);
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
}

impl<G> Negamax<G>
    where G: Game + Send + fmt::Display,
          G::Agent: Send,
          G::Move: Send + Ord
{
    fn negamax(&mut self, game: &G) -> (Score, Option<G::Move>) {
        let NegamaxParams { trials, max_depth, .. } = self.params;
        let mut node = Node::new(game, max_depth);
        node.negamax(trials, 0)
    }
}

impl<G> Strategy<G> for Negamax<G>
    where G: Game + Send + fmt::Display,
          G::Agent: Send,
          G::Move: Send + Ord
{
    type Params = NegamaxParams;
    fn decide(&mut self, game: &G) -> G::Move {
        let (_, maybe_move) = self.negamax(game);
        maybe_move.expect("No moves available from start position.")
    }
    fn create(params: NegamaxParams) -> Self {
        Self {
            params: params,
            state: NegamaxState {},
            _phantom: PhantomData,
        }
    }
}


impl<G> Node<G>
    where G: Game + Send + fmt::Display,
          G::Agent: Send,
          G::Move: Send + Ord
{
    fn new_seeded(g: &G, seed: [u32; 4], max_depth: usize) -> Self {
        Node {
            game: g.clone(),
            attrs: NodeInfo::new_seeded(seed, max_depth),
        }
    }

    fn new(g: &G, max_depth: usize) -> Self {
        let seed = rand::random::<[u32; 4]>();
        Self::new_seeded(g, seed, max_depth)
    }

    fn max_depth(&self) -> usize {
        self.attrs.max_depth
    }

    fn rng(&mut self) -> &mut XorShiftRng {
        &mut self.attrs.rng
    }

    fn preceding_move(&self) -> Option<G::Move> {
        self.attrs.preceding_move
    }

    fn negamax(&mut self, trials: usize, depth: usize) -> (Score, Option<G::Move>) {
        let nexts = self.possibilities();
        if depth > self.max_depth() || nexts.is_empty() {
            return (self.heuristic(trials) as Score * self.game.player_weight(&self.game.to_act()),
                    None);
        }
        let it = nexts.into_par_iter()
            .map(|mut node| {
                let (s, _) = node.negamax(trials, depth + 1);
                (-s, node.preceding_move())
            })
            .max()
            .unwrap();
        it
    }

    // TODO Make this not super slow.
    fn random_outcome(&mut self) -> Option<G::Agent> {
        let mut game = (*self).game.clone();

        while !game.has_winner() {
            let mut moves = game.possible_moves();

            if moves.is_empty() {
                return None;
            }

            self.rng().shuffle(&mut moves);
            let m = moves[0].clone();
            game = m.apply();
            if game.has_winner() {
                return game.winner();
            }
        }
        None
    }

    fn monte_carlo(&mut self, trials: u32) -> u32 {
        let ref_color = self.game.ref_player();
        let color = self.game.to_act();
        (0..trials)
            .flat_map(|_| self.random_outcome())
            .filter(move |c| *c == ref_color)
            .map(|_| 1)
            .sum()
    }

    fn heuristic(&mut self, trials: usize) -> usize {
        if let Some(c) = self.game.winner() {
            return if c == self.game.ref_player() {
                trials
            } else {
                0
            };
        };

        self.monte_carlo(trials as u32) as usize

    }

    fn possibilities(&mut self) -> Vec<Self> {
        let mut moves = self.game.possible_moves();
        // self.rng().shuffle(moves.as_mut_slice());
        moves.into_iter()
            .map(move |m| {
                let mut new_attrs = self.attrs.clone();
                new_attrs.preceding_move = Some(m.valid_move().clone());
                Node {
                    game: m.apply(),
                    attrs: new_attrs,
                }

            })
            .collect()
    }
}

trait Player<G>
    where G: Game + Send,
          G::Agent: Send,
          G::Move: Send + Ord
{
    fn choose_move(&mut self, game: &G) -> G::Move;
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

impl<G> Player<G> for HumanPlayer
    where G: ParseGame + Send,
          G::Agent: Send + fmt::Display,
          G::Move: Send + Ord + fmt::Debug
{
    fn display_name(&self) -> &str {
        self.name.as_str()
    }

    fn player_type(&self) -> &str {
        "Human"
    }

    fn choose_move(&mut self, game: &G) -> G::Move {
        let agent = game.to_act();
        println!("{}'s move.", agent);

        loop {
            println!("What is your move?");
            let mut choice = String::new();
            io::stdin()
                .read_line(&mut choice)
                .expect("Failed to read line... something is mad broke.");
            println!("");

            let choice = match game.parse_move(choice.trim()) {
                Some(m) => m,
                None => continue,
            };

            println!("{:?}", choice);

            if !game.move_valid(&choice) {
                println!("Invalid move..");
                continue;
            }

            return choice;
        }
    }
}

struct AIPlayer<G: Game> {
    name: String,
    strategy: Negamax<G>,
}

impl<G> Player<G> for AIPlayer<G>
    where G: Game + Send + fmt::Display,
          G::Agent: Send,
          G::Move: Send + Ord + fmt::Debug
{
    fn display_name(&self) -> &str {
        self.name.as_str()
    }

    fn player_type(&self) -> &str {
        "Computer"
    }

    fn choose_move(&mut self, board: &G) -> G::Move {
        println!("Computer is thinking.....");
        let m = self.strategy.decide(board);
        println!("CHOSE MOVE: {:?}", m);
        m
    }
}

impl<G: Game + fmt::Display> AIPlayer<G> {
    fn new(name: &str, search_depth: usize, trials: usize) -> Self {
        AIPlayer {
            name: String::from(name),
            strategy: Negamax::create(NegamaxParams {
                max_depth: search_depth,
                trials: trials,
            }),
        }
    }
}

type Plr<'a, G> = &'a mut Player<G>;

struct Runner<'a, G: Game + 'a> {
    board: G,
    players: (Plr<'a, G>, Plr<'a, G>),
    winner: Option<G::Agent>,
}

impl<'a, G> Runner<'a, G>
    where G: Game + Send + fmt::Display + Clone,
          G::Agent: Send + rand::Rand + fmt::Display,
          G::Move: Send + Ord
{
    fn new(p1: Plr<'a, G>, p2: Plr<'a, G>) -> Self {
        Self::new_with_first_to_act(rand::random::<G::Agent>(), p1, p2)
    }

    fn new_with_first_to_act(agent: G::Agent, p1: Plr<'a, G>, p2: Plr<'a, G>) -> Self {
        Runner {
            board: G::new(&agent),
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

        println!("{} goes first!", self.board.to_act());
    }

    fn check_winner(&mut self) -> bool {
        self.board.has_winner()
    }

    fn step(&mut self) {
        let cloned_board = self.board.clone();
        println!("{}", cloned_board);
        if self.board.agent_id(&cloned_board.to_act()) == 0 {
            let p1_move = (*self).players.0.choose_move(&cloned_board);
            self.board.try_move(p1_move);
        } else {
            let p2_move = (*self).players.1.choose_move(&cloned_board);
            self.board.try_move(p2_move);
        }
    }


    fn game_loop(&mut self) {
        while !self.check_winner() {
            self.step()
        }

        let winner = self.winner.unwrap();
        println!("Winner is: {}", winner);
    }

    fn run<'b>(p1: Plr<'b, G>, p2: Plr<'b, G>) -> Option<G::Agent> {
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

    let depth = value_t!(matches.value_of("depth"), usize).unwrap_or_else(|e| e.exit());
    let trials = value_t!(matches.value_of("trials"), usize).unwrap_or_else(|e| e.exit());

    // do_test(depth, trials);

    let mut human = HumanPlayer::new("Justin");
    let mut pc = AIPlayer::<ConnectFour>::new("IRobot", depth, trials);
    Runner::run(&mut human, &mut pc);
}

fn debug<T: fmt::Display>(x: &T) {
    println!("{}", x);
}

fn ddebug<T: fmt::Debug>(x: &T) {
    println!("{:?}", x);
}

// fn do_profile() {
//     PROFILER.lock().unwrap().start("./my-prof2.profile");
//     println!("{:?}", ConnectFour::new().minimax(R, 6, 100));
//     PROFILER.lock().unwrap().stop();
// }

fn main() {
    do_main();
}
