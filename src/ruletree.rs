use std::clone::Clone;
use std::collections::HashMap;
use std::fmt;
use std::mem;
use std::collections::VecDeque;
use std::cell::RefCell;

use crate::path::SynPath;
use crate::segment::SynSegment;
use crate::matching::SynMatching;
use crate::fact::Fact;


type Response<'a> = Box<Vec<(Vec<RuleRef<'a>>, SynMatching<'a>)>>;

pub fn new_response<'a>() -> Response<'a> {
    Box::new(
        vec![]
    )
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
    path: SynPath<'a>,
    var_child : RefCell<UVarChild<'a>>,
    var_children: RefCell<HashMap<SynPath<'a>, RSNode<'a>>>,
    children: RefCell<HashMap<SynPath<'a>, RSNode<'a>>>,
    rule_refs: RefCell<Vec<RuleRef<'a>>>,
    end_node: RefCell<EndNode>,
}

#[derive(Debug)]
struct UVarChild<'a> {
    node: Option<&'a RSNode<'a>>
}
#[derive(Debug)]
struct EndNode {
    end: bool
}

impl<'a> RSNode<'a> {

    pub fn new(root_path: SynPath<'a>) -> RSNode<'a> {
        RSNode {
            path: root_path,
            var_child: RefCell::new(UVarChild{ node: None }),
            children: RefCell::new(HashMap::new()),
            var_children: RefCell::new(HashMap::new()),
            rule_refs: RefCell::new(vec![]),
            end_node: RefCell::new(EndNode { end: false }),
        }
    }
    pub fn get_children(&'a self) -> &'a HashMap<SynPath<'a>, Self> {
        let children = self.children.borrow();
        unsafe { mem::transmute(&*children) }
    }
    pub fn get_var_children(&'a self) -> &'a HashMap<SynPath<'a>, Self> {
        let vchildren = self.var_children.borrow();
        unsafe { mem::transmute(&*vchildren) }
    }
    pub fn get_child(&'a self, path: &'a SynPath) -> Option<&'a Self> {
        let children = self.children.borrow();
        let child = children.get(path);
        if child.is_none() {
            None
        } else {
            let child_ref = unsafe { mem::transmute(child.unwrap()) };
            Some(child_ref)
        }
    }
    pub fn get_vchild(&'a self, path: &'a SynPath) -> Option<&'a Self> {
        let vchildren = self.var_children.borrow();
        let child = vchildren.get(path);
        if child.is_none() {
            None
        } else {
            let child_ref = unsafe { mem::transmute(child.unwrap()) };
            Some(child_ref)
        }
    }
    pub fn get_var_child(&'a self) -> Option<&'a Self> {
        match self.var_child.borrow().node {
            None => None,
            Some(child) => {
                let child_ref =  unsafe { mem::transmute(child) };
                Some(child_ref)
            }
        }
    }
    pub fn set_var_child(&'a self, node: RSNode<'a>) -> &'a Self {
        let mut var_child = self.var_child.borrow_mut();
        let node_ref = Box::leak(Box::new(node));
        var_child.node = Some(node_ref);
        unsafe { mem::transmute(var_child.node.unwrap()) }
    }
    pub fn intern_child(&'a self, path: &'a SynPath, node: Self) -> &'a Self {
        let mut children = self.children.borrow_mut();
        children.insert(path.clone(), node);
        let child = children.get(path).unwrap();
        unsafe { mem::transmute(child) }
    }
    pub fn intern_var_child(&'a self, path: &'a SynPath, node: Self) -> &'a Self {
        let mut var_children = self.var_children.borrow_mut();
        var_children.insert(path.clone(), node);
        let child = var_children.get(path).unwrap();
        unsafe { mem::transmute(child) }
    }
    pub fn follow_and_create_paths(&'a self, paths: &'a [SynPath], opt_rule_ref: RuleRef<'a>) {
        let mut parent: &RSNode = self; 
        let mut child: Option<&RSNode>; 
        let mut visited_vars: Vec<&SynSegment> = vec![];
        for (i, new_path) in paths.iter().enumerate() {
            if new_path.value.is_empty || !new_path.value.is_leaf {
                continue;
            }
            let mut found = false;
            if new_path.value.is_var {
                child = parent.get_vchild(new_path);
                if child.is_some() {
                    found = true;
                } else if parent.var_child.borrow().node.is_some() {
                    let var_child = parent.get_var_child().unwrap();
                    if &var_child.path == new_path {
                        visited_vars.push(new_path.value);
                        child = Some(&*var_child);
                        found = true;
                    }
                }
            } else {
                child = parent.get_child(new_path);
                if child.is_some() {
                   found = true;
                }
            }
            if found {
                parent = child.unwrap();
            } else {
                parent = parent.create_paths(&paths[i..], visited_vars);
                break;
            }
        }

        parent.rule_refs.borrow_mut().push(opt_rule_ref);
    }

    fn create_paths(&'a self, paths: &'a [SynPath], mut visited: Vec<&'a SynSegment>) -> &'a Self {
        let mut parent: &RSNode = self; 
        let mut child_ref: &RSNode; 
        for new_path in paths {
            if new_path.value.is_empty || !new_path.value.is_leaf {
                continue;
            }
            let child = RSNode::new(new_path.clone());
            if new_path.value.is_var {
                if visited.contains(&new_path.value) {
                    child_ref = parent.intern_var_child(new_path, child);
                } else {
                    visited.push(new_path.value);
                    child_ref = parent.set_var_child(child);
                }
            } else {
                child_ref = parent.intern_child(new_path, child);
            }
            parent = child_ref;
        }
        parent.end_node.borrow_mut().end = true;
        parent
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
            let var_children = parent.get_var_children();
            for (vpath, varchild) in var_children.iter() {
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
            if parent.var_child.borrow().node.is_some() {
                let var_child = parent.get_var_child().unwrap();
                let (new_path_slice, new_value) = path.sub_slice(var_child.path.len());
                let new_paths = SynPath::paths_after_slice(new_path_slice, rest_paths, false);
                let mut new_matched = matched.clone();
                new_matched.insert(var_child.path.value, new_value);
                let (new_response, new_matched) = var_child.climb(new_paths, response, new_matched);
                response = new_response;
                matched = new_matched;
            }
        }
        if parent.end_node.borrow().end {
            // println!("Found rules: {}", parent_rule_refs.len());
            response.push(( parent.rule_refs.borrow().to_vec(), matched.clone() ));
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