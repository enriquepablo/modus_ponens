use std::clone::Clone;
use std::collections::HashMap;

use crate::path::SynPath;
use crate::segment::SynSegment;
use crate::matching::SynMatching;
use crate::fact::Fact;


#[derive(Debug)]
pub struct Rule {
    premises: Vec<Fact>,
    consecs: Vec<Fact>,
}

#[derive(Debug)]
pub struct RuleRef {
    rule: Rule,
    premise: Fact,
    varmap: SynMatching,
}


#[derive(Debug)]
pub struct RSNode {
    path: SynPath,
    var_child : Option<Box<RSNode>>,
    var_children: HashMap<SynPath, RSNode>,
    children: HashMap<SynPath, RSNode>,
    rule_refs: Vec<RuleRef>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ChildType {
    Absurd,
    Root,
    Uvar,
    Var,
    Value,
}


#[derive(Debug)]
pub struct RSZipper {
    parent: Option<Box<RSZipper>>,
    child_type: ChildType,
    path: SynPath,
    var_child : Option<Box<RSNode>>,
    var_children: HashMap<SynPath, RSNode>,
    children: HashMap<SynPath, RSNode>,
    rule_refs: Vec<RuleRef>,
    rule_ref: Option<RuleRef>,
}

impl<'a> RSZipper {

    pub fn follow_and_create_paths(self, paths: &[&SynPath]) -> Box<RSNode> {
        let mut zipper: RSZipper = self; 
        let mut node: Option<RSNode>; 
        let mut visited_vars: Vec<SynSegment> = vec![];
        for (i, &new_path) in paths.iter().enumerate() {
            let RSZipper {
                parent, child_type,
                path, mut var_child,
                mut var_children,
                mut children,
                rule_refs,
                rule_ref,
            } = zipper;
            let mut new_child_type = ChildType::Absurd;
            let mut found = true;
            if new_path.value.is_var {
                node = var_children.remove(new_path);
                if node.is_some() {
                    new_child_type = ChildType::Var;
                } else if var_child.is_some() {
                    let RSNode {
                        path: vpath, var_child: vvar_child,
                        var_children: vvar_children, children: vchildren,
                        rule_refs: vrule_refs,
                    } = *var_child.unwrap();
                    if &vpath == new_path {
                        visited_vars.push(new_path.value.clone());
                        node = Some(RSNode{
                            path: vpath, var_child: vvar_child,
                            var_children: vvar_children, children: vchildren,
                            rule_refs: vrule_refs,
                        });
                        new_child_type = ChildType::Uvar;
                        var_child = None;
                    } else {
                        found = false;
                        var_child = Some( Box::new( RSNode {
                            path: vpath, var_child: vvar_child,
                            var_children: vvar_children, children: vchildren,
                            rule_refs: vrule_refs,
                        }));
                    }
                } else {
                    found = false;
                }
            } else {
                node = children.remove(new_path);
                new_child_type = ChildType::Value;
                if node.is_none() {
                   found = false;
                }
            }
            if found {
                let parent_zipper = RSZipper {
                    parent, child_type,
                    path, var_child,
                    var_children,
                    children,
                    rule_refs,
                    rule_ref: None,
                };
                let RSNode {
                    path: child_path,
                    var_child: child_var_child,
                    var_children: child_var_children,
                    children: child_children,
                    rule_refs: child_rule_refs,
                } = node.unwrap();
                zipper = RSZipper {
                    parent: Some(Box::new(parent_zipper)),
                    child_type: new_child_type,
                    path: child_path,
                    var_child: child_var_child,
                    var_children: child_var_children,
                    children: child_children,
                    rule_refs: child_rule_refs,
                    rule_ref: rule_ref,
                };
            } else {
                let parent_zipper = RSZipper {
                    parent, child_type,
                    path, var_child,
                    var_children,
                    children,
                    rule_refs,
                    rule_ref,
                };
                zipper = parent_zipper.create_paths(&paths[i..], visited_vars);
                break;
            }
        }
        zipper.finish()
    }

    fn create_paths(self, paths: &[&SynPath], mut visited: Vec<SynSegment>) -> RSZipper {
        let mut zipper: RSZipper = self; 
        for &new_path in paths {
            let RSZipper {
                parent: pre_parent, child_type: pre_child_type,
                path: pre_path, var_child: pre_var_child,
                var_children: pre_var_children, children: pre_children,
                rule_refs: pre_rule_refs,
                rule_ref,
            } = zipper;
            zipper = RSZipper {
                parent: pre_parent, child_type: pre_child_type,
                path: pre_path, var_child: pre_var_child,
                var_children: pre_var_children, children: pre_children,
                rule_refs: pre_rule_refs,
                rule_ref: None,
            };
            let child_type: ChildType;
            if new_path.is_var() {
                if visited.contains(&new_path.value) {
                    child_type = ChildType::Var;
                } else {
                    visited.push(new_path.value.clone());
                    child_type = ChildType::Uvar;
                }
            } else {
                child_type = ChildType::Value;
            }
            let new_zipper = RSZipper {
                parent: Some(Box::new(zipper)),
                child_type,
                path: new_path.clone(),
                var_child: None,
                var_children: HashMap::new(),
                children: HashMap::new(),
                rule_refs: vec![], 
                rule_ref,
            };
            zipper = new_zipper;
        }
        let RSZipper {
            parent, child_type,
            path, var_child,
            var_children, children,
            mut rule_refs,
            rule_ref,
        } = zipper;

        if rule_ref.is_some() {
            rule_refs.push(rule_ref.unwrap());
        }

        let zipper = RSZipper {
            parent, child_type,
            path, var_child,
            var_children, children,
            rule_refs,
            rule_ref: None,
        };
        zipper
    }
    
    fn get_parent(self) -> Option<RSZipper> {
        // Destructure this NodeZipper
        let RSZipper {
            parent, child_type,
            path, var_child,
            var_children,
            children,
            rule_refs,
            rule_ref,
        } = self;

        // Insert the node of this NodeZipper back in its parent.
        if child_type == ChildType::Root {
            None
        } else {

            // Destructure the parent NodeZipper
            let RSZipper {
                parent: parent_parent,
                child_type: parent_child_type,
                path: parent_path,
                var_child: mut parent_var_child,
                var_children: mut parent_var_children,
                children: mut parent_children,
                rule_refs: parent_rule_refs,
                rule_ref: parent_rule_ref,
            } = *parent.unwrap();
            let ppc = path.clone();    
            let node = RSNode {
                path,
                var_child,
                var_children,
                children,
                rule_refs,
            };
            if child_type == ChildType::Value {
                parent_children.insert(ppc, node);
            } else if child_type == ChildType::Var {
                parent_var_children.insert(ppc, node);
            } else if child_type == ChildType::Uvar {
                parent_var_child = Some(Box::new(node));
            }
            // Return a new NodeZipper focused on the parent.
            Some(RSZipper {
                parent: parent_parent,
                path: parent_path,
                child_type: parent_child_type,
                var_child: parent_var_child,
                var_children: parent_var_children,
                children: parent_children,
                rule_refs: parent_rule_refs,
                rule_ref: parent_rule_ref,
            })
        }
    }

    pub fn finish(mut self) -> Box<RSNode> {
        while let Some(_) = self.parent {
            self = self.get_parent().expect("parent node");
        }
        let RSZipper {
            path, var_child,
            var_children, children,
            rule_refs, ..
        } = self;

        Box::new(RSNode {
            path, var_child,
            var_children,
            children,
            rule_refs,
        })
    }
}




impl<'a> RSNode {
    pub fn zipper(self, rule_ref: Option<RuleRef>) -> RSZipper {
        let RSNode {
            path, var_child,
            var_children, children,
            rule_refs,
        } = self;
          
        RSZipper {
            parent: None,
            child_type: ChildType::Root,
            path, var_child,
            var_children, children,
            rule_refs,
            rule_ref: rule_ref,
        }
    }
    pub fn new() -> RSNode {
        RSNode {
            path: SynPath::empty_root(),
            var_child: None,
            children: HashMap::new(),
            var_children: HashMap::new(),
            rule_refs: vec![],
        }
    }
}


#[derive(Debug)]
pub struct IRSZipper<'a> {
    path: &'a SynPath,
    var_child: Option<&'a Box<RSNode>>,
    var_children: &'a HashMap<SynPath, RSNode>,
    children: &'a HashMap<SynPath, RSNode>,
    matched: SynMatching,
    response: Box<Vec<SynMatching>>,
}

impl<'a> IRSZipper<'a> {

    pub fn climb(self, paths: &'a [&SynPath]) -> IRSZipper<'a> {
        let IRSZipper {
            path: parent_path,
            var_child: mut parent_var_child,
            matched: mut parent_matched,
            children: parent_children,
            var_children: parent_var_children,
            mut response,
        } = self;
        let split_paths = paths.split_first();
        if split_paths.is_some() {
            let (&path, rest_paths) = split_paths.unwrap();
            let childo = parent_children.get(path);
            if childo.is_some() {
                let child = childo.unwrap();
                let vchild = match &child.var_child {
                    None => None,
                    Some(node) => Some(node),
                };
                let mut zipper = IRSZipper {
                    path: &child.path,
                    matched: parent_matched,
                    var_child: vchild,
                    children: &child.children,
                    var_children: &child.var_children,
                    response,
                };
                zipper = zipper.climb(rest_paths);
                let IRSZipper {
                    matched: old_parent_matched,
                    response: new_response, ..
                } = zipper;
                response = new_response;
                parent_matched = old_parent_matched;
            }
            for (vpath, varchild) in parent_var_children.iter() {
                let (new_path, _) = path.sub_path(vpath.len());
                let old_value = parent_matched.get(&vpath.value);
                if old_value.is_some() {
                    if &new_path.value == old_value.unwrap() {
                        let new_paths = new_path.paths_after(rest_paths, false);
                        let vchild = match &varchild.var_child {
                            None => None,
                            Some(node) => Some(node),
                        };
                        let mut zipper = IRSZipper {
                            path: &varchild.path,
                            matched: parent_matched,
                            var_child: vchild,
                            children: &varchild.children,
                            var_children: &varchild.var_children,
                            response,
                        };
                        zipper = zipper.climb(new_paths);
                        let IRSZipper {
                            matched: old_parent_matched,
                            response: new_response, ..
                        } = zipper;
                        response = new_response;
                        parent_matched = old_parent_matched;
                        break;
                    }
                }
            }
            if parent_var_child.is_some() {
                let var_child = parent_var_child.unwrap();
                let (new_path, val) = path.sub_path(var_child.path.len());
                let new_paths = new_path.paths_after(rest_paths, false);
                let mut new_matched = parent_matched.clone();
                new_matched.insert(var_child.path.value.clone(), val.clone());
                let vchild = match &var_child.var_child {
                    None => None,
                    Some(node) => Some(node),
                };
                let mut zipper = IRSZipper {
                    path: &var_child.path,
                    matched: new_matched,
                    var_child: vchild,
                    children: &var_child.children,
                    var_children: &var_child.var_children,
                    response,
                };
                zipper = zipper.climb(new_paths);
                let IRSZipper {
                    matched: _,
                    response: new_response, ..
                } = zipper;
                parent_var_child = Some(var_child);
                response = new_response;
            }
        } else {
            // ENDNODE
            response.push(parent_matched.clone());
        }
        IRSZipper {
            path: parent_path,
            var_child: parent_var_child,
            matched: parent_matched,
            children: parent_children,
            var_children: parent_var_children,
            response,
        }
    }

    pub fn finish(self) -> Box<Vec<SynMatching>> {
        
        let IRSZipper {
            response, ..
        } = self;
        response
    }
}


impl<'a> RSNode {
    pub fn izipper(&'a self, response: Box<Vec<SynMatching>>) -> IRSZipper<'a> {
        
        let matching: SynMatching = HashMap::new();
        let vchild = match &self.var_child {
            None => None,
            Some(node) => Some(node),
        };

        IRSZipper {
            path: &self.path,
            var_child: vchild,
            var_children: &self.var_children,
            children: &self.children,
            matched: matching,
            response
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact::Fact;
    use crate::parser::parse_text;


    pub struct PremSet {
        pub root: Box<RSNode>,
    }


    impl<'a> PremSet {
        fn new () -> PremSet {
            PremSet { root: Box::new(RSNode::new()) }
        }
        fn add_fact (self, fact: Fact) -> PremSet {
            let PremSet { mut root } = self;
            let zipper = root.zipper(None);
            let paths = fact.get_leaf_paths();
            root = zipper.follow_and_create_paths(&paths);
            PremSet { root }
        }
        fn ask_fact (&'a self, fact: &'a Fact) -> usize {
            let mut response: Box<Vec<SynMatching>> = Box::new(vec![]);
            let mut qzipper = self.root.izipper(response);
            let paths = fact.get_leaf_paths();
            qzipper = qzipper.climb(&paths);
            response = qzipper.finish();
            response.len()
        }
    }


    pub struct Fakeledge {
        pub factset: PremSet,
    }


    impl<'a> Fakeledge {
        pub fn new () -> Fakeledge {
            Fakeledge { factset: PremSet::new() }
        }
        fn tell(self, k: &str) -> Fakeledge {
            let Fakeledge {
                mut factset
            } = self;
            let parsed = parse_text(k);
            let facts = parsed.ok().unwrap().facts;
            for fact in facts {
                factset = factset.add_fact(fact);
            }
            Fakeledge {
                factset
            }
        }
        fn ask(&'a self, q: &str) -> usize {
            let parsed = parse_text(q);
            let mut facts = parsed.ok().unwrap().facts;
            let fact = facts.pop().unwrap();
            self.factset.ask_fact(&fact)
        }
    }
    
    #[test]
    fn test_1() {
        let mut kb = Fakeledge::new();
        kb = kb.tell("susan ISA person. john ISA person.");
        let resp1 = kb.ask("susan ISA person.");
        assert_eq!(resp1, 1);
        let resp2 = kb.ask("pepe ISA person.");
        assert_eq!(resp2, 0);
        let resp3 = kb.ask("john ISA person.");
        assert_eq!(resp3, 1);
    }
    #[test]
    fn test_2() {
        let mut kb = Fakeledge::new();
        kb = kb.tell("<X0> ISA person. john ISA <X0>.");
        let resp1 = kb.ask("susan ISA person.");
        assert_eq!(resp1, 1);
        let resp3 = kb.ask("john ISA person.");
        assert_eq!(resp3, 2);
        let resp3 = kb.ask("john ISA animal.");
        assert_eq!(resp3, 1);
        let resp1 = kb.ask("susan ISA animal.");
        assert_eq!(resp1, 0);
    }
    #[test]
    fn test_3() {
        let mut kb = Fakeledge::new();
        kb = kb.tell("\
            susan ISA person.\
            john ISA person.\
            <X0> IS animal.\
            (say: <X0>, what: (<X1>: <X0>, what: (love: <X2>, who: <X0>))) ISA fact.\
            (<X0>: <X1>, what: (love: <X1>, who: <X2>)) ISA fact.\
            (say: <X0>, what: <X1>) ISA fact.");
        let mut resp = kb.ask("susan ISA person.");
        assert_eq!(resp, 1);
        resp = kb.ask("pepe ISA person.");
        assert_eq!(resp, 0);
        resp = kb.ask("(say: susan, what: (want: susan, what: (love: john, who: susan))) ISA fact.");
        assert_eq!(resp, 1);  // XXX should be 2
        resp = kb.ask("(say: susan, what: (want: susan, what: (love: john, who: pepe))) ISA fact.");
        assert_eq!(resp, 1);
        resp = kb.ask("(want: john, what: (love: john, who: susan)) ISA fact.");
        assert_eq!(resp, 1);
        resp = kb.ask("(want: pepe, what: (love: john, who: susan)) ISA fact.");
        assert_eq!(resp, 0);
        resp = kb.ask("(say: susan, what: (love: susan)) ISA fact.");
        assert_eq!(resp, 1);
        resp = kb.ask("(say: susan, whit: (love: susan)) ISA fact.");
        assert_eq!(resp, 0);
    }
    #[test]
    fn test_4() {
        let mut kb = Fakeledge::new();
        kb = kb.tell("(say: <X0>, what: (<X1>: <X0>, what: (love: <X2>, who: <X0>))) ISA fact.");
        let resp = kb.ask("(say: susan, what: (want: susan, what: (love: john, who: susan))) ISA fact.");
        assert_eq!(resp, 1);
    }
    #[test]
    fn test_5() {
        let mut kb = Fakeledge::new();
        kb = kb.tell("(say: <X0>, what: <X1>) ISA fact.");
        let resp = kb.ask("(say: susan, what: (want: susan, what: (love: john, who: susan))) ISA fact.");
        assert_eq!(resp, 1);
    }
}