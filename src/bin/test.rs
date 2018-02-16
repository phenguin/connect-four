extern crate gameai;
extern crate rayon;

use rayon::prelude::*;

fn main() {
    let input: Vec<_> = (0..100000).collect();
    let mut result = Vec::new();
    result.par_extend(input.par_iter().map(|&x| x * 2));
    println!("{}", result.into_par_iter().sum::<i64>());
}
