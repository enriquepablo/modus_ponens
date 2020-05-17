
extern crate modus_ponens;
#[macro_use]
extern crate modus_ponens_derive;

extern crate pest;
#[macro_use]
extern crate pest_derive;


use crate::modus_ponens::kbase::KBGen;
use crate::modus_ponens::kbase::KBase;

mod kb;


fn main() {
    env_logger::init();
    let kb = kb::KBGenerator::gen_kb();
    kb.tell("<X0> ⊆ <X1> ∧ <X1> ⊆ <X2> → <X0> ⊆ <X2>.\
             <X0> ∈ <X1> ∧ <X1> ⊆ <X2> → <X0> ∈ <X2>.\
             human ⊆ primate.\
             primate ⊆ animal.\
             susan ∈ human.");
}
