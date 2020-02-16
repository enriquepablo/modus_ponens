use std::collections::HashMap;

use crate::matching::SynMatching;
use crate::fact::Fact;
use crate::facttree::FSNode;
use crate::parser::parse_text;


pub struct FactSet {
    pub root: Box<FSNode>,
}


impl<'a> FactSet {
    fn new () -> FactSet {
        FactSet { root: Box::new(FSNode::new()) }
    }
    fn add_fact (self, fact: Fact) -> FactSet {
        let FactSet { mut root } = self;
        let mut zipper = root.zipper();
        let paths = fact.get_all_paths();
        zipper = zipper.follow_and_create_paths(&paths);
        root = zipper.finish();
        FactSet { root }
    }
    fn ask_fact (&'a self, fact: &'a Fact) -> Vec<SynMatching<'a>> {
        let mut response: Box<Vec<SynMatching>> = Box::new(vec![]);
        let mut qzipper = self.root.qzipper(response);
        let paths = fact.get_leaf_paths();
        let matching: SynMatching = HashMap::new();
        qzipper = qzipper.query_paths(paths, matching);
        response = qzipper.finish();
        *response
    }
}


pub struct Knowledge {
    pub factset: FactSet,
}


impl<'a> Knowledge {
    pub fn new () -> Knowledge {
        Knowledge { factset: FactSet::new() }
    }
    fn tell(self, k: &str) -> Knowledge {
        let Knowledge {
            mut factset
        } = self;
        let parsed = parse_text(k);
        let facts = parsed.ok().unwrap().facts;
        for fact in facts {
            factset = factset.add_fact(fact);
        }
        Knowledge {
            factset
        }
    }
    fn ask(&'a self, q: &str) -> bool {
        let parsed = parse_text(q);
        let mut facts = parsed.ok().unwrap().facts;
        let fact = facts.pop().unwrap();
        let response = self.factset.ask_fact(&fact);
        response.len() > 0
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let mut kb = Knowledge::new();
        kb = kb.tell("susan ISA person. john ISA person.");
        let resp1 = kb.ask("susan ISA person.");
        assert_eq!(resp1, true);
        let resp2 = kb.ask("pepe ISA person.");
        assert_eq!(resp2, false);
        let resp3 = kb.ask("john ISA person.");
        assert_eq!(resp3, true);
        let resp4 = kb.ask("<X0> ISA person.");
        assert_eq!(resp4, true);
        let resp5 = kb.ask("<X0> ISA animal.");
        assert_eq!(resp5, false);
    }
    #[test]
    fn test_2() {
        let mut kb = Knowledge::new();
        kb = kb.tell("\
            susan ISA person.\
            john ISA person.\
            person IS animal.\
            (say: susan, what: (want: susan, what: (love: john, who: susan))) ISA fact.\
            (want: john, what: (love: john, who: susan)) ISA fact.\
            (love: susan, who: john) ISA fact.");
        let mut resp = kb.ask("susan ISA person.");
        assert_eq!(resp, true);
        resp = kb.ask("pepe ISA person.");
        assert_eq!(resp, false);
        resp = kb.ask("john ISA person.");
        assert_eq!(resp, true);
        resp = kb.ask("<X0> ISA person.");
        assert_eq!(resp, true);
        resp = kb.ask("<X0> ISA animal.");
        assert_eq!(resp, false);
        resp = kb.ask("<X0> IS animal.");
        assert_eq!(resp, true);
        resp = kb.ask("(say: susan, what: (want: susan, what: <X0>)) ISA fact.");
        assert_eq!(resp, true);
        resp = kb.ask("(say: susan, what: (want: susan, what: (love: <X0>, who: susan))) ISA fact.");
        assert_eq!(resp, true);
        resp = kb.ask("(say: <X1>, what: (want: <X1>, what: (love: <X0>, who: <X1>))) ISA fact.");
        assert_eq!(resp, true);
        resp = kb.ask("(say: <X1>, what: (want: <X1>, what: (love: <X1>, who: <X1>))) ISA fact.");
        assert_eq!(resp, false);
        resp = kb.ask("(say: <X1>, what: (want: <X1>, what: <X1>)) ISA fact.");
        assert_eq!(resp, false);
        resp = kb.ask("(say: john, what: (want: susan, what: <X0>)) ISA fact.");
        assert_eq!(resp, false);
        resp = kb.ask("(want: <X0>, what: (love: <X0>, who: <X1>)) ISA fact.");
        assert_eq!(resp, true);
        resp = kb.ask("(want: <X0>, what: (love: <X0>, who: <X0>)) ISA fact.");
        assert_eq!(resp, false);
        resp = kb.ask("(say: susan, what: <X0>) ISA fact.");
        assert_eq!(resp, true);
        resp = kb.ask("(want: susan, what: <X0>) ISA fact.");
        assert_eq!(resp, false);
        resp = kb.ask("(want: john, what: <X0>) ISA fact.");
        assert_eq!(resp, true);
        resp = kb.ask("(love: john, who: susan) ISA fact.");
        assert_eq!(resp, false);
        resp = kb.ask("(love: susan, who: john) ISA fact.");
        assert_eq!(resp, true);
    }
}
