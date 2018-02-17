#![feature(test, generators, generator_trait, conservative_impl_trait)]


use std::fmt;

extern crate test;
extern crate evmap;
extern crate rand;
extern crate rayon;
extern crate clap;
extern crate cpuprofiler;

#[macro_use]
extern crate serde_derive;

extern crate futures;
extern crate serde;
extern crate bincode;
extern crate crossbeam;
extern crate rayon_futures;

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
    use super::strategies::negamax::*;
    use super::game::*;
    use super::game::connectfour::*;
    use rayon::prelude::*;
    use test::Bencher;

    // #[bench]
    // fn bench_negamax(b: &mut Bencher) {
    //     b.iter(|| {
    //         let mut strategy = Negamax::create(NegamaxParams {
    //             max_depth: 3,
    //             trials: 5,
    //         });
    //         strategy.decide(&ConnectFour::new(&Color::R));
    //     });
    // }
}
