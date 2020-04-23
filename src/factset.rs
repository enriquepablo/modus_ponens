use std::collections::HashMap;

use crate::matching::SynMatching;
use crate::fact::Fact;
use crate::facttree::FSNode;


pub struct FactSet<'a> {
    pub root: Box<FSNode<'a>>,
}


impl<'a> FactSet<'a> {
    pub fn new () -> FactSet<'a> {
        FactSet { root: Box::new(FSNode::new()) }
    }
    pub fn add_fact (&'a mut self, fact: &'a Fact<'a>) {
        let mut zipper = self.root.zipper();
        let paths = fact.paths.as_slice();
        zipper = zipper.follow_and_create_paths(paths);
    }
    pub fn ask_fact (&'a self, fact: &'a Fact) -> Vec<SynMatching<'a>> {
        let response: Vec<SynMatching<'a>> = vec![];
        let mut qzipper = self.root.qzipper(response);
        let paths = fact.paths.as_slice();
        let matching: SynMatching = HashMap::new();
        qzipper = qzipper.query_paths(paths, matching);
        qzipper.finish()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::parser;

    pub struct Knowledge<'a> {
        pub factset: FactSet<'a>,
    }


    impl<'a> Knowledge<'a> {
        pub fn new () -> Knowledge<'a> {
            Knowledge {
                factset: FactSet::new(),
            }
        }
        fn tell(&'a mut self, grammar: &'a parser::Grammar<'a>, k: &'a str) {
            let parsed = grammar.parse_text(k);
            let facts = parsed.ok().unwrap().facts;
            for fact in facts {
                self.factset.add_fact(&fact);
            }
        }
        fn ask(&'a self, grammar: &'a parser::Grammar<'a>, q: &'static str) -> bool {
            let parsed = grammar.parse_text(q);
            let mut facts = parsed.ok().unwrap().facts;
            let fact = facts.pop().unwrap();
            let resp = self.factset.ask_fact(&fact);
            resp.len() > 0
        }
    }

    #[test]
    fn test_1() {
        let mut kb = Knowledge::new();
        let grammar = parser::Grammar::new();
        kb.tell(&grammar, "susan ISA person. john ISA person.");
        let resp1 = kb.ask(&grammar, "susan ISA person.");
        assert_eq!(resp1, true);
        let resp2 = kb.ask(&grammar, "pepe ISA person.");
        assert_eq!(resp2, false);
        let resp3 = kb.ask(&grammar, "john ISA person.");
        assert_eq!(resp3, true);
        let resp4 = kb.ask(&grammar, "<X0> ISA person.");
        assert_eq!(resp4, true);
        let resp5 = kb.ask(&grammar, "<X0> ISA animal.");
        assert_eq!(resp5, false);
    }
//    #[test]
//    fn test_factset_grammar_1() {
//        let mut kb = Knowledge::new();
//        let grammar = parser::Grammar::new();
//        kb = kb.tell(&grammar, "\
//            susan ISA person.\
//            john ISA person.\
//            person IS animal.\
//            (say: susan, what: (want: susan, what: (love: john, who: susan))) ISA fact.\
//            (want: john, what: (love: john, who: susan)) ISA fact.\
//            (love: susan, who: john) ISA fact.");
//        let (kb, resp) = kb.ask(&grammar, "susan ISA person.");
//        assert_eq!(resp, true);
//        let (kb, resp) = kb.ask(&grammar, "pepe ISA person.");
//        assert_eq!(resp, false);
//        let (kb, resp) = kb.ask(&grammar, "john ISA person.");
//        assert_eq!(resp, true);
//        let (kb, resp) = kb.ask(&grammar, "<X0> ISA person.");
//        assert_eq!(resp, true);
//        let (kb, resp) = kb.ask(&grammar, "<X0> ISA animal.");
//        assert_eq!(resp, false);
//        let (kb, resp) = kb.ask(&grammar, "<X0> IS animal.");
//        assert_eq!(resp, true);
//        let (kb, resp) = kb.ask(&grammar, "(say: susan, what: (want: susan, what: <X0>)) ISA fact.");
//        assert_eq!(resp, true);
//        let (kb, resp) = kb.ask(&grammar, "(say: susan, what: (want: susan, what: (love: <X0>, who: susan))) ISA fact.");
//        assert_eq!(resp, true);
//        let (kb, resp) = kb.ask(&grammar, "(say: <X1>, what: (want: <X1>, what: (love: <X0>, who: <X1>))) ISA fact.");
//        assert_eq!(resp, true);
//        let (kb, resp) = kb.ask(&grammar, "(say: <X1>, want: (what: <X1>, what: (love: <X0>, who: <X1>))) ISA fact.");
//        assert_eq!(resp, false);
//        let (kb, resp) = kb.ask(&grammar, "(say: <X1>, what: (want: <X1>, what: (love: <X1>, who: <X1>))) ISA fact.");
//        assert_eq!(resp, false);
//        let (kb, resp) = kb.ask(&grammar, "(say: <X1>, what: (want: <X1>, what: <X1>)) ISA fact.");
//        assert_eq!(resp, false);
//        let (kb, resp) = kb.ask(&grammar, "(say: john, what: (want: susan, what: <X0>)) ISA fact.");
//        assert_eq!(resp, false);
//        let (kb, resp) = kb.ask(&grammar, "(want: <X0>, what: (love: <X0>, who: <X1>)) ISA fact.");
//        assert_eq!(resp, true);
//        let (kb, resp) = kb.ask(&grammar, "(want: <X0>, what: (love: <X0>, who: <X0>)) ISA fact.");
//        assert_eq!(resp, false);
//        let (kb, resp) = kb.ask(&grammar, "(say: susan, what: <X0>) ISA fact.");
//        assert_eq!(resp, true);
//        let (kb, resp) = kb.ask(&grammar, "(want: susan, what: <X0>) ISA fact.");
//        assert_eq!(resp, false);
//        let (kb, resp) = kb.ask(&grammar, "(want: john, what: <X0>) ISA fact.");
//        assert_eq!(resp, true);
//        let (kb, resp) = kb.ask(&grammar, "(love: john, who: susan) ISA fact.");
//        assert_eq!(resp, false);
//        let (_, resp) = kb.ask(&grammar, "(love: susan, who: john) ISA fact.");
//        assert_eq!(resp, true);
//    }
//    #[test]
//    fn test_factset_grammar_2() {
//        let mut kb = Knowledge::new();
//        let grammar = parser::Grammar::new();
//        kb = kb.tell(&grammar, "\
//            susan ISA person.\
//            john ISA person.\
//            person IS animal.\
//            (say: susan, what: (want: susan, what: (love: john, who: susan))) ISA fact.\
//            (want: john, what: (love: john, who: susan)) ISA fact.\
//            (love: susan, who: john) ISA fact.");
//        let (_, resp) = kb.ask(&grammar, "(say: <X1>, what: (want: <X1>, what: (love: john, who: <X1>))) ISA fact.");
//        assert_eq!(resp, true);
//    }
//    #[test]
//    fn test_3() {
//        let mut kb = Knowledge::new();
//        let grammar = parser::Grammar::new();
//        kb = kb.tell(&grammar, "(p1: (s: (s: 0)), p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
//        let (kb, resp1) = kb.ask(&grammar, "(p1: (s: (s: 0)), p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
//        assert!(resp1);
//        let (kb, resp2) = kb.ask(&grammar, "(p1: (s: <X0>), p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
//        assert!(resp2);
//        let (_, resp3) = kb.ask(&grammar, "(p1: <X0>, p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
//        assert!(resp3);
//    }
//    #[test]
//    fn test_fs_4() {
//        let mut kb = Knowledge::new();
//        let grammar = parser::Grammar::new();
//        kb = kb.tell(&grammar, "(p1: (s: 0), p2: john) ISA (fn: pr, on: (p1: nat, p2: people)).");
//        kb = kb.tell(&grammar, "(p1: (s: (s: 0)), p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
//        let (_, resp2) = kb.ask(&grammar, "(p1: <X0>, p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
//        assert!(resp2);
//    }
//    #[test]
//    fn test_fs_5() {
//        let mut kb = Knowledge::new();
//        let grammar = parser::Grammar::new();
//        kb = kb.tell(&grammar, "(p1: (s: 0), p2: john) ISA fact.");
//        kb = kb.tell(&grammar, "(p1: (s: (s: 0)), p2: susan) ISA fact.");
//        let (_, resp2) = kb.ask(&grammar, "(p1: <X0>, p2: susan) ISA fact.");
//        assert!(resp2);
//    }
}