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
    let mut kb = knowledge::KnowledgeBase::new();
    kb = kb.tell("(p1: <X4>, p2: <X5>) ISA (hom1: (p1: <X2>, p2: <X3>), hom2: (p1: <X2>, p2: <X3>))\
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
    kb = kb.tell("(p1: <X2>, p2: <X3>) ISA (fn: <X1>, on: (p1: <X4>, p2: <X5>))\
                 -> \
                 <X2> ISA (fn: <X1>, on: <X4>);\
                 <X3> ISA (fn: <X1>, on: <X5>).");
    kb = kb.tell("<X1> ISA (fn: pr, on: nat)\
                 -> \
                 (fn: (fn: pr, on: s1), on: <X1>) EQ (s: <X1>).");
    kb = kb.tell("s2 ISA (hom1: people, hom2: people).");
    kb = kb.tell("(p1: s1, p2: s2) ISA (hom1: (p1: nat, p2: people), hom2: (p1: nat, p2: people)).");
    kb = kb.tell("s1 ISA (hom1: nat, hom2: nat).");
    kb = kb.tell("(p1: (s: 0), p2: john) ISA (fn: pr, on: (p1: nat, p2: people)).");
    kb = kb.tell("john ISA (fn: pr, on: people).\
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
    kb = kb.tell("(fn: (fn: pr, on: s2), on: john) EQ susan.\
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
