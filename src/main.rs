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

fn main() {
    println!("Hello, world!");
}
