// Copyright (c) 2020 by Enrique Pérez Arnaud <enrique at cazalla.net>    
//    
// This file is part of the modus_ponens project.    
// http://www.modus_ponens.net    
//    
// The modus_ponens project is free software: you can redistribute it and/or modify    
// it under the terms of the GNU General Public License as published by    
// the Free Software Foundation, either version 3 of the License, or    
// (at your option) any later version.    
//    
// The modus_ponens project is distributed in the hope that it will be useful,    
// but WITHOUT ANY WARRANTY; without even the implied warranty of    
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the    
// GNU General Public License for more details.    
//    
// You should have received a copy of the GNU General Public License    
// along with any part of the modus_ponens project.    
// If not, see <http://www.gnu.org/licenses/>.

extern crate modus_ponens;
#[macro_use]
extern crate modus_ponens_derive;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::mem;
use structopt::StructOpt;
//use std::{thread, time};


use crate::modus_ponens::kbase::KBGen;
use crate::modus_ponens::kbase::KBase;

mod kb;


#[derive(Debug, StructOpt)]
#[structopt(name = "fib-linear", about = "fibonacci.")]
struct Opt {
    /// number
    #[structopt(short, long)]
    n: usize,
}

fn main() {
    env_logger::init();
    let kb = kb::KBGenerator::gen_kb();
    let opt = Opt::from_args();
    kb.tell("

        q <N> 
            →
        fib <Fst> <Val1>
        {={
            <Snd> n= <Fst> + 1
        }=} {?{
            <Snd> < <N>
        }?}
            →
        fib <Snd> <Val2>
        {={
            <Nxt> n= <Snd> + 1 ∧
            <NxtVal> n= <Val1> + <Val2>
        }=}
            →
        fib <Nxt> <NxtVal> ◊

        q <N>
            →
        fib <N> <Val>
            →
        {<{ fib <N> <Val> }>} ◊

    ");
    let query = format!("q {} ◊", opt.n);
    kb.tell( unsafe { mem::transmute( query.as_str() ) });

    kb.tell("fib 0 0 ◊");
    kb.tell("fib 1 1 ◊");
}
