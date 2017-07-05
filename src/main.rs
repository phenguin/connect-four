#![feature(plugin)]
#![feature(conservative_impl_trait)]
#![feature(use_extern_macros)]
#![plugin(clippy)]
// #![allow(dead_code)]

extern crate rand;
extern crate rayon;
extern crate clap;
extern crate cpuprofiler;

// use cpuprofiler::PROFILER;


use clap::{Arg, App, value_t};
use std::fmt;

mod game;
mod strategies;
mod runner;

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

    let mut human = runner::HumanPlayer::new("Justin");
    let mut pc = runner::AIPlayer::<game::connectfour::ConnectFour>::new("IRobot", depth, trials);
    runner::Runner::run(&mut human, &mut pc);
}

fn display<T: fmt::Display>(x: &T) {
    println!("{}", x);
}

fn debug<T: fmt::Debug>(x: &T) {
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
