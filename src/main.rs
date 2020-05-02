#![feature(hash_set_entry)]
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
//mod factset;
mod ruletree;
mod knowledge;
mod lexicon;

fn main() {
     let grammar = parser::Grammar::new();
     let kb = knowledge::KnowledgeBase::new(&grammar);
     kb.tell("(p1: <X4>, p2: <X5>) ISA (hom1: (p1: <X2>, p2: <X3>), hom2: (p1: <X2>, p2: <X3>))\
             -> \
             <X4> ISA (hom1: <X2>, hom2: <X2>);\
             <X5> ISA (hom1: <X3>, hom2: <X3>)\
             -> \
             (p1: <X6>, p2: <X8>) ISA (fn: <X1>, on: (p1: <X2>, p2: <X3>))\
             -> \
             (fn: (fn: <X1>, on: <X4>), on: <X6>) EQ <X7>;\
             (fn: (fn: <X1>, on: <X5>), on: <X8>) EQ <X9>\
              -> \
             (p1: <X7>, p2: <X9>) ISA (fn: <X1>, on: (p1: <X2>, p2: <X3>));\
             (fn: (fn: <X1>, on: (p1: <X4>, p2: <X5>)), on: (p1: <X6>, p2: <X8>)) EQ (p1: <X7>, p2: <X9>).");
     kb.tell("(p1: <X2>, p2: <X3>) ISA (fn: <X1>, on: (p1: <X4>, p2: <X5>))\
            -> \
            <X2> ISA (fn: <X1>, on: <X4>);\
            <X3> ISA (fn: <X1>, on: <X5>).");
     kb.tell("<X1> ISA (fn: pr, on: nat)\
            -> \
            (fn: (fn: pr, on: s1), on: <X1>) EQ (s: <X1>).");
     kb.tell("s2 ISA (hom1: people, hom2: people).");
     kb.tell("(p1: s1, p2: s2) ISA (hom1: (p1: nat, p2: people), hom2: (p1: nat, p2: people)).");
     kb.tell("s1 ISA (hom1: nat, hom2: nat).");
     kb.tell("(p1: (s: 0), p2: john) ISA (fn: pr, on: (p1: nat, p2: people)).");
     kb.tell("john ISA (fn: pr, on: people).\
                   susan ISA (fn: pr, on: people).\
                   sue1 ISA (fn: pr, on: people).\
                   sue2 ISA (fn: pr, on: people).\
                   sue3 ISA (fn: pr, on: people).\
                   sue4 ISA (fn: pr, on: people).\
                   sue5 ISA (fn: pr, on: people).\
                   sue6 ISA (fn: pr, on: people).\
                   sue7 ISA (fn: pr, on: people).\
                   sue8 ISA (fn: pr, on: people).\
                   sue9 ISA (fn: pr, on: people).\
                   sue10 ISA (fn: pr, on: people).\
                   sue11 ISA (fn: pr, on: people).\
                   sue12 ISA (fn: pr, on: people).\
                   sue13 ISA (fn: pr, on: people).\
                   sue14 ISA (fn: pr, on: people).\
                   sue15 ISA (fn: pr, on: people).\
                   sue16 ISA (fn: pr, on: people).\
                   sue17 ISA (fn: pr, on: people).\
                   sue18 ISA (fn: pr, on: people).\
                   sue19 ISA (fn: pr, on: people).\
                   ken ISA (fn: pr, on: people).\
                   bob ISA (fn: pr, on: people).\
                   isa ISA (fn: pr, on: people).\
                   peter ISA (fn: pr, on: people).");
     kb.tell("(fn: (fn: pr, on: s2), on: john) EQ susan.\
                  (fn: (fn: pr, on: s2), on: susan) EQ sue1.\
                  (fn: (fn: pr, on: s2), on: sue1) EQ sue2.\
                  (fn: (fn: pr, on: s2), on: sue2) EQ sue3.\
                  (fn: (fn: pr, on: s2), on: sue3) EQ sue4.\
                  (fn: (fn: pr, on: s2), on: sue4) EQ sue5.\
                  (fn: (fn: pr, on: s2), on: sue5) EQ sue6.\
                  (fn: (fn: pr, on: s2), on: sue6) EQ sue7.\
                  (fn: (fn: pr, on: s2), on: sue7) EQ sue8.\
                  (fn: (fn: pr, on: s2), on: sue8) EQ sue9.\
                  (fn: (fn: pr, on: s2), on: sue9) EQ sue10.\
                  (fn: (fn: pr, on: s2), on: sue10) EQ sue11.\
                  (fn: (fn: pr, on: s2), on: sue11) EQ sue12.\
                  (fn: (fn: pr, on: s2), on: sue12) EQ sue13.\
                  (fn: (fn: pr, on: s2), on: sue13) EQ sue14.\
                  (fn: (fn: pr, on: s2), on: sue14) EQ sue15.\
                  (fn: (fn: pr, on: s2), on: sue15) EQ sue16.\
                  (fn: (fn: pr, on: s2), on: sue16) EQ sue17.\
                  (fn: (fn: pr, on: s2), on: sue17) EQ sue18.\
                  (fn: (fn: pr, on: s2), on: sue18) EQ sue19.\
                  (fn: (fn: pr, on: s2), on: sue19) EQ ken.\
                  (fn: (fn: pr, on: s2), on: ken) EQ bob.\
                  (fn: (fn: pr, on: s2), on: bob) EQ isa.\
                  (fn: (fn: pr, on: s2), on: isa) EQ peter.");
}



//fn main() {
//     let grammar = parser::Grammar::new();
//     let kb = knowledge::KnowledgeBase::new(&grammar);
//     kb.tell("<X0> ISA <X1>; <X1> IS <X2> -> <X0> ISA <X2>.\
//<X0> IS <X1>; <X1> IS <X2> -> <X0> IS <X2>.\
//animal IS thing.\
//mammal IS animal.\
//primate IS mammal.\
//human IS primate.\
//thing0 ISA thing.\
//animal1 ISA animal.\
//mammal2 ISA mammal.\
//primate3 ISA primate.\
//human4 ISA human.\
//thing5 ISA thing.\
//animal6 ISA animal.\
//mammal7 ISA mammal.\
//primate8 ISA primate.\
//human9 ISA human.\
//thing10 ISA thing.\
//animal11 ISA animal.\
//mammal12 ISA mammal.\
//primate13 ISA primate.\
//human14 ISA human.\
//thing15 ISA thing.\
//animal16 ISA animal.\
//mammal17 ISA mammal.\
//primate18 ISA primate.\
//human19 ISA human.\
//thing20 ISA thing.\
//animal21 ISA animal.\
//mammal22 ISA mammal.\
//primate23 ISA primate.\
//human24 ISA human.\
//thing25 ISA thing.\
//animal26 ISA animal.\
//mammal27 ISA mammal.\
//primate28 ISA primate.\
//human29 ISA human.\
//thing30 ISA thing.\
//animal31 ISA animal.\
//mammal32 ISA mammal.\
//primate33 ISA primate.\
//human34 ISA human.\
//thing35 ISA thing.\
//animal36 ISA animal.\
//mammal37 ISA mammal.\
//primate38 ISA primate.\
//human39 ISA human.\
//thing40 ISA thing.\
//animal41 ISA animal.\
//mammal42 ISA mammal.\
//primate43 ISA primate.\
//human44 ISA human.\
//thing45 ISA thing.\
//animal46 ISA animal.\
//mammal47 ISA mammal.\
//primate48 ISA primate.\
//human49 ISA human.\
//thing50 ISA thing.\
//animal51 ISA animal.\
//mammal52 ISA mammal.\
//primate53 ISA primate.\
//human54 ISA human.\
//thing55 ISA thing.\
//animal56 ISA animal.\
//mammal57 ISA mammal.\
//primate58 ISA primate.\
//human59 ISA human.\
//thing60 ISA thing.\
//animal61 ISA animal.\
//mammal62 ISA mammal.\
//primate63 ISA primate.\
//human64 ISA human.\
//thing65 ISA thing.\
//animal66 ISA animal.\
//mammal67 ISA mammal.\
//primate68 ISA primate.\
//human69 ISA human.\
//thing70 ISA thing.\
//animal71 ISA animal.\
//mammal72 ISA mammal.\
//primate73 ISA primate.\
//human74 ISA human.\
//thing75 ISA thing.\
//animal76 ISA animal.\
//mammal77 ISA mammal.\
//primate78 ISA primate.\
//human79 ISA human.\
//thing80 ISA thing.\
//animal81 ISA animal.\
//mammal82 ISA mammal.\
//primate83 ISA primate.\
//human84 ISA human.\
//thing85 ISA thing.\
//animal86 ISA animal.\
//mammal87 ISA mammal.\
//primate88 ISA primate.\
//human89 ISA human.\
//thing90 ISA thing.\
//animal91 ISA animal.\
//mammal92 ISA mammal.\
//primate93 ISA primate.\
//human94 ISA human.\
//thing95 ISA thing.\
//animal96 ISA animal.\
//mammal97 ISA mammal.\
//primate98 ISA primate.\
//human99 ISA human.");