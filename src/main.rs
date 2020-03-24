#![allow(dead_code)]

extern crate pest;
#[macro_use]
extern crate pest_derive;

mod constants;
mod segment;
mod matching;
mod path;
mod fact;
mod parser;
mod facttree;
mod factset;
mod ruletree;
mod knowledge;
mod lexicon;

fn main() {
    println!("Hello, world!");
}
