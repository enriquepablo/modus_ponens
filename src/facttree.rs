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
use std::collections::HashMap;
use std::cell::{ RefCell };
use std::mem;

use bumpalo::{Bump};

use crate::constants;
use crate::path::MPPath;
use crate::matching::MPMatching;


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
    children: RefCell<HashMap<MPPath<'a>, &'a FSNode<'a>>>,
    lchildren: RefCell<HashMap<MPPath<'a>, &'a FSNode<'a>>>,
}

pub struct FactSet<'a> {
    pub root: Box<FSNode<'a>>,
    nodes: Bump,
}


impl<'a> FactSet<'a> {
    pub fn new () -> FactSet<'a> {
        FactSet {
            root: Box::new(FSNode::new(1)),
            nodes: Bump::new(),
         }
    }
    pub fn add_fact (&'a self, fact: Vec<MPPath<'a>>) {
        let carry = CarryOver(HashMap::new());
        self.follow_and_create_paths(&self.root, fact, 1, carry);
    }
    pub fn ask_fact (&'a self, fact: Vec<MPPath<'a>>) -> (Vec<MPMatching<'a>>, Vec<MPPath<'a>>) {
        let mut response: Vec<MPMatching> = vec![];
        let matching: MPMatching = HashMap::new();
        let paths: &[MPPath] = unsafe { mem::transmute( fact.as_slice() ) };
        response = self.root.query_paths(paths, matching, response);
        (response, fact)
    }
    pub fn ask_fact_bool (&'a self, fact: Vec<MPPath<'a>>) -> (bool, Vec<MPPath<'a>>) {
        let (resp, fact) = self.ask_fact(fact);
        (resp.len() > 1, fact)
    }
    pub fn follow_and_create_paths(&'a self,
                                   mut parent: &'a FSNode<'a>,
                                   mut paths: Vec<MPPath<'a>>,
                                   mut depth: usize,
                                   mut carry: CarryOver<'a>,) {
        let mut child: &FSNode;
        let mut path_index = 0;
        while paths.len() > 0 {
            let path = paths.remove(0);
            if path.value.is_empty {
                path_index += 1;
                continue;
            }
            depth += 1;
            if path.value.in_var_range {
                let (opt_child, path) = parent.get_lchild(path);
                let reindex = path.paths_after(&paths);
                if opt_child.is_some() {
                    child = opt_child.expect("node");
                    if !path.value.is_leaf {
                        carry = carry.add(reindex, child);
                        path_index += 1;
                        continue;
                    }
                } else if path.value.is_leaf {
                    paths.insert(0, path);
                    self.create_paths(parent, paths, depth, carry, path_index);
                    return;
                } else {
                    let child_node = FSNode::new(depth);
                    let (new_child, new_carry) = self.intern_lchild(parent, path, child_node, carry, path_index);
                    child = new_child;
                    carry = new_carry;

                    carry = carry.add(reindex, child);
                    path_index += 1;
                    continue;
                }
            } else {
                let (opt_child, path) = parent.get_child(path);
                if opt_child.is_none() {
                    paths.insert(0, path);
                    self.create_paths(parent, paths, depth, carry, path_index);
                    return;
                } else {
                    child = opt_child.expect("node");
                }
            }
            parent = child;
            path_index += 1;
        }
    }

    fn create_paths(&'a self,
                    mut parent: &'a FSNode<'a>,
                    mut paths: Vec<MPPath<'a>>,
                    mut depth: usize,
                    mut carry: CarryOver<'a>,
                    offset: usize,) {
        let mut child: &FSNode;
        let mut path_index = 0;
        while paths.len() > 0 {
            let path = paths.remove(0);
            if path.value.is_empty {
                path_index += 1;
                continue;
            }
            depth += 1;
            let child_node = FSNode::new(depth);
            let logic_node = path.value.in_var_range;
            let real_index = path_index + offset;
            let reindex = path.paths_after(&paths);
            let is_leaf = path.value.is_leaf;
            if logic_node {
                let (new_child, new_carry) = self.intern_lchild(parent, path, child_node, carry, real_index);
                child = new_child;
                carry = new_carry;
            } else {
                let (new_child, new_carry) = self.intern_child(parent, path, child_node, carry, real_index);
                child = new_child;
                carry = new_carry;
            };
            if logic_node && !is_leaf {
                carry = carry.add(reindex, child);
                path_index += 1;
                continue;
            }
            parent = child;
            path_index += 1;
        }
    }
    pub fn intern_child(&'a self,
                        parent: &'a FSNode<'a>,
                        path: MPPath<'a>,
                        child: FSNode<'a>,
                        mut carry: CarryOver<'a>,
                        index: usize,
                    ) -> (&'a FSNode<'a>, CarryOver<'a>) {

        let child_ref = self.nodes.alloc(child);
        let (new_carry, more) = carry.node(index);
        carry = new_carry;
        if more.is_some() {
            more.unwrap().children.borrow_mut().insert(path.clone(), child_ref);
        }
        parent.children.borrow_mut().insert(path, child_ref);
        (child_ref, carry)
    }
    pub fn intern_lchild(&'a self,
                         parent: &'a FSNode<'a>,
                         path: MPPath<'a>,
                         child: FSNode<'a>,
                         mut carry: CarryOver<'a>,
                         index: usize,
                        ) -> (&'a FSNode<'a>, CarryOver<'a>) {
        let child_ref = self.nodes.alloc(child);
        let (new_carry, more) = carry.node(index);
        carry = new_carry;
        if more.is_some() {
            more.unwrap().lchildren.borrow_mut().insert(path.clone(), child_ref);
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
    pub fn get_child(&'a self, path: MPPath<'a>) -> (Option<&'a Self>, MPPath<'a>) {
        let children = self.children.borrow();
        match children.get(&path) {
            None => (None, path),
            Some(child_ref) => (Some(*child_ref), path)
        }
    }
    pub fn get_lchild(&'a self, path: MPPath<'a>) -> (Option<&'a Self>, MPPath<'a>) {
        let children = self.lchildren.borrow();
        match children.get(&path) {
            None => (None, path),
            Some(child_ref) => (Some(*child_ref), path)
        }
    }
    pub fn get_child_r(&'a self, path: &'a MPPath<'a>) -> Option<&'a Self> {
        let children = self.children.borrow();
        match children.get(path) {
            None => None,
            Some(child_ref) => Some(*child_ref)
        }
    }
    pub fn get_lchild_r(&'a self, path: &'a MPPath<'a>) -> Option<&'a Self> {
        let children = self.lchildren.borrow();
        match children.get(path) {
            None => None,
            Some(child_ref) => Some(*child_ref)
        }
    }
    pub fn query_paths(&'a self,
                   mut all_paths: &'a [MPPath],
                   matching: MPMatching<'a>,
                   mut resp: Vec<MPMatching<'a>>,
                   ) -> Vec<MPMatching<'a>> {

        let mut finished = false;
        let mut next_path: Option<&MPPath> = None;
        let mut next_paths: Option<&'a [MPPath]> = None;
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
            let mut subs_path: Option<&MPPath> = None;
            let path = next_path.unwrap();
            let paths = next_paths.unwrap();
            if path.value.is_var {
                if !matching.contains_key(&path.value) {
                    for (lchild_path, lchild_node) in self.lchildren.borrow().iter()  {
                        let mut new_matching = matching.clone();
                        new_matching.insert(path.value, lchild_path.value);
                        resp = lchild_node.query_paths(paths, new_matching, resp);
                    }
                    return resp;
                } else {
                    let (new_path, _) = path.substitute_owning(matching.clone());
                    let new_path_ref = unsafe { mem::transmute(&new_path) };
                    subs_path = Some(new_path_ref);
                }
            }
            let next: Option<&FSNode>;
            let new_path: &MPPath;
            if subs_path.is_some() {
                new_path = subs_path.unwrap();
            } else {
                new_path = path;
            }
            if new_path.value.in_var_range {
                next = self.get_lchild_r(new_path);
            } else {
                next = self.get_child_r(new_path);
            }
            if next.is_some() {
                let next_node = next.unwrap();
                resp = next_node.query_paths(paths, matching, resp);
            }
        } else {
            resp.push(matching);
        }
        resp
    }
}
