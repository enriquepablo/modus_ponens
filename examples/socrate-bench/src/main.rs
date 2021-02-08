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
        
    let num_r = 2;
    let num_f = opt.facts + 4;

    //let o_one_sec = time::Duration::from_millis(100);
    let t0 = SystemTime::now();
    let mut start = 0;
    
    for r in 0..opt.rules {
        start += 1;
        let f1 = format!("mammal{start} <X1> -> animal{start} <X1> ◊", start = start);
        kb.tell( unsafe { mem::transmute( f1.as_str() ) });
        let f2 = format!("primate{start} <X1> -> mammal{start} <X1> ◊", start = start);
        kb.tell( unsafe { mem::transmute( f2.as_str() ) });
        let f3 = format!("human{start} <X1> -> primate{start} <X1> ◊", start = start);
        kb.tell( unsafe { mem::transmute( f3.as_str() ) });
        let f4 = format!("living{start} <X1> -> animal{start} <X1> -> mortal{start} <X1> ◊", start = start);
        kb.tell( unsafe { mem::transmute( f4.as_str() ) });
        for i in 0..opt.facts {
            let name = format!("socrate{}n{}", start, i);
            let ff1 = Box::leak(Box::new(format!("human{start} {name} ◊", name = name, start = start)));
            kb.tell( unsafe { mem::transmute( ff1.as_str() ) });
            let ff2 = Box::leak(Box::new(format!("living{start} {name} ◊", name = name, start = start)));
            kb.tell( unsafe { mem::transmute( ff2.as_str() ) });
        }
        if ((r % opt.report) == 0) || (r + 1 == opt.rules) {
            for _s in 0..opt.samples {
                //thread::sleep(o_one_sec);

                let mut t_rs = vec![];
                let mut t_fs = vec![];

                for _x in 0..20 {
                    start += 1;

                    let t0 = SystemTime::now();

                    let f1 = format!("mammal{start} <X1> -> animal{start} <X1> ◊", start = start);
                    kb.tell( unsafe { mem::transmute( f1.as_str() ) });
                    let f2 = format!("primate{start} <X1> -> mammal{start} <X1> ◊", start = start);
                    kb.tell( unsafe { mem::transmute( f2.as_str() ) });
                    let f3 = format!("human{start} <X1> -> primate{start} <X1> ◊", start = start);
                    kb.tell( unsafe { mem::transmute( f3.as_str() ) });
                    let f4 = format!("living{start} <X1> -> animal{start} <X1> -> mortal{start} <X1> ◊", start = start);
                    kb.tell( unsafe { mem::transmute( f4.as_str() ) });

                    let t1 = SystemTime::now();

                    for i in 0..opt.facts {
                        let name = format!("socrate{}n{}", start, i);
                        let ff1 = Box::leak(Box::new(format!("human{start} {name} ◊", name = name, start = start)));
                        kb.tell( unsafe { mem::transmute( ff1.as_str() ) });
                        let ff2 = Box::leak(Box::new(format!("living{start} {name} ◊", name = name, start = start)));
                        kb.tell( unsafe { mem::transmute( ff2.as_str() ) });
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
