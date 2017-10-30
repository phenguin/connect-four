#![feature(use_extern_macros)]

extern crate gameai;
extern crate clap;

use clap::{Arg, App, value_t};


use gameai::game;
use gameai::runner;

fn do_main() {
    let matches = App::new("Connect Four")
        .version("0.1.0")
        .about("Simple project to play with while learning rust")
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .value_name("UINT")
                .long("monte_carlo_timeout")
                .help("How many milliseconds to give the monte carlo simulator to run.")
                .takes_value(true))
        .arg(
            Arg::with_name("workers")
                .short("w")
                .value_name("UINT")
                .long("monte_carlo_workers")
                .help("How many parallel workers for MCTS.")
                .takes_value(true))
        .get_matches();

    let timeout = value_t!(matches.value_of("timeout"), u64).unwrap_or_else(|e| e.exit());
    let workers = value_t!(matches.value_of("workers"), u64).unwrap_or_else(|e| e.exit());

    use game::connectfour::ConnectFour;
    use gameai::strategies::negamax::*;
    use gameai::strategies::mcts_parallel::*;
    use runner::AIPlayer;
    let mut human = runner::HumanPlayer::new("Justin");
    let mut pc2 = AIPlayer::<ConnectFour, MCTS<ConnectFour>>::new(
        "IRobot",
        MCTSParams {
            workers: workers,
            timeout: timeout,
            c: (2.0 as f64).sqrt(),
        },
    );
    runner::Runner::run(&mut human, &mut pc2);
}

fn main() {
    do_main();
}
