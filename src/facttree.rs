use std::clone::Clone;
use std::collections::HashMap;
use std::cell::{ RefCell };

use bumpalo::{Bump};

use crate::constants;
use crate::fact::Fact;
use crate::path::SynPath;
use crate::matching::SynMatching;


pub struct CarryOver<'a>(HashMap<usize, &'a FSNode<'a>>);

impl<'a> CarryOver<'a> {
    pub fn add (mut self, index: usize, node: &'a FSNode<'a>) -> Self {
        self.0.insert(index, node);
        self
    }
    pub fn node (mut self, index: usize) -> (Self, Option<&'a FSNode<'a>>) {
        let node_opt = self.0.remove(&index);
        (self, node_opt)
    }
}

#[derive(Debug, PartialEq)]
pub struct FSNode<'a> {
    children: RefCell<HashMap<&'a SynPath<'a>, &'a FSNode<'a>>>,  // XXX try putting keys and vals in boxes
    lchildren: RefCell<HashMap<&'a SynPath<'a>, &'a FSNode<'a>>>,
}

pub struct FactSet<'a> {
    pub root: Box<FSNode<'a>>,
    nodes: Bump,
    paths: Bump,
}


impl<'a> FactSet<'a> {
    pub fn new () -> FactSet<'a> {
        FactSet {
            root: Box::new(FSNode::new(1)),
            nodes: Bump::new(),
            paths: Bump::new(),
         }
    }
    pub fn add_fact (&'a self, fact: &'a Fact<'a>) {
        let paths = fact.paths.as_slice();
        let carry = CarryOver(HashMap::new());
        self.follow_and_create_paths(&self.root, paths, 1, carry);
    }
    pub fn ask_fact (&'a self, fact: &'a Fact) -> Vec<SynMatching<'a>> {
        let response: Vec<SynMatching<'a>> = vec![];
        let paths = fact.paths.as_slice();
        let matching: SynMatching = HashMap::new();
        self.root.query_paths(paths, matching, response, &self.paths)
    }
    pub fn ask_fact_bool (&'a self, fact: &'a Fact) -> bool {
        self.ask_fact(fact).len() > 0
    }
    pub fn follow_and_create_paths(&'a self,
                                   mut parent: &'a FSNode<'a>,
                                   paths: &'a [SynPath],
                                   mut depth: usize,
                                   mut carry: CarryOver<'a>,) {
        let mut child: &FSNode;
        for (path_index, path) in paths.iter().enumerate() {
            if path.value.is_empty {
                continue;
            }
            depth += 1;
            if path.value.in_var_range {
                let opt_child = parent.get_lchild(path);
                let reindex = path.paths_after(paths);
                if opt_child.is_some() {
                    child = opt_child.expect("node");
                    if !path.value.is_leaf {
                        carry = carry.add(reindex, child);
                        continue;
                    }
                } else if path.value.is_leaf {
                    self.create_paths(parent, &paths[path_index..], depth, carry, path_index);
                    return;
                } else {
                    let child_node = FSNode::new(depth);
                    let (new_child, new_carry) = self.intern_lchild(parent, path, child_node, carry, path_index);
                    child = new_child;
                    carry = new_carry;

                    carry = carry.add(reindex, child);
                    continue;
                }
            } else {
                let opt_child = parent.get_child(path);
                if opt_child.is_none() {
                    self.create_paths(parent, &paths[path_index..], depth, carry, path_index);
                    return;
                } else {
                    child = opt_child.expect("node");
                }
            }
            parent = child;
        }
    }

    fn create_paths(&'a self,
                    mut parent: &'a FSNode<'a>,
                    paths: &'a [SynPath],
                    mut depth: usize,
                    mut carry: CarryOver<'a>,
                    offset: usize,) {
        let mut child: &FSNode;
        for (path_index, path) in paths.iter().enumerate() {
            if path.value.is_empty {
                continue;
            }
            depth += 1;
            let child_node = FSNode::new(depth);
            let logic_node = path.value.in_var_range;
            let real_index = path_index + offset;
            if logic_node {
                let (new_child, new_carry) = self.intern_lchild(parent, path, child_node, carry, real_index);
                child = new_child;
                carry = new_carry;
            } else {
                let (new_child, new_carry) = self.intern_child(parent, path, child_node, carry, real_index);
                child = new_child;
                carry = new_carry;
            };
            let reindex = path.paths_after(&paths);
            if path.value.in_var_range && !path.value.is_leaf {
                carry = carry.add(reindex, child);
                continue;
            }
            parent = child;
        }
    }
    pub fn intern_child(&'a self,
                        parent: &'a FSNode<'a>,
                        path: &'a SynPath<'a>,
                        child: FSNode<'a>,
                        mut carry: CarryOver<'a>,
                        index: usize,
                    ) -> (&'a FSNode<'a>, CarryOver<'a>) {

        let child_ref = self.nodes.alloc(child);
        let (new_carry, more) = carry.node(index);
        carry = new_carry;
        if more.is_some() {
            more.unwrap().children.borrow_mut().insert(path, child_ref);
        }
        parent.children.borrow_mut().insert(path, child_ref);
        (child_ref, carry)
    }
    pub fn intern_lchild(&'a self,
                         parent: &'a FSNode<'a>,
                         path: &'a SynPath<'a>,
                         child: FSNode<'a>,
                         mut carry: CarryOver<'a>,
                         index: usize,
                        ) -> (&'a FSNode<'a>, CarryOver<'a>) {
        let child_ref = self.nodes.alloc(child);
        let (new_carry, more) = carry.node(index);
        carry = new_carry;
        if more.is_some() {
            more.unwrap().lchildren.borrow_mut().insert(path, child_ref);
        }
        parent.lchildren.borrow_mut().insert(path, child_ref);
        (child_ref, carry)
    }
}

impl<'a> FSNode<'a> {
    pub fn new(depth: usize) -> FSNode<'a> {
        let capacity = constants::NODE_MAP_CAPACITY / depth;
        FSNode { 
            children: RefCell::new(HashMap::with_capacity(capacity)),
            lchildren: RefCell::new(HashMap::with_capacity(capacity)),
        }
    }
    pub fn get_child(&'a self, path: &'a SynPath) -> Option<&'a Self> {
        let children = self.children.borrow();
        match children.get(path) {
            None => None,
            Some(child_ref) => Some(*child_ref)
        }
    }
    pub fn get_lchild(&'a self, path: &'a SynPath) -> Option<&'a Self> {
        let children = self.lchildren.borrow();
        match children.get(path) {
            None => None,
            Some(child_ref) => Some(*child_ref)
        }
    }
    pub fn query_paths(&'a self,
                   mut all_paths: &'a [SynPath],
                   matching: SynMatching<'a>,
                   mut resp: Vec<SynMatching<'a>>,
                   arena: &'a Bump
                   ) -> Vec<SynMatching<'a>> {

        let mut finished = false;
        let mut next_path: Option<&SynPath> = None;
        let mut next_paths: Option<&'a [SynPath]> = None;
        while !finished {
            let split_paths = all_paths.split_first();
            if split_paths.is_some() {
                let (path, paths) = split_paths.unwrap();
                if !path.value.is_empty && path.value.is_leaf {
                    finished = true;
                    next_path = Some(path);
                    next_paths = Some(paths);
                } else {
                    all_paths = paths;
                }
            } else {
                finished = true;
            }

        }
        if next_path.is_some(){
            let mut subs_path: Option<&SynPath> = None;
            let path = next_path.unwrap();
            let paths = next_paths.unwrap();
            if path.value.is_var {
                if !matching.contains_key(&path.value) {
                    for (lchild_path, lchild_node) in self.lchildren.borrow().iter()  {
                        let mut new_matching = matching.clone();
                        new_matching.insert(path.value, lchild_path.value);
                        resp = lchild_node.query_paths(paths, new_matching, resp, arena);
                    }
                    return resp;
                } else {
                    let (new_path, _) = path.substitute_owning(matching.clone());
                    let new_path_ref = arena.alloc(new_path);
                    subs_path = Some(new_path_ref);
                }
            }
            let next: Option<&FSNode>;
            let new_path: &SynPath;
            if subs_path.is_some() {
                new_path = subs_path.unwrap();
            } else {
                new_path = path;
            }
            if new_path.value.in_var_range {
                next = self.get_lchild(new_path);
            } else {
                next = self.get_child(new_path);
            }
            if next.is_some() {
                let next_node = next.unwrap();
                resp = next_node.query_paths(paths, matching, resp, arena);
            }
        } else {
            resp.push(matching);
        }
        resp
    }
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::segment::SynSegment;
//
//    #[test]
//    fn factset_1() {
//        let mut factset = FSNode::new();
//
//        let segm11 = SynSegment::new("rule-name1".to_string(), "(text)".to_string(), false);
//        let segms1 = vec![&segm11];
//        let path1 = SynPath::new(segms1);
//        let cpath1 = path1.clone();
//
//        let node1 = FSNode::new();
//        
//        factset.children.borrow_mut().insert(cpath1, node1);
//
//        let segm21 = SynSegment::new("rule-name1".to_string(), "(text)".to_string(), false);
//        let segm22 = SynSegment::new("rule-name2".to_string(), "(".to_string(), true);
//        let segms2 = vec![&segm21, &segm22];
//        let path2 = SynPath::new(segms2);
//        let cpath2 = path2.clone();
//
//        let node2 = FSNode::new();
//        let rnode1 = factset.children.borrow().get_mut(&path1).expect("path");
//        rnode1.children.borrow_mut().insert(cpath2, node2);
//
//        let segm31 = SynSegment::new("rule-name1".to_string(), "(text)".to_string(), false);
//        let segm32 = SynSegment::new("rule-name3".to_string(), "text".to_string(), true);
//        let segms3 = vec![&segm31, &segm32];
//        let path3 = SynPath::new(segms3);
//        let cpath3 = path3.clone();
//
//        let node3 = FSNode::new();
//        let rnode2 = rnode1.children.borrow().get_mut(&path2).expect("path");
//        rnode2.children.insert(cpath3, node3);
//
//        let segm41 = SynSegment::new("rule-name1".to_string(), "(text)".to_string(), false);
//        let segm42 = SynSegment::new("rule-name4".to_string(), ")".to_string(), true);
//        let segms4 = vec![&segm41, &segm42];
//        let path4 = SynPath::new(segms4);
//        let cpath4 = path4.clone();
//
//        let node4 = FSNode::new();
//        let rnode3 = rnode2.children.get_mut(&path3).expect("path");
//        rnode3.children.insert(cpath4, node4);
//
//        let paths = vec![&path1, &path2, &path3, &path4];
//
//        let leaf = factset.get_leaf(&paths);
//
//        assert!(!leaf.is_none());
//    }
//}
