use std::clone::Clone;
use std::collections::HashMap;
use std::fmt;
use std::mem;
//use std::collections::VecDeque;
use std::cell::{ RefCell, Cell };

use typed_arena::Arena;

use crate::path::SynPath;
use crate::segment::SynSegment;
use crate::matching::SynMatching;
use crate::fact::Fact;


type Response<'a> = Vec<(Vec<&'a RuleRef<'a>>, SynMatching<'a>)>;

pub fn new_response<'a>() -> Response<'a> {
    vec![]
}

#[derive(Debug, Clone)]
pub struct Rule<'a> {
    pub antecedents: Vec<&'a Fact<'a>>,
    pub more_antecedents: Vec<Vec<&'a Fact<'a>>>,
    pub consequents: Vec<&'a Fact<'a>>,
}

impl<'a> fmt::Display for Rule<'a> {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let more = &self.more_antecedents.iter()
                                         .map(|ants| ants.iter()
                                                         .map(|a| format!("{}", a))
                                                         .collect::<Vec<String>>()
                                                         .join("; "))
                                         .collect::<Vec<String>>()
                                         .join(" -> ");
        write!(f, "{} -> {} -> {}", &self.antecedents.iter()
                                   .map(|a| format!("{}", a))
                                   .collect::<Vec<String>>()
                                   .join("; "),
                              more,
                              &self.consequents.iter()
                                   .map(|a| format!("{}", a))
                                   .collect::<Vec<String>>()
                                   .join("; "))
    }
}

#[derive(Debug, Clone)]
pub struct RuleRef<'a> {
    pub rule: Rule<'a>,
    pub varmap: SynMatching<'a>,
}


#[derive(Debug)]
pub struct RSNode<'a> {
    path: &'a SynPath<'a>,
    var_child : RefCell<Box<UVarChild<'a>>>,
    var_children: RefCell<HashMap<&'a SynPath<'a>, &'a RSNode<'a>>>,
    children: RefCell<HashMap<&'a SynPath<'a>, &'a RSNode<'a>>>,
    rule_refs: RefCell<Vec<&'a RuleRef<'a>>>,
    end_node: Cell<bool>,
}

#[derive(Debug)]
struct UVarChild<'a> {
    node: Option<&'a RSNode<'a>>
}

pub struct RuleSet<'a> {
    pub root: RSNode<'a>,
    nodes: Arena<RSNode<'a>>,
    paths: Arena<SynPath<'a>>,
    rule_refs: Arena<RuleRef<'a>>,
}

impl<'a> RuleSet<'a> {

    pub fn new(root_path: SynPath<'a>) -> Self {
        let root_path_ref = Box::leak(Box::new(root_path));
        let root = RSNode::new(root_path_ref);
        RuleSet {
            root,
            nodes: Arena::new(),
            paths: Arena::new(),
            rule_refs: Arena::new(),
        }
    }
    pub fn follow_and_create_paths(&'a self, paths: &'a [SynPath], rule_ref: RuleRef<'a>) {
        let mut parent: &RSNode = &self.root; 
        let mut child: Option<&RSNode>; 
        let mut visited_vars: Vec<&SynSegment> = vec![];
        for (i, new_path) in paths.iter().enumerate() {
            if new_path.value.is_empty || !new_path.value.is_leaf {
                continue;
            }
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
                parent = self.create_paths(parent, &paths[i..], visited_vars);
                break;
            }
        }

        let rule_ref_ref = self.rule_refs.alloc(rule_ref);
        parent.rule_refs.borrow_mut().push(rule_ref_ref);
    }

    fn create_paths(&'a self, mut parent: &'a RSNode<'a>, paths: &'a [SynPath], mut visited: Vec<&'a SynSegment>) -> &'a RSNode {
        for new_path in paths {
            if new_path.value.is_empty || !new_path.value.is_leaf {
                continue;
            }
            let new_path_ref = self.paths.alloc(new_path.clone());
            let child_ref = self.nodes.alloc(RSNode::new(new_path_ref));
            if new_path.value.is_var {
                if visited.contains(&new_path.value) {
                    parent.var_children.borrow_mut().insert(new_path_ref, child_ref);
                } else {
                    visited.push(new_path.value);
                    parent.var_child.borrow_mut().node = Some(child_ref);
                }
            } else {
                parent.children.borrow_mut().insert(new_path_ref, child_ref);
            }
            parent = child_ref;
        }
        parent.end_node.set(true);
        parent
    }
    pub fn query_paths(&'a self, paths: &'a [SynPath<'a>]) -> Response {
        let response = new_response();
        let matched: SynMatching = HashMap::new();
        let (response, _) = self.root.climb(paths, response, matched);
        response
    }
}

impl<'a> RSNode<'a> {

    pub fn new(root_path: &'a SynPath<'a>) -> RSNode<'a> {
        RSNode {
            path: root_path,
            var_child: RefCell::new(Box::new(UVarChild { node: None })),
            children: RefCell::new(HashMap::new()),
            var_children: RefCell::new(HashMap::new()),
            rule_refs: RefCell::new(vec![]),
            end_node: Cell::new(false),
        }
    }
    pub fn get_child(&'a self, path: &'a SynPath) -> Option<&'a Self> {
        let children = self.children.borrow();
        match children.get(path) {
            None => None,
            Some(child_ref) => {
                Some(*child_ref)
            }
        }
    }
    pub fn get_vchild(&'a self, path: &'a SynPath) -> Option<&'a Self> {
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
                 mut paths: &'a [SynPath<'a>],
                 mut response: Response<'a>,
                 mut matched: SynMatching<'a>) -> (Response<'a>, SynMatching<'a>) {
        let parent = self;
        let mut finished = false;
        let mut next_path: Option<&SynPath> = None;
        let mut next_paths: Option<&'a [SynPath]> = None;
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
            let child_opt = parent.get_child(path);
            if child_opt.is_some() {
                let child = child_opt.unwrap();
                let (new_response, new_matched) = child.climb(rest_paths, response, matched);
                response = new_response;
                matched = new_matched;
            }
            for (vpath, varchild) in parent.var_children.borrow().iter() {
                let (new_path_slice, new_value) = path.sub_slice(vpath.len());
                let old_value = matched.get(vpath.value);
                if old_value.is_some() {
                    if &new_value == old_value.unwrap() {
                        let new_paths = SynPath::paths_after_slice(new_path_slice, rest_paths, false);
                        let (new_response, new_matched) = varchild.climb(new_paths, response, matched);
                        response = new_response;
                        matched = new_matched;
                        break;
                    }
                }
            }
            let var_child_opt = parent.get_var_child();
            if var_child_opt.is_some() {
                let var_child = var_child_opt.unwrap();
                let (new_path_slice, new_value) = path.sub_slice(var_child.path.len());
                let new_paths = SynPath::paths_after_slice(new_path_slice, rest_paths, false);
                let mut new_matched = matched.clone();
                new_matched.insert(var_child.path.value, new_value);
                let (new_response, _) = var_child.climb(new_paths, response, new_matched);
                response = new_response;
            }
        }
        if parent.end_node.get() {
            // println!("Found rules: {}", parent_rule_refs.len());
            response.push(( parent.rule_refs.borrow().clone(), matched.clone() ));
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
//        fn new (root_path: SynPath<'a>) -> PremSet<'a> {
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