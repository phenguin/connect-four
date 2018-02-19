#![allow(unused)]
use super::*;
use rand;
use std::default::Default;
use std::fmt;
pub const MAX_STATE: i64 = 49;


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Player {
    A,
    B,
}

impl Player {
    fn other(&self) -> Self {
        match *self {
            Player::A => Player::B,
            Player::B => Player::A,
        }
    }
}

impl rand::Rand for Player {
    fn rand<R: rand::Rng>(rng: &mut R) -> Player {
        if rng.gen() {
            return Player::A;
        }
        Player::B
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <Player as fmt::Debug>::fmt(self, f)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct PlayerState(i64);

impl fmt::Display for PlayerState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <PlayerState as fmt::Debug>::fmt(self, f)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TrivialGame {
    to_act: Player,
    states: [PlayerState; 2],
}

impl fmt::Display for TrivialGame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <TrivialGame as fmt::Debug>::fmt(self, f)
    }
}

impl Game for TrivialGame {
    type Move = ();
    type Agent = Player;
    fn to_act(&self) -> Self::Agent { self.to_act }
    fn player_weight(&self, &p: &Self::Agent) -> Score {
        match p {
            Player::A => 1,
            Player::B => -1,
        }
    }
    fn winner(&self) -> Option<Self::Agent> {
        for p in &[Player::A, Player::B] {
            if self.has_won(p) {
                return Some(*p);
            }
        }
        None
    }
    fn has_won(&self, &p: &Self::Agent) -> bool {
        self.state(p) == MAX_STATE
    }
    fn agent_id(&self, &p: &Self::Agent) -> u32 {
        match p {
            Player::A => 0,
            Player::B => 1,
        }
    }
    fn ref_player(&self) -> Self::Agent {
        Player::A
    }

    fn new(&p: &Self::Agent) -> Self {
        Self {
            to_act: p,
            states: [PlayerState(0),PlayerState(0)],
        }
    }


    fn move_valid(&self, _: &Self::Move) -> bool {
        true
    }

    fn possible_moves(&self) -> Vec<ValidMove<Self>> {
        self.verify_move(()).into_iter().collect()
    }

    fn apply(&mut self, _: Self::Move) {
        self.states[self.agent_id(&self.to_act) as usize].0 += 1;
        self.to_act = self.to_act.other()
    }
}

impl RandGame for TrivialGame {}

impl ParseGame for TrivialGame {
    fn parse_move(&self, _: &str) -> Option<Self::Move> {
        Some(())
    }
}

impl TrivialGame {
    fn state(&self, p: Player) -> i64 {
        self.states[self.agent_id(&p) as usize].0
    }
}



