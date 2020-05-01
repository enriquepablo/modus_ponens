use std::mem;
use std::clone::Clone;
use std::collections::HashMap;
use std::cell::{ RefCell };

use crate::path::SynPath;
use crate::matching::SynMatching;

#[derive(Debug, PartialEq)]
pub struct FSNode<'a> {
    children: RefCell<HashMap<SynPath<'a>, FSNode<'a>>>,  // XXX try putting keys and vals in boxes
    lchildren: RefCell<HashMap<SynPath<'a>, FSNode<'a>>>,
}

impl<'a> FSNode<'a> {
    pub fn new() -> FSNode<'a> {
        FSNode { 
            children: RefCell::new(HashMap::new()),
            lchildren: RefCell::new(HashMap::new()),
        }
    }
    pub fn get_child(&'a self, path: &'a SynPath) -> Option<&'a Self> {
        let children = self.children.borrow();
        let child = children.get(path);
        if child.is_none() {
            None
        } else {
            unsafe { mem::transmute(child) }
        }
    }
    pub fn intern_child(&'a self, path: &'a SynPath, node: Self) -> &'a Self {
        let mut children = self.children.borrow_mut();
        children.insert(path.clone(), node);
        let child = children.get(path).unwrap();
        unsafe { mem::transmute(child) }
    }
    pub fn get_lchild(&'a self, path: &'a SynPath) -> Option<&'a Self> {
        let lchildren = self.lchildren.borrow();
        let child = lchildren.get(path);
        if child.is_none() {
            None
        } else {
            unsafe { mem::transmute(child) }
        }
    }
    pub fn get_lchildren(&'a self) -> &'a HashMap<SynPath<'a>, FSNode<'a>> {
        let lchildren = self.lchildren.borrow();
        unsafe { mem::transmute(&*lchildren) }
    }
    pub fn get_children(&'a self) -> &'a HashMap<SynPath<'a>, FSNode<'a>> {
        let children = self.children.borrow();
        unsafe { mem::transmute(&*children) }
    }
    pub fn intern_lchild(&'a self, path: &'a SynPath, node: Self) -> &'a Self {
        let mut lchildren = self.lchildren.borrow_mut();
        lchildren.insert(path.clone(), node);
        let child = lchildren.get(path).unwrap();
        unsafe { mem::transmute(child) }
    }
    pub fn follow_and_create_paths(&'a self, paths: &'a [SynPath]) {
        let mut parent = self;
        let mut child: &FSNode;
        for (path_index, path) in paths.iter().enumerate() {
            if path.value.is_empty {
                continue;
            }
            if path.value.in_var_range {
                let opt_child = parent.get_lchild(path);
                let new_paths = path.paths_after(paths, true);
                if opt_child.is_some() {
                    child = opt_child.expect("node");
                    if !path.value.is_leaf {
                        child.follow_and_create_paths(new_paths);
                        continue;
                    }
                } else if path.value.is_leaf {
                    parent.create_paths(&paths[path_index..]);
                    return;
                } else {
                    let child_node = FSNode {
                        children: RefCell::new(HashMap::new()),
                        lchildren: RefCell::new(HashMap::new()),
                    };
                    if path.value.in_var_range {
                        child = parent.intern_lchild(path, child_node);
                    } else {
                        child = parent.intern_child(path, child_node);
                    };
                    let renew_paths = path.paths_after(&new_paths, true);
                    child.create_paths(&renew_paths);
                    parent.create_paths(&new_paths);
                    continue;
                }
            } else {
                let opt_child = parent.get_child(path);
                if opt_child.is_none() {
                    parent.create_paths(&paths[path_index..]);
                    return;
                } else {
                    child = opt_child.expect("node");
                }
            }
            parent = child;
        }
    }

    fn create_paths(&'a self, paths: &'a [SynPath]) {
        let mut parent = self;
        let mut child: &FSNode;
        for path in paths {
            if path.value.is_empty {
                continue;
            }
            let child_node = FSNode {
                children: RefCell::new(HashMap::new()),
                lchildren: RefCell::new(HashMap::new()),
            };
            let logic_node = path.value.in_var_range;
            if logic_node {
                child = parent.intern_lchild(path, child_node);
            } else {
                child = parent.intern_child(path, child_node);
            };
            if path.value.in_var_range && !path.value.is_leaf {
                let new_paths = path.paths_after(&paths, true);
                child.create_paths(new_paths);
                continue;
            }
            parent = child;
        }
    }
    pub fn query_paths(&'a self,
                   mut all_paths: &'a [SynPath],
                   matching: SynMatching<'a>,
                   mut resp: Vec<SynMatching<'a>>,
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
            let mut subs_path: Option<SynPath> = None;
            let path = next_path.unwrap();
            let paths = next_paths.unwrap();
            if path.value.is_var {
                if !matching.contains_key(&path.value) {
                    let lchildren = self.get_lchildren();
                    for (lchild_path, lchild_node) in lchildren.iter()  {
                        let mut new_matching = matching.clone();
                        new_matching.insert(path.value, lchild_path.value);
                        resp = lchild_node.query_paths(paths, new_matching, resp);
                    }
                    return resp;
                } else {
                    let (new_path, _) = path.substitute_owning(matching.clone());
                    subs_path = Some(new_path);
                }
            }
            let next: Option<&FSNode>;
            let new_path: SynPath;
            if subs_path.is_some() {
                new_path = subs_path.unwrap();
            } else {
                new_path = path.clone();
            }
            if new_path.value.in_var_range {
                let parent_lchildren = self.get_lchildren();
                next = parent_lchildren.get(&new_path);
            } else {
                let parent_children = self.get_children();
                next = parent_children.get(&new_path);
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
