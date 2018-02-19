#![allow(unused_must_use)]

extern crate gameai;
extern crate cpuprofiler;

use cpuprofiler::PROFILER;
use gameai::strategies::Strategy;
use gameai::game::trivial::*;
use gameai::strategies::mcts_rayon;
use gameai::*;
use gameai::game::Game;

fn do_profile() {
    PROFILER.lock().unwrap().start("./negamax.profile");
    let mut strategy = mcts_rayon::MCTS::create(mcts_rayon::MCTSParams {
        workers: 1,
        worker_batch_size: 1,
        merger_batch_size: 1,
        min_flush_interval: 1,
        merger_queue_bound: 1,
        timeout: 1,
        c: (2.0 as f64).sqrt(),
    });
    debug(&strategy.decide(&TrivialGame::new(&Player::A)));
    PROFILER.lock().unwrap().stop();
}


fn main() {
    do_profile();
}
