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
use std::time::SystemTime;
use structopt::StructOpt;
//use std::{thread, time};


use crate::modus_ponens::kbase::KBGen;
use crate::modus_ponens::kbase::KBase;

mod kb;


#[derive(Debug, StructOpt)]
#[structopt(name = "socrate-benchmark", about = "Benchmarking modus_ponens.")]
struct Opt {
    /// depth of implications
    #[structopt(short, long)]
    depth: usize,

    /// width of implications
    #[structopt(short, long)]
    number: usize,

    /// amount of garbage
    #[structopt(short, long)]
    garbage: usize,

    /// amount of related garbage
    #[structopt(short, long)]
    rgarbage: usize,
}

fn main() {
    let opt = Opt::from_args();
    env_logger::init();
    let kb = kb::IsaGen::gen_kb();
    let t_0 = SystemTime::now();
        
    //let mut num_rules = 0;
    //let mut num_facts = 0;

    //let o_one_sec = time::Duration::from_millis(100);
    //let t0 = SystemTime::now();
    kb.tell("animal <X1> -> living <X1> -> mortal <X1> ◊");
    let mut animal = String::from("animal");
    let mut living = String::from("living");
    
    for d in 0..opt.depth {
        let animal_next = format!("animal{d}", d=d);
        let living_next = format!("living{d}", d=d);

        let f1 = format!("{animal_next} <X1> -> {animal} <X1> ◊", animal=animal, animal_next=animal_next);
        kb.tell( unsafe { mem::transmute( f1.as_str() ) });

        let f2 = format!("{living_next} <X1> -> {living} <X1> ◊", living=living, living_next=living_next);
        kb.tell( unsafe { mem::transmute( f2.as_str() ) });

        for g in 0..opt.garbage {
            let thingy = format!("thing{d}n{g}", d=d, g=g);
            let thongy = format!("thong{d}n{g}", d=d, g=g);

            let f1 = format!("{thingy} <X1> -> pre{thingy} <X1> ◊", thingy=thingy);
            kb.tell( unsafe { mem::transmute( f1.as_str() ) });

            let f2 = format!("{thingy} lattle{thingy} ◊", thingy=thingy);
            kb.tell( unsafe { mem::transmute( f2.as_str() ) });

            let f3 = format!("{thongy} <X1> -> pre{thongy} <X1> ◊", thongy=thongy);
            kb.tell( unsafe { mem::transmute( f3.as_str() ) });

            let f4 = format!("{thongy} lattle{thingy} ◊", thongy=thongy, thingy=thingy);
            kb.tell( unsafe { mem::transmute( f4.as_str() ) });
        }

        for g in 0..opt.rgarbage {
            let thingy = format!("thing{d}n{g}", d=d, g=g);
            let thongy = format!("thong{d}n{g}", d=d, g=g);

            let f1 = format!("{thingy} <X1> -> {animal} <X1> ◊", thingy=thingy, animal=animal);
            kb.tell( unsafe { mem::transmute( f1.as_str() ) });

            let f2 = format!("{thingy} little{thingy} ◊", thingy=thingy);
            kb.tell( unsafe { mem::transmute( f2.as_str() ) });

            let f3 = format!("{thongy} <X1> -> {living} <X1> ◊", thongy=thongy, living=living);
            kb.tell( unsafe { mem::transmute( f3.as_str() ) });

            let f4 = format!("{thongy} little{thongy} ◊", thongy=thongy);
            kb.tell( unsafe { mem::transmute( f4.as_str() ) });
        }
        animal = String::from(animal_next);
        living = String::from(living_next);
    }

    for n in 0..opt.number {
        let mortal = format!("mortal{n}", n=n);

        let f1 = format!("{animal} {mortal} ◊", animal=animal, mortal=mortal);
        kb.tell( unsafe { mem::transmute( f1.as_str() ) });

        let f2 = format!("{living} {mortal} ◊", living=living, mortal=mortal);
        kb.tell( unsafe { mem::transmute( f2.as_str() ) });
    }

    let t_1 = SystemTime::now();
    let res = kb.ask("mortal <X1> ◊");
    let num_results = res.len();
    let t_2 = SystemTime::now();

    println!("{:#?}", res);

    let query_time = t_2.duration_since(t_1).unwrap().as_micros() as f64;
    let total_time = t_2.duration_since(t_0).unwrap().as_micros() as f64;

    println!("total: {}, query: {}, results: {}", total_time, query_time, num_results);
}
