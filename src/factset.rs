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
    pub fn add_fact (mut self, fact: &'a Fact<'a>) -> FactSet<'a> {
        let mut zipper = self.root.zipper();
        let paths = fact.get_all_paths();
        zipper = zipper.follow_and_create_paths(&paths);
        self.root = zipper.finish();
        self
    }
    pub fn ask_fact (&'a self, fact: &'a Fact) -> &'a [SynMatching] {
        let mut response: &'a mut [SynMatching<'a>] = &mut [];
        let mut qzipper = self.root.qzipper(response);
        let paths = fact.get_leaf_paths();
        let matching: SynMatching = HashMap::new();
        qzipper = qzipper.query_paths(paths, matching);
        response = qzipper.finish();
        response
    }
    pub fn ask_fact_slice (&'a self, fact: &'a Fact) -> &'a [SynMatching<'a>] {
        let mut response: &'a mut [SynMatching<'a>] = &mut [];
        let mut qzipper = self.root.qzipper(response);
        let paths = fact.get_leaf_paths();
        let matching: SynMatching = HashMap::new();
        qzipper = qzipper.query_paths(paths, matching);
        response = qzipper.finish();
        response
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
        fn tell(mut self, grammar: &'a parser::Grammar<'a>, k: &'a str) -> Knowledge<'a> {
            let parsed = grammar.parse_text(k);
            let facts = parsed.ok().unwrap().facts;
            for fact in facts {
                self.factset = self.factset.add_fact(&fact);
            }
            self
        }
        fn ask(&'a self, grammar: &'a parser::Grammar<'a>, q: &'static str) -> bool {
            let parsed = grammar.parse_text(q);
            let mut facts = parsed.ok().unwrap().facts;
            let fact = facts.pop().unwrap();
            let response = self.factset.ask_fact(&fact);
            response.len() > 0
        }
    }

    #[test]
    fn test_1() {
        let mut kb = Knowledge::new();
        let grammar = parser::Grammar::new();
        kb = kb.tell(&grammar, "susan ISA person. john ISA person.");
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
    #[test]
    fn test_2() {
        let mut kb = Knowledge::new();
        let grammar = parser::Grammar::new();
        kb = kb.tell(&grammar, "\
            susan ISA person.\
            john ISA person.\
            person IS animal.\
            (say: susan, what: (want: susan, what: (love: john, who: susan))) ISA fact.\
            (want: john, what: (love: john, who: susan)) ISA fact.\
            (love: susan, who: john) ISA fact.");
        let mut resp = kb.ask(&grammar, "susan ISA person.");
        assert_eq!(resp, true);
        resp = kb.ask(&grammar, "pepe ISA person.");
        assert_eq!(resp, false);
        resp = kb.ask(&grammar, "john ISA person.");
        assert_eq!(resp, true);
        resp = kb.ask(&grammar, "<X0> ISA person.");
        assert_eq!(resp, true);
        resp = kb.ask(&grammar, "<X0> ISA animal.");
        assert_eq!(resp, false);
        resp = kb.ask(&grammar, "<X0> IS animal.");
        assert_eq!(resp, true);
        resp = kb.ask(&grammar, "(say: susan, what: (want: susan, what: <X0>)) ISA fact.");
        assert_eq!(resp, true);
        resp = kb.ask(&grammar, "(say: susan, what: (want: susan, what: (love: <X0>, who: susan))) ISA fact.");
        assert_eq!(resp, true);
        resp = kb.ask(&grammar, "(say: <X1>, what: (want: <X1>, what: (love: <X0>, who: <X1>))) ISA fact.");
        assert_eq!(resp, true);
        resp = kb.ask(&grammar, "(say: <X1>, want: (what: <X1>, what: (love: <X0>, who: <X1>))) ISA fact.");
        assert_eq!(resp, false);
        resp = kb.ask(&grammar, "(say: <X1>, what: (want: <X1>, what: (love: <X1>, who: <X1>))) ISA fact.");
        assert_eq!(resp, false);
        resp = kb.ask(&grammar, "(say: <X1>, what: (want: <X1>, what: <X1>)) ISA fact.");
        assert_eq!(resp, false);
        resp = kb.ask(&grammar, "(say: john, what: (want: susan, what: <X0>)) ISA fact.");
        assert_eq!(resp, false);
        resp = kb.ask(&grammar, "(want: <X0>, what: (love: <X0>, who: <X1>)) ISA fact.");
        assert_eq!(resp, true);
        resp = kb.ask(&grammar, "(want: <X0>, what: (love: <X0>, who: <X0>)) ISA fact.");
        assert_eq!(resp, false);
        resp = kb.ask(&grammar, "(say: susan, what: <X0>) ISA fact.");
        assert_eq!(resp, true);
        resp = kb.ask(&grammar, "(want: susan, what: <X0>) ISA fact.");
        assert_eq!(resp, false);
        resp = kb.ask(&grammar, "(want: john, what: <X0>) ISA fact.");
        assert_eq!(resp, true);
        resp = kb.ask(&grammar, "(love: john, who: susan) ISA fact.");
        assert_eq!(resp, false);
        resp = kb.ask(&grammar, "(love: susan, who: john) ISA fact.");
        assert_eq!(resp, true);
    }
    #[test]
    fn test_3() {
        let mut kb = Knowledge::new();
        let grammar = parser::Grammar::new();
        kb = kb.tell(&grammar, "(p1: (s: (s: 0)), p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
        let resp1 = kb.ask(&grammar, "(p1: (s: (s: 0)), p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
        assert!(resp1);
        let resp2 = kb.ask(&grammar, "(p1: (s: <X0>), p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
        assert!(resp2);
        let resp3 = kb.ask(&grammar, "(p1: <X0>, p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
        assert!(resp3);
    }
    #[test]
    fn test_fs_4() {
        let mut kb = Knowledge::new();
        let grammar = parser::Grammar::new();
        kb = kb.tell(&grammar, "(p1: (s: 0), p2: john) ISA (fn: pr, on: (p1: nat, p2: people)).");
        kb = kb.tell(&grammar, "(p1: (s: (s: 0)), p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
        let resp2 = kb.ask(&grammar, "(p1: <X0>, p2: susan) ISA (fn: pr, on: (p1: nat, p2: people)).");
        assert!(resp2);
    }
    #[test]
    fn test_fs_5() {
        let mut kb = Knowledge::new();
        let grammar = parser::Grammar::new();
        kb = kb.tell(&grammar, "(p1: (s: 0), p2: john) ISA fact.");
        kb = kb.tell(&grammar, "(p1: (s: (s: 0)), p2: susan) ISA fact.");
        let resp2 = kb.ask(&grammar, "(p1: <X0>, p2: susan) ISA fact.");
        assert!(resp2);
    }
}











































































































































































