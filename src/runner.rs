use game::{RandGame, Game, ParseGame};
use std::fmt;
use std::sync::mpsc;
use std::io;
use rand;
use strategies::Strategy;
pub trait Player<G>
where
    G: Game + Send,
    G::Agent: Send,
    G::Move: Send + Ord,
{
    fn choose_move(&mut self, game: &G, output: mpsc::Sender<G::Move>);
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
where
    G: ParseGame + Send,
    G::Agent: Send + fmt::Display,
    G::Move: Send + Ord + fmt::Debug,
{
    fn display_name(&self) -> &str {
        self.name.as_str()
    }

    fn player_type(&self) -> &str {
        "Human"
    }

    fn choose_move(&mut self, game: &G, output: mpsc::Sender<G::Move>) {
        let agent = game.to_act();
        println!("{}'s move.", agent);

        loop {
            println!("What is your move?");
            let mut choice = String::new();
            io::stdin().read_line(&mut choice).expect(
                "Failed to read line... something is mad broke.",
            );
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

            output.send(choice).expect("Sending move choice failed");
            break;
        }
    }
}

use std::marker::PhantomData;
pub struct AIPlayer<G: Game, S: Strategy<G>> {
    name: String,
    strategy: S,
    _phantom: PhantomData<G>,
}

impl<G, S> Player<G> for AIPlayer<G, S>
where
    G: RandGame + Send + fmt::Display,
    G::Agent: Send,
    G::Move: Send + Ord + fmt::Debug,
    S: Strategy<G>,
{
    fn display_name(&self) -> &str {
        self.name.as_str()
    }

    fn player_type(&self) -> &str {
        "Computer"
    }

    fn choose_move(&mut self, board: &G, output: mpsc::Sender<G::Move>) {
        println!("{} is thinking.....", self.name);
        let m = self.strategy.decide(board);
        println!("CHOSE MOVE: {:?}", m);
        output.send(m).expect("Send failed.");
    }
}

impl<S, G: Game + fmt::Display> AIPlayer<G, S>
where
    S: Strategy<G>,
{
    pub fn new(name: &str, params: S::Params) -> Self {
        AIPlayer {
            name: String::from(name),
            strategy: S::create(params),
            _phantom: PhantomData,
        }
    }
}

pub type Plr<'a, G> = &'a mut Player<G>;

pub struct Runner<'a, G: Game + 'a> {
    board: G,
    players: (Plr<'a, G>, Plr<'a, G>),
    channel: (mpsc::Sender<G::Move>, mpsc::Receiver<G::Move>),
}

impl<'a, G> Runner<'a, G>
where
    G: Game + Send + fmt::Display + Clone,
    G::Agent: Send + rand::Rand + fmt::Display,
    G::Move: Send + Ord,
{
    pub fn new(p1: Plr<'a, G>, p2: Plr<'a, G>) -> Self {
        // Self::new_with_first_to_act(rand::random::<G::Agent>(), p1, p2)
        Self::new_with_first_to_act(rand::random::<G::Agent>(), p1, p2)
    }

    pub fn new_with_first_to_act(agent: G::Agent, p1: Plr<'a, G>, p2: Plr<'a, G>) -> Self {
        Runner {
            board: G::new(&agent),
            players: (p1, p2),
            channel: mpsc::channel(),
        }
    }

    fn init(&mut self) {
        println!("Player 1 is {}", self.players.0.full_name());
        println!("Player 2 is {}", self.players.1.full_name());
        println!("Flipping to see who starts...");

        println!("{} goes first!", self.board.to_act());
    }

    fn check_winner(&mut self) -> bool {
        self.board.has_winner()
    }

    fn step(&mut self) {
        println!("{}", self.board);
        // Hacky: Generalize to multi player games.
        let to_act_id = self.board.agent_id(&self.board.to_act());
        if to_act_id == 0 {
            (*self).players.0.choose_move(&self.board, self.channel.0.clone());
        }

        if to_act_id == 1 {
            (*self).players.1.choose_move(&self.board, self.channel.0.clone());
        }

        let next_move = self.channel.1.recv().expect("Receiving next move failed.");
        let success = self.board.try_move(next_move);

        if !success {
            println!("Received invalid move");
            return;
        }
    }


    fn game_loop(&mut self) {
        while !self.check_winner() {
            self.step()
        }

        let winner = self.board.winner().unwrap();
        println!("Winner is: {}", winner);
    }

    pub fn run<'b>(p1: Plr<'b, G>, p2: Plr<'b, G>) -> Option<G::Agent> {
        let mut runner = Runner::new(p1, p2);
        runner.init();
        runner.game_loop();
        runner.board.winner()
    }
}
