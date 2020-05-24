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

use std::clone::Clone;
use std::collections::{ HashMap, VecDeque };
use std::fmt;
//use std::collections::VecDeque;
use std::cell::{ RefCell, Cell };

use bumpalo::{Bump};
//use log::debug;

use crate::constants;
use crate::path::MPPath;
use crate::segment::MPSegment;
use crate::matching::MPMatching;
use crate::fact::Fact;


pub type Response<'a> = Vec<(&'a RefCell<Vec<RuleRef<'a>>>, MPMatching<'a>)>;

pub fn new_response<'a>() -> Response<'a> {
    vec![]
}

#[derive(Debug, Clone)]
pub struct Antecedents<'a> {
    pub facts: Vec<&'a Fact<'a>>,
    pub transforms: &'a str,
    pub conditions: &'a str,
}

#[derive(Debug, Clone)]
pub struct MPRule<'a> {
    pub antecedents: Antecedents<'a>,
    pub more_antecedents: VecDeque<Antecedents<'a>>,
    pub consequents: Vec<&'a Fact<'a>>,
    pub matched: MPMatching<'a>,
    pub output: Option<&'a Fact<'a>>,
}

impl<'a> fmt::Display for MPRule<'a> {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string = String::from("rule\n\n");
        for fact in self.antecedents.facts.iter() {
            string.push_str(&fact.text);
            string.push_str(" ; ");
        }
        string.push_str("{={ ");
        string.push_str(self.antecedents.transforms);
        string.push_str("}=} ; {?{ ");
        string.push_str(self.antecedents.conditions);
        string.push_str("}?} ->\n");
        for more_ants in &self.more_antecedents {
            for fact in more_ants.facts.iter() {
                string.push_str(&fact.text);
                string.push_str(" ; ");
            }
            string.push_str("{={ ");
            string.push_str(more_ants.transforms);
            string.push_str("}=} ; {?{ ");
            string.push_str(more_ants.conditions);
            string.push_str("}?} ->\n");
        }

        for consequent in &self.consequents {
            string.push_str(&consequent.text);
            string.push_str(" ; ");
        }

        if self.output.is_some() {
            string.push_str(" {!{ ");
            string.push_str(self.output.unwrap().text);
            string.push_str(" }!} <>");
        }

        string.push_str(&format!("\n\nmatching: {:?}", &self.matched));

        write!(f, "{}", string)
    }
}

#[derive(Debug, Clone)]
pub struct RuleRef<'a> {
    pub rule: MPRule<'a>,
    pub varmap: MPMatching<'a>,
}


#[derive(Debug)]
pub struct RSNode<'a> {
    path: &'a MPPath<'a>,
    var_child : RefCell<Box<UVarChild<'a>>>,
    var_children: RefCell<HashMap<&'a MPPath<'a>, &'a RSNode<'a>>>,
    children: RefCell<HashMap<&'a MPPath<'a>, &'a RSNode<'a>>>,
    rule_refs: RefCell<Vec<RuleRef<'a>>>,
    end_node: Cell<bool>,
}

#[derive(Debug)]
struct UVarChild<'a> {
    node: Option<&'a RSNode<'a>>
}

pub struct RuleSet<'a> {
    pub root: RSNode<'a>,
    nodes: Bump,
}

impl<'a> RuleSet<'a> {

    pub fn new(root_path: MPPath<'a>) -> Self {
        let root_path_ref = Box::leak(Box::new(root_path));
        let root = RSNode::new(root_path_ref, 1);
        RuleSet {
            root,
            nodes: Bump::new(),
        }
    }
    pub fn follow_and_create_paths(&'a self, paths: &'a [MPPath], rule_ref: RuleRef<'a>, mut depth: usize) {
        let mut parent: &RSNode = &self.root; 
        let mut child: Option<&RSNode>; 
        let mut visited_vars: Vec<&MPSegment> = vec![];
        for (i, new_path) in paths.iter().enumerate() {
            if new_path.value.is_empty || !new_path.value.is_leaf {
                continue;
            }
            depth += 1;
            if new_path.value.is_var {
                child = parent.get_vchild(new_path);
                if child.is_none() {
                    let var_child_opt = parent.get_var_child();
                    if var_child_opt.is_some() {
                        let var_child =  var_child_opt.unwrap();
                        if var_child.path == new_path {
                            visited_vars.push(new_path.value);
                            child = Some(var_child);
                        }
                    }
                }
            } else {
                child = parent.get_child(new_path);
            }
            if child.is_some() {
                parent = child.unwrap();
            } else {
                parent = self.create_paths(parent, &paths[i..], visited_vars, depth);
                break;
            }
        }

        parent.end_node.set(true);
        parent.rule_refs.borrow_mut().push(rule_ref);
    }

    fn create_paths(&'a self, mut parent: &'a RSNode<'a>, paths: &'a [MPPath], mut visited: Vec<&'a MPSegment>, mut depth: usize) -> &'a RSNode {
        for new_path in paths {
            if new_path.value.is_empty || !new_path.value.is_leaf {
                continue;
            }
            depth += 1;
            let child_ref = self.nodes.alloc(RSNode::new(new_path, depth));
            if new_path.value.is_var {
                if visited.contains(&new_path.value) {
                    parent.var_children.borrow_mut().insert(new_path, child_ref);
                } else {
                    visited.push(new_path.value);
                    parent.var_child.borrow_mut().node = Some(child_ref);
                }
            } else {
                parent.children.borrow_mut().insert(new_path, child_ref);
            }
            parent = child_ref;
        }
        parent.end_node.set(true);
        parent
    }
    pub fn query_paths(&'a self, paths: &'a [MPPath<'a>]) -> Response {
        let response = new_response();
        let matched: MPMatching = HashMap::new();
        let (response, _) = self.root.climb(paths, response, matched);
        response
    }
}

impl<'a> RSNode<'a> {

    pub fn new(root_path: &'a MPPath<'a>, depth: usize) -> RSNode<'a> {
        let capacity = constants::NODE_MAP_CAPACITY / depth;
        RSNode {
            path: root_path,
            var_child: RefCell::new(Box::new(UVarChild { node: None })),
            children: RefCell::new(HashMap::with_capacity(capacity)),
            var_children: RefCell::new(HashMap::with_capacity(capacity)),
            rule_refs: RefCell::new(vec![]),
            end_node: Cell::new(false),
        }
    }
    pub fn get_child(&'a self, path: &'a MPPath) -> Option<&'a Self> {
        let children = self.children.borrow();
        match children.get(path) {
            None => None,
            Some(child_ref) => {
                Some(*child_ref)
            }
        }
    }
    pub fn get_vchild(&'a self, path: &'a MPPath) -> Option<&'a Self> {
        let vchildren = self.var_children.borrow();
        match vchildren.get(path) {
            None => None,
            Some(child_ref) => {
                Some(*child_ref)
            }
        }
    }

    pub fn get_var_child(&'a self) -> Option<&'a Self> {
        let var_child_opt = self.var_child.borrow().node;
        match var_child_opt {
            None => None,
            Some(var_child) => {
                Some(var_child)
            }
        }
    }

    pub fn climb(&'a self,
                 mut paths: &'a [MPPath<'a>],
                 mut response: Response<'a>,
                 mut matched: MPMatching<'a>) -> (Response<'a>, MPMatching<'a>) {
        let mut finished = false;
        let mut next_path: Option<&MPPath> = None;
        let mut next_paths: Option<&'a [MPPath]> = None;
        while !finished {
            let split_paths = paths.split_first();
            if split_paths.is_some() {
                let (path, less_paths) = split_paths.unwrap();
                if !path.value.is_empty && path.value.is_leaf {
                    finished = true;
                    next_path = Some(path);
                    next_paths = Some(less_paths);
                } else {
                    paths = less_paths;
                }
            } else {
                finished = true;
            }
            
        }
        if next_path.is_some(){
            let path = next_path.unwrap();
            let rest_paths = next_paths.unwrap();
            let child_opt = self.get_child(path);
            if child_opt.is_some() {
                let child = child_opt.unwrap();
                let (new_response, old_matched) = child.climb(rest_paths, response, matched);
                response = new_response;
                matched = old_matched;
            }
            for (vpath, varchild) in self.var_children.borrow().iter() {
                let (new_path_slice, new_value) = path.sub_slice(vpath.len());
                let old_value = matched.get(vpath.value);
                if old_value.is_some() {
                    if &new_value == old_value.unwrap() {
                        let new_paths = MPPath::paths_after_slice(new_path_slice, rest_paths);
                        let (new_response, old_matched) = varchild.climb(new_paths, response, matched);
                        response = new_response;
                        matched = old_matched;
                        break;
                    }
                }
            }
            let var_child_opt = self.get_var_child();
            if var_child_opt.is_some() {
                let var_child = var_child_opt.unwrap();
                let (new_path_slice, new_value) = path.sub_slice(var_child.path.len());
                let new_paths = MPPath::paths_after_slice(new_path_slice, rest_paths);
                let mut new_matched = matched.clone();
                new_matched.insert(var_child.path.value, new_value);
                let (new_response, _) = var_child.climb(new_paths, response, new_matched);
                response = new_response;
            }
        }
        if self.end_node.get() {
            //debug!("Pushing to response: {}\n\n{:?} \n\n{:?}", &parent.rule_refs.borrow().len(), &parent.rule_refs.borrow().first().unwrap().varmap, &matched);
            // println!("Found rules: {}", parent_rule_refs.len());
            response.push(( &self.rule_refs, matched.clone() ));
        }
        (response, matched)
    }
}

//
//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::fact::Fact;
//    use crate::parser::Grammar;
//
//
//    pub struct PremSet<'a> {
//        pub root: Box<RSNode<'a>>,
//    }
//
//
//    impl<'a> PremSet<'a> {
//        fn new (root_path: MPPath<'a>) -> PremSet<'a> {
//            PremSet { root: Box::new(RSNode::new(root_path)) }
//        }
//        fn add_fact (self, fact: &'a Fact<'a>) -> PremSet {
//            let PremSet { mut root } = self;
//            let zipper = root.zipper(None);
//            let paths = fact.get_leaf_paths();
//            root = zipper.follow_and_create_paths(paths);
//            PremSet { root }
//        }
//        fn ask_fact (&'a self, fact: &'a Fact) -> usize {
//            let mut qzipper = self.root.izipper();
//            let paths = fact.get_leaf_paths();
//            qzipper = qzipper.climb(&paths);
//            let response = qzipper.finish();
//            response.len()
//        }
//    }
//
//
//    pub struct Fakeledge<'a> {
//        pub factset: PremSet<'a>,
//        grammar: Grammar<'a>,
//    }
//
//
//    impl<'a> Fakeledge<'a> {
//        pub fn new () -> Fakeledge<'a> {
//            let grammar = Grammar::new();
//            let root_path = grammar.lexicon.empty_path();
//            Fakeledge {
//                grammar: grammar,
//                factset: PremSet::new(root_path),
//            }
//        }
//        fn tell(self, k: &'a str) -> Fakeledge<'a> {
//            let Fakeledge {
//                mut factset,
//                grammar,
//            } = self;
//            let parsed = grammar.parse_text(k);
//            let facts = parsed.ok().unwrap().facts;
//            for fact in facts {
//                factset = factset.add_fact(fact);
//            }
//            Fakeledge {
//                grammar,
//                factset,
//            }
//        }
//        fn ask(&'a self, q: &'a str) -> usize {
//            let parsed = self.grammar.parse_text(q);
//            let mut facts = parsed.ok().unwrap().facts;
//            let fact = facts.pop().unwrap();
//            self.factset.ask_fact(&fact)
//        }
//    }
//    
//    #[test]
//    fn test_1() {
//        let mut kb = Fakeledge::new();
//        kb = kb.tell("susan ISA person. john ISA person.");
//        let resp1 = kb.ask("susan ISA person.");
//        assert_eq!(resp1, 1);
//        let resp2 = kb.ask("pepe ISA person.");
//        assert_eq!(resp2, 0);
//        let resp3 = kb.ask("john ISA person.");
//        assert_eq!(resp3, 1);
//    }
//    #[test]
//    fn test_2() {
//        let mut kb = Fakeledge::new();
//        kb = kb.tell("<X0> ISA person. john ISA <X0>.");
//        let resp1 = kb.ask("susan ISA person.");
//        assert_eq!(resp1, 1);
//        let resp3 = kb.ask("john ISA person.");
//        assert_eq!(resp3, 2);
//        let resp3 = kb.ask("john ISA animal.");
//        assert_eq!(resp3, 1);
//        let resp1 = kb.ask("susan ISA animal.");
//        assert_eq!(resp1, 0);
//    }
//    #[test]
//    fn test_3() {
//        let mut kb = Fakeledge::new();
//        kb = kb.tell("\
//            susan ISA person.\
//            john ISA person.\
//            <X0> IS animal.\
//            (say: <X0>, what: (<X1>: <X0>, what: (love: <X2>, who: <X0>))) ISA fact.\
//            (<X0>: <X1>, what: (love: <X1>, who: <X2>)) ISA fact.\
//            (say: <X0>, what: <X1>) ISA fact.");
//        let mut resp = kb.ask("susan ISA person.");
//        assert_eq!(resp, 1);
//        resp = kb.ask("pepe ISA person.");
//        assert_eq!(resp, 0);
//        resp = kb.ask("(say: susan, what: (want: susan, what: (love: john, who: susan))) ISA fact.");
//        assert_eq!(resp, 2);  // XXX should be 2
//        resp = kb.ask("(say: susan, what: (want: susan, what: (love: john, who: pepe))) ISA fact.");
//        assert_eq!(resp, 1);
//        resp = kb.ask("(want: john, what: (love: john, who: susan)) ISA fact.");
//        assert_eq!(resp, 1);
//        resp = kb.ask("(want: pepe, what: (love: john, who: susan)) ISA fact.");
//        assert_eq!(resp, 0);
//        resp = kb.ask("(say: susan, what: (love: susan)) ISA fact.");
//        assert_eq!(resp, 1);
//        resp = kb.ask("(say: susan, whit: (love: susan)) ISA fact.");
//        assert_eq!(resp, 0);
//    }
//    #[test]
//    fn test_4() {
//        let mut kb = Fakeledge::new();
//        kb = kb.tell("(say: <X0>, what: (<X1>: <X0>, what: (love: <X2>, who: <X0>))) ISA fact.");
//        let resp = kb.ask("(say: susan, what: (want: susan, what: (love: john, who: susan))) ISA fact.");
//        assert_eq!(resp, 1);
//    }
//    #[test]
//    fn test_5() {
//        let mut kb = Fakeledge::new();
//        kb = kb.tell("(say: <X0>, what: <X1>) ISA fact.");
//        let resp = kb.ask("(say: susan, what: (want: susan, what: (love: john, who: susan))) ISA fact.");
//        assert_eq!(resp, 1);
//    }
//    #[test]
//    fn test_6() {
//        let mut kb = Fakeledge::new();
//        kb = kb.tell("(fn: (fn: <X1>, on: <X4>), on: <X6>) EQ <X7>.");
//        let resp = kb.ask("(fn: (fn: pr, on: s1), on: (s: 0)) EQ (s: (s: 0)).");
//        assert_eq!(resp, 1);
//    }
//}
