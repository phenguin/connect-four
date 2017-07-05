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
