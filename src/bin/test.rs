extern crate gameai;
extern crate rayon;

use rayon::prelude::*;
use gameai::strategies::mcts_rayon::MCTS;

fn main() {
    let input: Vec<_> = (0..100000).collect();
    let mut result = Vec::new();
    result.par_extend(input.par_iter().map(|&x| x * 2));
    println!("{}", result.into_par_iter().sum::<i64>());
}
