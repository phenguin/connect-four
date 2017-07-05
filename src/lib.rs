#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

use std::fmt;

extern crate rand;
extern crate rayon;
extern crate clap;
extern crate cpuprofiler;

pub mod game;
pub mod strategies;
pub mod runner;

#[allow(dead_code)]
pub fn display<T: fmt::Display>(x: &T) {
    println!("{}", x);
}

#[allow(dead_code)]
pub fn debug<T: fmt::Debug>(x: &T) {
    println!("{:?}", x);
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::strategies::*;
    use super::game::*;
    use super::game::connectfour::*;
    use test::Bencher;

    #[test]
    fn it_works() {
        assert_eq!(4, add_two(2));
    }

    #[bench]
    fn bench_negamax(b: &mut Bencher) {
        b.iter(|| {
            let mut strategy = Negamax::create(NegamaxParams {
                max_depth: 3,
                trials: 10,
            });
            debug(&strategy.decide(&ConnectFour::new(&Color::R)));
        });
    }

    #[bench]
    fn bench_random_outcome(b: &mut Bencher) {
        b.iter(|| {
            let mut strategy = Negamax::create(NegamaxParams {
                max_depth: 2,
                trials: 100,
            });
            debug(&strategy.decide(&ConnectFour::new(&Color::R)));
        });
    }
}
