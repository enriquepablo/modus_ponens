
extern crate modus_ponens;
#[macro_use]
extern crate modus_ponens_derive;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use std::mem;
use structopt::StructOpt;

use crate::modus_ponens::kbase::KBGen;
use crate::modus_ponens::kbase::KBase;

mod kb;


#[derive(Debug, StructOpt)]
#[structopt(name = "isa-benchmark", about = "Benchmarking modus_ponens.")]
struct Opt {
    /// number of facts
    #[structopt(short, long)]
    facts: i32,

    /// number of rules
    #[structopt(short, long)]
    rules: i32,
}

fn main() {
    let opt = Opt::from_args();
    env_logger::init();
    let kb = kb::IsaGen::gen_kb();

    let sets = ["thing", "animal", "mammal", "primate", "human"];
    let nsets = 5;

    for r in 0..opt.rules {
        let f1 = format!("<X0> ISA{r} <X1>; <X1> IS{r} <X2> -> <X0> ISA{r} <X2>.", r = r);
        kb.tell( unsafe { mem::transmute( f1.as_str() ) });
        let f2 = format!("<X0> IS{r} <X1>; <X1> IS{r} <X2> -> <X0> IS{r} <X2>.", r = r);
        kb.tell( unsafe { mem::transmute( f2.as_str() ) });
        let f3 = format!("animal IS{r} thing.", r = r);
        kb.tell( unsafe { mem::transmute( f3.as_str() ) });
        let f4 = format!("mammal IS{r} animal.", r = r);
        kb.tell( unsafe { mem::transmute( f4.as_str() ) });
        let f5 = format!("primate IS{r} mammal.", r = r);
        kb.tell( unsafe { mem::transmute( f5.as_str() ) });
        let f6 = format!("human IS{r} primate.", r = r);
        kb.tell( unsafe { mem::transmute( f6.as_str() ) });
    }
    for r in 0..opt.rules {
        for i in 0..opt.facts {
            let s = sets[(i % nsets) as usize];
            let name = format!("{}{}{}", s, i, r);
            let f = Box::leak(Box::new(format!("{name} ISA{r} {s}.", name = name, r = r, s = s)));
            kb.tell( unsafe { mem::transmute( f.as_str() ) });
        }
    }
}
