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
use std::{thread, time};


use crate::modus_ponens::kbase::KBGen;
use crate::modus_ponens::kbase::KBase;

mod kb;


#[derive(Debug, StructOpt)]
#[structopt(name = "isa-benchmark", about = "Benchmarking modus_ponens.")]
struct Opt {
    /// number of facts
    #[structopt(short, long)]
    facts: usize,

    /// number of rules
    #[structopt(short, long)]
    rules: usize,

    /// number of samples per batch
    #[structopt(short, long)]
    samples: usize,

    /// take batch of samples every so many rules
    #[structopt(short, long)]
    report: usize,
}

fn main() {
    let opt = Opt::from_args();
    env_logger::init();
    let kb = kb::IsaGen::gen_kb();

    let sets = ["thing", "animal", "mammal", "primate", "human"];
    let nsets = 5;
        
    let num_r = 2;
    let num_f = opt.facts + 4;

    let o_one_sec = time::Duration::from_millis(100);
    let t0 = SystemTime::now();
    let mut start = 0;
    
    for r in 0..opt.rules {
        start += 1;
        let f1 = format!("<X0> ISA{start} <X1>; <X1> IS{start} <X2> -> <X0> ISA{start} <X2>.", start = start);
        kb.tell( unsafe { mem::transmute( f1.as_str() ) });
        let f2 = format!("<X0> IS{start} <X1>; <X1> IS{start} <X2> -> <X0> IS{start} <X2>.", start = start);
        kb.tell( unsafe { mem::transmute( f2.as_str() ) });
        let f3 = format!("animal IS{start} thing.", start = start);
        kb.tell( unsafe { mem::transmute( f3.as_str() ) });
        let f4 = format!("mammal IS{start} animal.", start = start);
        kb.tell( unsafe { mem::transmute( f4.as_str() ) });
        let f5 = format!("primate IS{start} mammal.", start = start);
        kb.tell( unsafe { mem::transmute( f5.as_str() ) });
        let f6 = format!("human IS{start} primate.", start = start);
        kb.tell( unsafe { mem::transmute( f6.as_str() ) });
        for i in 0..opt.facts {
            let s = sets[(i % nsets) as usize];
            let name = format!("{}{}{}", s, i, start);
            let f = Box::leak(Box::new(format!("{name} ISA{start} {s}.", name = name, start = start, s = s)));
            kb.tell( unsafe { mem::transmute( f.as_str() ) });
        }
        if ((r % opt.report) == 0) || (r + 1 == opt.rules) {
            for _s in 0..opt.samples {
                thread::sleep(o_one_sec);

                let mut t_rs = vec![];
                let mut t_fs = vec![];

                for _x in 0..20 {
                    start += 1;

                    let t0 = SystemTime::now();

                    let f1 = format!("<X0> ISA{start} <X1>; <X1> IS{start} <X2> -> <X0> ISA{start} <X2>.", start = start);
                    kb.tell( unsafe { mem::transmute( f1.as_str() ) });
                    let f2 = format!("<X0> IS{start} <X1>; <X1> IS{start} <X2> -> <X0> IS{start} <X2>.", start = start);
                    kb.tell( unsafe { mem::transmute( f2.as_str() ) });

                    let t1 = SystemTime::now();

                    let f3 = format!("animal IS{start} thing.", start = start);
                    kb.tell( unsafe { mem::transmute( f3.as_str() ) });
                    let f4 = format!("mammal IS{start} animal.", start = start);
                    kb.tell( unsafe { mem::transmute( f4.as_str() ) });
                    let f5 = format!("primate IS{start} mammal.", start = start);
                    kb.tell( unsafe { mem::transmute( f5.as_str() ) });
                    let f6 = format!("human IS{start} primate.", start = start);
                    kb.tell( unsafe { mem::transmute( f6.as_str() ) });
                    for i in 0..opt.facts {
                        let s = sets[(i % nsets) as usize];
                        let name = format!("{}{}{}", s, i, start);
                        let f = Box::leak(Box::new(format!("{name} ISA{start} {s}.", name = name, start = start, s = s)));
                        kb.tell( unsafe { mem::transmute( f.as_str() ) });
                    }
                    let t2 = SystemTime::now();

                    t_rs.push( t1.duration_since(t0).unwrap().as_micros() as f64 );
                    t_fs.push( t2.duration_since(t1).unwrap().as_micros() as f64 );
                }

                
                let t_r_micros: f64 = t_rs.iter().sum::<f64>() / t_rs.len() as f64;
                let t_f_micros: f64 = t_fs.iter().sum::<f64>() / t_fs.len() as f64;
                
                let t_r_1 = t_r_micros / (1000.0 * num_r as f64);    
                let t_f_1 = t_f_micros / (1000.0 * num_f as f64);    
                println!("{} {} {} {}", num_r * start, t_r_1, num_f * start, t_f_1);
            }
        }
    }
    let t1 = SystemTime::now();
    let total_time = t1.duration_since(t0).unwrap().as_micros() as f64 / 1000000.0;

    println!("total time: {}", total_time);

}
