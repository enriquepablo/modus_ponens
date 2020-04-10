use std::clone::Clone;
use std::collections::HashMap;
use std::fmt;
use std::collections::VecDeque;

use crate::path::SynPath;
use crate::segment::SynSegment;
use crate::matching::SynMatching;
use crate::fact::Fact;


#[derive(Debug, Clone)]
pub struct Rule<'a> {
    pub antecedents: Vec<&'a Fact<'a>>,
    pub more_antecedents: VecDeque<Vec<&'a Fact<'a>>>,
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
    var_child : Option<Box<RSNode<'a>>>,
    var_children: HashMap<SynPath<'a>, RSNode<'a>>,
    children: HashMap<SynPath<'a>, RSNode<'a>>,
    rule_refs: Vec<RuleRef<'a>>,
    end_node: bool,
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
pub struct RSZipper<'a> {
    parent: Option<Box<RSZipper<'a>>>,
    child_type: ChildType,
    path: SynPath<'a>,
    var_child : Option<Box<RSNode<'a>>>,
    var_children: HashMap<SynPath<'a>, RSNode<'a>>,
    children: HashMap<SynPath<'a>, RSNode<'a>>,
    rule_refs: Vec<RuleRef<'a>>,
    rule_ref: Option<RuleRef<'a>>,
    end_node: bool,
}

impl<'a> RSZipper<'a> {

    pub fn follow_and_create_paths(self, paths: &'a [&SynPath]) -> Box<RSNode<'a>> {
        let mut zipper: RSZipper = self; 
        let mut node: Option<RSNode>; 
        let mut visited_vars: Vec<&SynSegment> = vec![];
        for (i, &new_path) in paths.iter().enumerate() {
            let RSZipper {
                parent, child_type,
                path, mut var_child,
                mut var_children,
                mut children,
                rule_refs,
                rule_ref,
                end_node,
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
                        rule_refs: vrule_refs, end_node: vend_node,
                    } = *var_child.unwrap();
                    if &vpath == new_path {
                        visited_vars.push(new_path.value);
                        node = Some(RSNode{
                            path: vpath, var_child: vvar_child,
                            var_children: vvar_children, children: vchildren,
                            rule_refs: vrule_refs, end_node: vend_node,
                        });
                        new_child_type = ChildType::Uvar;
                        var_child = None;
                    } else {
                        found = false;
                        var_child = Some( Box::new( RSNode {
                            path: vpath, var_child: vvar_child,
                            var_children: vvar_children, children: vchildren,
                            rule_refs: vrule_refs, end_node: vend_node,
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
                    end_node,
                };
                let RSNode {
                    path: child_path,
                    var_child: child_var_child,
                    var_children: child_var_children,
                    children: child_children,
                    rule_refs: child_rule_refs,
                    end_node: child_end_node,
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
                    end_node: child_end_node,
                };
            } else {
                let parent_zipper = RSZipper {
                    parent, child_type,
                    path, var_child,
                    var_children,
                    children,
                    rule_refs,
                    rule_ref,
                    end_node,
                };
                zipper = parent_zipper.create_paths(&paths[i..], visited_vars);
                break;
            }
        }
        let RSZipper {
            parent, child_type,
            path, var_child,
            var_children, children,
            mut rule_refs,
            rule_ref,
            end_node,
        } = zipper;

        if rule_ref.is_some() {
            rule_refs.push(rule_ref.unwrap());
        }

        zipper = RSZipper {
            parent, child_type,
            path, var_child,
            var_children, children,
            rule_refs,
            rule_ref: None,
            end_node,
        };
        zipper.finish()
    }

    fn create_paths(self, paths: &'a [&SynPath], mut visited: Vec<&'a SynSegment>) -> RSZipper<'a> {
        let mut zipper: RSZipper = self; 
        for &new_path in paths {
            let RSZipper {
                parent: pre_parent, child_type: pre_child_type,
                path: pre_path, var_child: pre_var_child,
                var_children: pre_var_children, children: pre_children,
                rule_refs: pre_rule_refs,
                rule_ref,
                end_node: pre_end_node,
            } = zipper;
            zipper = RSZipper {
                parent: pre_parent, child_type: pre_child_type,
                path: pre_path, var_child: pre_var_child,
                var_children: pre_var_children, children: pre_children,
                rule_refs: pre_rule_refs,
                rule_ref: None,
                end_node: pre_end_node,
            };
            let child_type: ChildType;
            if new_path.is_var() {
                if visited.contains(&new_path.value) {
                    child_type = ChildType::Var;
                } else {
                    visited.push(new_path.value);
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
                end_node: false,
            };
            zipper = new_zipper;
        }
        zipper.end_node = true;
        zipper
    }
    
    fn get_parent(self) -> Option<RSZipper<'a>> {
        // Destructure this NodeZipper
        let RSZipper {
            parent, child_type,
            path, var_child,
            var_children,
            children,
            rule_refs,
            end_node, ..
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
                end_node: parent_end_node,
            } = *parent.unwrap();
            let ppc = path.clone();    
            let node = RSNode {
                path,
                var_child,
                var_children,
                children,
                rule_refs,
                end_node,
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
                end_node: parent_end_node,
            })
        }
    }

    pub fn finish(mut self) -> Box<RSNode<'a>> {
        while let Some(_) = self.parent {
            self = self.get_parent().expect("parent node");
        }
        let RSZipper {
            path, var_child,
            var_children, children,
            rule_refs, end_node, ..
        } = self;

        Box::new(RSNode {
            path, var_child,
            var_children,
            children,
            rule_refs,
            end_node,
        })
    }
}




impl<'a> RSNode<'a> {
    pub fn zipper(self, rule_ref: Option<RuleRef<'a>>) -> RSZipper<'a> {
        let RSNode {
            path, var_child,
            var_children, children,
            rule_refs,
            end_node,
        } = self;
          
        RSZipper {
            parent: None,
            child_type: ChildType::Root,
            path, var_child,
            var_children, children,
            rule_refs,
            rule_ref,
            end_node,
        }
    }
    pub fn new(root_path: SynPath<'a>) -> RSNode<'a> {
        RSNode {
            path: root_path,
            var_child: None,
            children: HashMap::new(),
            var_children: HashMap::new(),
            rule_refs: vec![],
            end_node: false,
        }
    }
}


type Response<'a> = Box<Vec<(Vec<RuleRef<'a>>, SynMatching<'a>)>>;

pub fn new_response<'a>() -> Response<'a> {
    Box::new(
        vec![]
    )
}





#[derive(Debug)]
pub struct IRSZipper<'a> {
    path: SynPath<'a>,
    child_type: ChildType,
    var_child: Option<Box<RSNode<'a>>>,
    var_children: HashMap<SynPath<'a>, RSNode<'a>>,
    children: HashMap<SynPath<'a>, RSNode<'a>>,
    rule_refs: Vec<RuleRef<'a>>,
    matched: SynMatching<'a>,
    response: Response<'a>,
    end_node: bool,
}

impl<'a> IRSZipper<'a> {

    pub fn climb(self, paths: &'a [&'a SynPath<'a>]) -> IRSZipper<'a> {
        let IRSZipper {
            path: parent_path,
            child_type: parent_child_type,
            var_child: mut parent_var_child,
            var_children: mut parent_var_children,
            children: mut parent_children,
            rule_refs: parent_rule_refs,
            matched: mut parent_matched,
            mut response,
            end_node: parent_end_node,
        } = self;
        let split_paths = paths.split_first();
        if split_paths.is_some() {
            let (&path, rest_paths) = split_paths.unwrap();
            let pchild = parent_children.remove_entry(path);
            if pchild.is_some() {
                let (chpath, child) = pchild.unwrap();
                let RSNode {
                    path: child_path,
                    var_child: child_var_child,
                    var_children: child_var_children,
                    children: child_children,
                    rule_refs: child_rule_refs,
                    end_node: child_end_node,
                } = child;
                let mut zipper = IRSZipper {
                    path: child_path,
                    matched: parent_matched,
                    child_type: ChildType::Value,
                    var_child: child_var_child,
                    var_children: child_var_children,
                    children: child_children,
                    rule_refs: child_rule_refs,
                    response,
                    end_node: child_end_node,
                };
                zipper = zipper.climb(rest_paths);
                let IRSZipper {
                    path: child_path,
                    var_child: child_var_child,
                    var_children: child_var_children,
                    children: child_children,
                    rule_refs: child_rule_refs,
                    matched: old_parent_matched,
                    end_node: child_end_node,
                    response: new_response, ..
                } = zipper;
                response = new_response;
                parent_matched = old_parent_matched;
                let child = RSNode {
                    path: child_path,
                    var_child: child_var_child,
                    var_children: child_var_children,
                    children: child_children,
                    rule_refs: child_rule_refs,
                    end_node: child_end_node,
                };
                parent_children.insert(chpath, child);
            }
            let rpaths = parent_var_children.keys().cloned().collect::<Vec<SynPath>>();
            for rpath in rpaths {
                let new_path = path.sub_path(rpath.len());
                let old_value = parent_matched.get(rpath.value);
                if old_value.is_some() {
                    if &new_path.value == old_value.unwrap() {
                        let (vpath, varchild) = parent_var_children.remove_entry(&rpath).unwrap();
                        let RSNode {
                            path: varchild_path,
                            var_child: varchild_var_child,
                            var_children: varchild_var_children,
                            children: varchild_children,
                            rule_refs: varchild_rule_refs,
                            end_node: varchild_end_node,
                        } = varchild;
                        let new_paths = new_path.clone().paths_after_owning(rest_paths, false);
                        let mut zipper = IRSZipper {
                            path: varchild_path,
                            matched: parent_matched,
                            child_type: ChildType::Var,
                            var_child: varchild_var_child,
                            var_children: varchild_var_children,
                            children: varchild_children,
                            rule_refs: varchild_rule_refs,
                            response,
                            end_node: varchild_end_node,
                        };
                        zipper = zipper.climb(new_paths);
                        let IRSZipper {
                            path: varchild_path,
                            matched: old_parent_matched,
                            var_child: varchild_var_child,
                            var_children: varchild_var_children,
                            children: varchild_children,
                            rule_refs: varchild_rule_refs,
                            response: new_response,
                            end_node: varchild_end_node, ..
                        } = zipper;
                        let varchild = RSNode {
                            path: varchild_path,
                            var_child: varchild_var_child,
                            var_children: varchild_var_children,
                            children: varchild_children,
                            rule_refs: varchild_rule_refs,
                            end_node: varchild_end_node,
                        };
                        parent_var_children.insert(vpath, varchild);
                        response = new_response;
                        parent_matched = old_parent_matched;
                        break;
                    }
                }
            }
            if parent_var_child.is_some() {
                let var_child = parent_var_child.unwrap();
                let new_path = path.sub_path(var_child.path.len());
                let new_paths = new_path.clone().paths_after_owning(rest_paths, false);
                let mut new_matched = parent_matched.clone();
                new_matched.insert(var_child.path.value, new_path.value);
                let RSNode {
                    path: varchild_path,
                    var_child: varchild_var_child,
                    var_children: varchild_var_children,
                    children: varchild_children,
                    rule_refs: varchild_rule_refs,
                    end_node: varchild_end_node,
                } = *var_child;
                let mut zipper = IRSZipper {
                    path: varchild_path,
                    matched: new_matched,
                    child_type: ChildType::Uvar,
                    var_child: varchild_var_child,
                    var_children: varchild_var_children,
                    children: varchild_children,
                    rule_refs: varchild_rule_refs,
                    response,
                    end_node: varchild_end_node,
                };
                zipper = zipper.climb(new_paths);
                let IRSZipper {
                    path: varchild_path,
                    var_child: varchild_var_child,
                    var_children: varchild_var_children,
                    children: varchild_children,
                    rule_refs: varchild_rule_refs,
                    response: new_response,
                    end_node: varchild_end_node, ..
                } = zipper;
                let var_child = Box::new(RSNode {
                    path: varchild_path,
                    var_child: varchild_var_child,
                    var_children: varchild_var_children,
                    children: varchild_children,
                    rule_refs: varchild_rule_refs,
                    end_node: varchild_end_node,
                });
                parent_var_child = Some(var_child);
                response = new_response;
            }
        }
        if parent_end_node {
            // println!("Found rules: {}", parent_rule_refs.len());
            response.push(( parent_rule_refs.clone(), parent_matched.clone() ));
        }
        IRSZipper {
            path: parent_path,
            var_child: parent_var_child,
            matched: parent_matched,
            child_type: parent_child_type,
            children: parent_children,
            var_children: parent_var_children,
            rule_refs: parent_rule_refs,
            response,
            end_node: parent_end_node,
        }
    }

    pub fn finish(self) -> (Box<RSNode<'a>>, Response<'a>) {
        
        let IRSZipper {
            path, var_child, matched,
            child_type, children, var_children,
            rule_refs,
            response,
            end_node,
        } = self;
        let root = Box::new(RSNode {
            path,
            var_child,
            var_children,
            children,
            rule_refs,
            end_node,
        });
        (root, response)
    }
}

impl<'a> RSNode<'a> {
    pub fn izipper(self) -> IRSZipper<'a> {
        
        let response = new_response();
        let matching: SynMatching = HashMap::new();

        IRSZipper {
            path: self.path,
            child_type: ChildType::Root,
            var_child: self.var_child,
            var_children: self.var_children,
            children: self.children,
            rule_refs: self.rule_refs,
            matched: matching,
            response,
            end_node: self.end_node,
        }
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