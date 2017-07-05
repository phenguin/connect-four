use game::{Game, ParseGame};
use std::fmt;
use std::io;
use rand;
use strategies::{Strategy, Negamax, NegamaxParams};
pub trait Player<G>
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

pub struct HumanPlayer {
    name: String,
}

impl HumanPlayer {
    pub fn new(name: &str) -> Self {
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

pub struct AIPlayer<G: Game> {
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
    pub fn new(name: &str, search_depth: usize, trials: usize) -> Self {
        AIPlayer {
            name: String::from(name),
            strategy: Negamax::create(NegamaxParams {
                max_depth: search_depth,
                trials: trials,
            }),
        }
    }
}

pub type Plr<'a, G> = &'a mut Player<G>;

pub struct Runner<'a, G: Game + 'a> {
    board: G,
    players: (Plr<'a, G>, Plr<'a, G>),
    winner: Option<G::Agent>,
}

impl<'a, G> Runner<'a, G>
    where G: Game + Send + fmt::Display + Clone,
          G::Agent: Send + rand::Rand + fmt::Display,
          G::Move: Send + Ord
{
    pub fn new(p1: Plr<'a, G>, p2: Plr<'a, G>) -> Self {
        Self::new_with_first_to_act(rand::random::<G::Agent>(), p1, p2)
    }

    pub fn new_with_first_to_act(agent: G::Agent, p1: Plr<'a, G>, p2: Plr<'a, G>) -> Self {
        Runner {
            board: G::new(&agent),
            players: (p1, p2),
            winner: None,
        }
    }

    fn init(&mut self) {
        println!("CONNECT FOUR");
        println!("Player 1 is {}", self.players.0.full_name());
        println!("Player 2 is {}", self.players.1.full_name());
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

    pub fn run<'b>(p1: Plr<'b, G>, p2: Plr<'b, G>) -> Option<G::Agent> {
        let mut runner = Runner::new(p1, p2);
        runner.init();
        runner.game_loop();
        runner.winner
    }
}
