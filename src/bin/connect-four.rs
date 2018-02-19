#![feature(use_extern_macros)]
#![allow(unused)]

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
                .help(
                    "How many milliseconds to give the monte carlo simulator to run.",
                )
                .takes_value(true),
        )
        .arg(
            Arg::with_name("workers")
                .short("w")
                .value_name("UINT")
                .long("monte_carlo_workers")
                .default_value("1")
                .help("How many parallel workers for MCTS.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("worker_batch_size")
                .short("wb")
                .value_name("UINT")
                .long("worker_batch_size")
                .default_value("200")
                .help("Batch size for workers.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("merger_batch_size")
                .short("wm")
                .value_name("UINT")
                .long("merger_batch_size")
                .default_value("100")
                .help("Batch size for mergers.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("min_flush_interval")
                .short("i")
                .value_name("UINT")
                .long("min_flush_interval")
                .default_value("100")
                .help("Minimum time in between worker stats flushes.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("merger_queue_bound")
                .short("i")
                .value_name("UINT")
                .long("merger_queue_bound")
                .help("Max length of the merger input channel of updates.")
                .default_value("200")
                .takes_value(true),
        )
        .get_matches();

    let timeout = value_t!(matches.value_of("timeout"), u64).unwrap_or_else(|e| e.exit());
    let workers = value_t!(matches.value_of("workers"), u64).unwrap_or_else(|e| e.exit());
    let worker_batch_size = value_t!(matches.value_of("worker_batch_size"), u64)
        .unwrap_or_else(|e| e.exit());
    let merger_batch_size = value_t!(matches.value_of("worker_batch_size"), u64)
        .unwrap_or_else(|e| e.exit());
    let min_flush_interval = value_t!(matches.value_of("min_flush_interval"), u64)
        .unwrap_or_else(|e| e.exit());
    let merger_queue_bound = value_t!(matches.value_of("merger_queue_bound"), usize)
        .unwrap_or_else(|e| e.exit());

    use game::connectfour::ConnectFour;
    use gameai::strategies::mcts;
    use gameai::strategies::mcts_parallel;
    use gameai::strategies::mcts_rayon;
    use runner::AIPlayer;
    let mut _human = runner::HumanPlayer::new("Justin");
    let mut _pc1 = AIPlayer::<ConnectFour, mcts::MCTS<ConnectFour>>::new(
        "MCTS_AI",
        mcts::MCTSParams {
            timeout: timeout,
            c: (2.0 as f64).sqrt(),
        },
    );
    use runner::NetworkPlayer;
    use std::net::TcpListener;
    // let mut _net = NetworkPlayer::new("Network guy", TcpListener::bind("127.0.0.1:8080").unwrap());
    let mut _pc2 = AIPlayer::<ConnectFour, mcts_rayon::MCTS<ConnectFour>>::new(
        "RAYON_MCTS_AI",
        mcts_rayon::MCTSParams {
            workers: workers,
            worker_batch_size: worker_batch_size,
            merger_batch_size: merger_batch_size,
            min_flush_interval: min_flush_interval,
            merger_queue_bound: merger_queue_bound,
            timeout: timeout,
            c: (2.0 as f64).sqrt(),
        },
    );

    let mut _pc3 = AIPlayer::<ConnectFour, mcts_parallel::MCTS<ConnectFour>>::new(
        "RAYON_MCTS_AI",
        mcts_parallel::MCTSParams {
            workers: workers,
            worker_batch_size: worker_batch_size,
            merger_batch_size: merger_batch_size,
            min_flush_interval: min_flush_interval,
            merger_queue_bound: merger_queue_bound,
            timeout: timeout,
            c: (2.0 as f64).sqrt(),
        },
    );
    runner::Runner::run(&mut _pc2, &mut _human);
}

fn main() {
    do_main();
}
