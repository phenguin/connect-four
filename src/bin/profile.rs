#![allow(unused_must_use)]

extern crate gameai;
extern crate cpuprofiler;

use cpuprofiler::PROFILER;
use gameai::strategies::{Negamax, NegamaxParams, Strategy};
use gameai::game::Game;
use gameai::game::connectfour::{ConnectFour, Color};
use gameai::*;

fn do_profile() {
    PROFILER.lock().unwrap().start("./negamax.profile");
    let mut strategy = Negamax::create(NegamaxParams {
        max_depth: 5,
        trials: 100,
    });
    debug(&strategy.decide(&ConnectFour::new(&Color::R)));
    PROFILER.lock().unwrap().stop();
}


fn main() {
    do_profile();
}
