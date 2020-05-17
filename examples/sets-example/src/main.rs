
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
    let kb = kb::SetGen::gen_kb();
    kb.tell("<X0> ISA <X1>; <X1> IS <X2> -> <X0> ISA <X2>.\
        <X0> IS <X1>; <X1> IS <X2> -> <X0> IS <X2>.\
        animal IS thing.\
        mammal IS animal.\
        primate IS mammal.\
        human IS primate.\
        thing0 ISA thing.\
        animal1 ISA animal.\
        mammal2 ISA mammal.\
        primate3 ISA primate.\
        human4 ISA human.\
        thing5 ISA thing.\
        animal6 ISA animal.\
        mammal7 ISA mammal.\
        primate8 ISA primate.\
        human9 ISA human.\
        thing10 ISA thing.\
        animal11 ISA animal.\
        mammal12 ISA mammal.\
        primate13 ISA primate.\
        human14 ISA human.\
        thing15 ISA thing.\
        animal16 ISA animal.\
        mammal17 ISA mammal.\
        primate18 ISA primate.\
        human19 ISA human.\
        thing20 ISA thing.\
        animal21 ISA animal.\
        mammal22 ISA mammal.\
        primate23 ISA primate.\
        human24 ISA human.\
        thing25 ISA thing.\
        animal26 ISA animal.\
        mammal27 ISA mammal.\
        primate28 ISA primate.\
        human29 ISA human.\
        thing30 ISA thing.\
        animal31 ISA animal.\
        mammal32 ISA mammal.\
        primate33 ISA primate.\
        human34 ISA human.\
        thing35 ISA thing.\
        animal36 ISA animal.\
        mammal37 ISA mammal.\
        primate38 ISA primate.\
        human39 ISA human.\
        thing40 ISA thing.\
        animal41 ISA animal.\
        mammal42 ISA mammal.\
        primate43 ISA primate.\
        human44 ISA human.\
        thing45 ISA thing.\
        animal46 ISA animal.\
        mammal47 ISA mammal.\
        primate48 ISA primate.\
        human49 ISA human.\
        thing50 ISA thing.\
        animal51 ISA animal.\
        mammal52 ISA mammal.\
        primate53 ISA primate.\
        human54 ISA human.\
        thing55 ISA thing.\
        animal56 ISA animal.\
        mammal57 ISA mammal.\
        primate58 ISA primate.\
        human59 ISA human.\
        thing60 ISA thing.\
        animal61 ISA animal.\
        mammal62 ISA mammal.\
        primate63 ISA primate.\
        human64 ISA human.\
        thing65 ISA thing.\
        animal66 ISA animal.\
        mammal67 ISA mammal.\
        primate68 ISA primate.\
        human69 ISA human.\
        thing70 ISA thing.\
        animal71 ISA animal.\
        mammal72 ISA mammal.\
        primate73 ISA primate.\
        human74 ISA human.\
        thing75 ISA thing.\
        animal76 ISA animal.\
        mammal77 ISA mammal.\
        primate78 ISA primate.\
        human79 ISA human.\
        thing80 ISA thing.\
        animal81 ISA animal.\
        mammal82 ISA mammal.\
        primate83 ISA primate.\
        human84 ISA human.\
        thing85 ISA thing.\
        animal86 ISA animal.\
        mammal87 ISA mammal.\
        primate88 ISA primate.\
        human89 ISA human.\
        thing90 ISA thing.\
        animal91 ISA animal.\
        mammal92 ISA mammal.\
        primate93 ISA primate.\
        human94 ISA human.\
        thing95 ISA thing.\
        animal96 ISA animal.\
        mammal97 ISA mammal.\
        primate98 ISA primate.\
        human99 ISA human.");
}
