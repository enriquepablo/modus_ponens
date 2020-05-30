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
use std::mem;
//use std::collections::VecDeque;
use std::cell::{ RefCell, Cell };

use bumpalo::{Bump};
//use log::debug;

use crate::constants;
use crate::path::MPPath;
use crate::segment::MPSegment;
use crate::matching::MPMatching;


pub type Response<'a> = Vec<(&'a RefCell<Vec<RuleRef<'a>>>, MPMatching<'a>)>;

pub fn new_response<'a>() -> Response<'a> {
    vec![]
}

#[derive(Debug, Clone)]
pub struct Antecedents<'a> {
    pub fact: Option<&'a str>,
    pub transforms: &'a str,
    pub conditions: &'a str,
}

#[derive(Debug, Clone)]
pub struct MPRule<'a> {
    pub antecedents: Antecedents<'a>,
    pub more_antecedents: VecDeque<Antecedents<'a>>,
    pub consequents: Vec<&'a str>,
    pub matched: MPMatching<'a>,
    pub output: Option<&'a str>,
}

impl<'a> fmt::Display for MPRule<'a> {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string = String::from("rule\n\n");
        string.push_str(
            match self.antecedents.fact {
                Some(fact) => fact,
                None => "Empty",
            }
        );
        string.push_str("\n{={\n");
        string.push_str(self.antecedents.transforms);
        string.push_str("}=}\n{?{\n");
        string.push_str(self.antecedents.conditions);
        string.push_str("\n}?}\n->\n");
        for more_ants in &self.more_antecedents {
            if more_ants.fact.is_some() {
                string.push_str(more_ants.fact.unwrap());
            }
            string.push_str("{={\n");
            string.push_str(more_ants.transforms);
            string.push_str("}=}\n{?{\n");
            string.push_str(more_ants.conditions);
            string.push_str("}?}\n->\n");
        }

        for consequent in &self.consequents {
            string.push_str(consequent);
            string.push_str(" ;\n");
        }

        if self.output.is_some() {
            string.push_str(" {!{ ");
            string.push_str(self.output.unwrap());
            string.push_str(" }!}\n<>");
        }

        string.push_str(&format!("\n\nmatching:\n{:?}", &self.matched));

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
    paths: Bump,
}

impl<'a> RuleSet<'a> {

    pub fn new(root_path: MPPath<'a>) -> Self {
        let root_path_ref = Box::leak(Box::new(root_path));
        let root = RSNode::new(root_path_ref, 1);
        RuleSet {
            root,
            nodes: Bump::new(),
            paths: Bump::new(),
        }
    }
    pub fn follow_and_create_paths(&'a self, mut paths: Vec<MPPath<'a>>, rule_ref: RuleRef<'a>, mut depth: usize) {
        let mut parent: &RSNode = &self.root; 
        let mut child: Option<&RSNode>; 
        let mut visited_vars: Vec<&MPSegment> = vec![];
        // XXX probably just a for loop?
        while paths.len() > 0 {
            let mut new_path = paths.remove(0);
            if new_path.value.is_empty || !new_path.value.is_leaf {
                continue;
            }
            depth += 1;
            if new_path.value.is_var {
                let (new_child, old_path) = parent.get_vchild_o(new_path);
                new_path = old_path;
                child = new_child;
                if child.is_none() {
                    let var_child_opt = parent.get_var_child();
                    if var_child_opt.is_some() {
                        let var_child =  var_child_opt.unwrap();
                        if var_child.path == &new_path {
                            visited_vars.push(new_path.value);
                            child = Some(var_child);
                        }
                    }
                }
            } else {
                let new_path_ref = unsafe { mem::transmute( &new_path ) };
                let new_child = parent.get_child(new_path_ref);
                child = new_child;
            }
            if child.is_some() {
                parent = child.unwrap();
            } else {
                paths.insert(0, new_path);
                parent = self.create_paths(parent, paths, visited_vars, depth);
                break;
            }
        }

        parent.end_node.set(true);
        parent.rule_refs.borrow_mut().push(rule_ref);
    }

    fn create_paths(&'a self, mut parent: &'a RSNode<'a>, mut paths: Vec<MPPath<'a>>, mut visited: Vec<&'a MPSegment>, mut depth: usize) -> &'a RSNode {
        while paths.len() > 0 {
            let new_path = paths.remove(0);
            if new_path.value.is_empty || !new_path.value.is_leaf {
                continue;
            }
            depth += 1;
            let val = new_path.value;
            let path_ref = self.paths.alloc(new_path);
            let child_ref = self.nodes.alloc(RSNode::new(path_ref, depth));
            if val.is_var {
                if visited.contains(&val) {
                    parent.var_children.borrow_mut().insert(path_ref, child_ref);
                } else {
                    visited.push(val);
                    parent.var_child.borrow_mut().node = Some(child_ref);
                }
            } else {
                parent.children.borrow_mut().insert(path_ref, child_ref);
            }
            parent = child_ref;
        }
        parent.end_node.set(true);
        parent
    }
    pub fn query_paths(&'a self, paths: Vec<MPPath<'a>>) -> (Response, Vec<MPPath<'a>>) {
        let response = new_response();
        let matched: MPMatching = HashMap::new();
        let paths_slice: &[MPPath] = unsafe { mem::transmute( paths.as_slice() ) };
        let (response, _) = self.root.climb(paths_slice, response, matched);
        (response, paths)
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
    pub fn get_child_o(&'a self, path: MPPath<'a>) -> (Option<&'a Self>, MPPath<'a>) {
        let children = self.children.borrow();
        match children.get(&path) {
            None => (None, path),
            Some(child_ref) => {
                (Some(*child_ref), path)
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
    pub fn get_vchild_o(&'a self, path: MPPath<'a>) -> (Option<&'a Self>, MPPath<'a>) {
        let vchildren = self.var_children.borrow();
        match vchildren.get(&path) {
            None => (None, path),
            Some(child_ref) => {
                (Some(*child_ref), path)
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
