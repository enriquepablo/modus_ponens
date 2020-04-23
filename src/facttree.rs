use std::clone::Clone;
use std::collections::HashMap;
use std::cell::{ RefCell, RefMut };

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
}

#[derive(Debug)]
pub struct NodeZipper<'a> {
    parent: Option<Box<NodeZipper<'a>>>,
    path_in_parent: Option<&'a SynPath<'a>>,
    logic_node: bool,
    children: &'a RefCell<HashMap<SynPath<'a>, FSNode<'a>>>,
    lchildren: &'a RefCell<HashMap<SynPath<'a>, FSNode<'a>>>,
}

impl<'a> NodeZipper<'a> {
    

    fn get_parent(self) -> Option<NodeZipper<'a>> {
        // Destructure this NodeZipper
        let NodeZipper {
            parent, ..
        } = self;

        if parent.is_none() {
            None
        } else {
            Some(*parent.unwrap())
        }
    }

    fn ancestor(mut self, n: usize) -> NodeZipper<'a> {
        for _ in 0..n {
            self = self.get_parent().expect("parent node");
        }
        self
    }
    
    fn get_child(mut self, path: &'a SynPath, logic: bool) -> (Option<NodeZipper<'a>>, Option<NodeZipper<'a>>) {
        // Remove the specified child from the node's children.
        // A NodeZipper shouldn't let its users inspect its parent,
        // since we mutate the parents
        // to move the focused nodes out of their list of children.
        // We use swap_remove() for efficiency.
        let child_opt: Option<&FSNode>;
        if logic {
            child_opt = self.lchildren.borrow().get(path);
        } else {
            child_opt = self.children.borrow().get(path);
        }

        // Return a new NodeZipper focused on the specified child.
        if child_opt.is_none() {
            (None, Some(self))
        } else {

            let child = child_opt.unwrap();

            (Some(NodeZipper {
                parent: Some(Box::new(self)),
                path_in_parent: Some(path),
                logic_node: logic,
                children: &child.children,
                lchildren: &child.lchildren,
            }), None)
        }
    }

    pub fn follow_and_create_paths(self, paths: &'a [SynPath]) -> NodeZipper<'a> {
        let mut parent = self;
        let mut child: NodeZipper;
        let mut child_index = 0;
        for (path_index, path) in paths.iter().enumerate() {
            if path.value.is_empty {
                continue;
            }
            if path.value.in_var_range {
                let (opt_child, opt_zipper) = parent.get_child(path, true);
                let new_paths = path.paths_after(paths, true);
                if opt_child.is_some() {
                    child = opt_child.expect("node");
                    if !path.value.is_leaf {
                        child = child.follow_and_create_paths(new_paths);
                        parent = child.get_parent().expect("we set the parent");
                        continue;
                    }
                } else if path.value.is_leaf {
                    parent = opt_zipper.expect("node").create_paths(&paths[path_index..]);
                    return parent.ancestor(child_index);
                } else {
                    parent = opt_zipper.expect("node");
                    let child_node = FSNode {
                        children: RefCell::new(HashMap::new()),
                        lchildren: RefCell::new(HashMap::new()),
                    };
                    let parent_col: RefMut<HashMap<SynPath<'a>, FSNode<'a>>>;
                    let logic_node = path.value.in_var_range;
                    if logic_node {
                        parent_col = parent.lchildren.borrow_mut();
                    } else {
                        parent_col = parent.children.borrow_mut();
                    };
                    parent_col.insert(path.clone(), child_node);
                    let child_node_ref = parent_col.get(path).unwrap();

                    child = NodeZipper {
                        parent: Some(Box::new(parent)),
                        path_in_parent: Some(path),
                        logic_node,
                        children: &child_node_ref.children,
                        lchildren: &child_node_ref.lchildren,
                    };
                    let renew_paths = path.paths_after(&new_paths, true);
                    child = child.create_paths(&renew_paths);
                    parent = child.get_parent().expect("we set the parent");
                    parent = parent.create_paths(&new_paths);
                    continue;
                }
            } else {
                let (opt_child, opt_zipper) = parent.get_child(path, false);
                if opt_child.is_none() {
                    parent = opt_zipper.expect("node").create_paths(&paths[path_index..]);
                    return parent.ancestor(child_index);
                } else {
                    child = opt_child.expect("node");
                }
            }
            parent = child;
            child_index += 1;
        }
        parent.ancestor(child_index)
    }

    fn create_paths(self, paths: &'a [SynPath]) -> NodeZipper<'a> {
        let mut parent = self;
        let mut child: NodeZipper;
        let mut child_index = 0;
        for path in paths {
            if path.value.is_empty {
                continue;
            }
            let child_node = FSNode {
                children: RefCell::new(HashMap::new()),
                lchildren: RefCell::new(HashMap::new()),
            };
            let parent_col: RefMut<HashMap<SynPath<'a>, FSNode<'a>>>;
            let logic_node = path.value.in_var_range;
            if logic_node {
                parent_col = parent.lchildren.borrow_mut();
            } else {
                parent_col = parent.children.borrow_mut();
            };
            parent_col.insert(path.clone(), child_node);
            let child_node_ref = parent_col.get(path).unwrap();
            child = NodeZipper {
                parent: Some(Box::new(parent)),
                path_in_parent: Some(path),
                logic_node: path.value.in_var_range,
                children: &child_node_ref.children,
                lchildren: &child_node_ref.lchildren,
            };
            if path.value.in_var_range && !path.value.is_leaf {
                let new_paths = path.paths_after(&paths, true);
                child = child.create_paths(new_paths);
                parent = child.get_parent().expect("we set the parent");
                continue;
            }
            parent = child;
            child_index += 1;
        }
        parent.ancestor(child_index)
    }
}

#[derive(Debug)]
pub struct INodeZipper<'a> {
    children: &'a RefCell<HashMap<SynPath<'a>, FSNode<'a>>>,
    lchildren: &'a RefCell<HashMap<SynPath<'a>, FSNode<'a>>>,
    response: Vec<SynMatching<'a>>,
}

impl<'a> INodeZipper<'a> {
    
    pub fn query_paths(self,
                   mut all_paths: &'a [SynPath],
                   matching: SynMatching<'a>,
                   ) -> INodeZipper<'a> {

        let INodeZipper {
            children: mut parent_children_cell,
            lchildren: mut parent_lchildren_cell,
            response: mut resp,
        } = self;
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
                    let mut child: INodeZipper;
                    let parent_lchildren = parent_lchildren_cell.borrow();
                    for lchild_path in parent_lchildren.keys()  {
                        let lchild_node = parent_lchildren.get(&lchild_path).unwrap();
                        child = INodeZipper {
                            children: &lchild_node.children,
                            lchildren: &lchild_node.lchildren,
                            response: resp,
                        };
                        let mut new_matching = matching.clone();
                        new_matching.insert(path.value, lchild_path.value);
                        child = child.query_paths(paths, new_matching);
                        let INodeZipper {
                            response: new_response, ..
                        } = child;
                        resp = new_response;
                    }
                    return INodeZipper {
                        children: parent_children_cell,
                        lchildren: parent_lchildren_cell,
                        response: resp,
                    };
                } else {
                    let (new_path, _) = path.substitute_owning(matching.clone());
                    subs_path = Some(new_path);
                }
            }
            let next: Option<&FSNode>;
            let logic: bool;
            let new_path: SynPath;
            if subs_path.is_some() {
                new_path = subs_path.unwrap();
            } else {
                new_path = path.clone();
            }
            if new_path.value.in_var_range {
                let parent_lchildren = parent_lchildren_cell.borrow();
                next = parent_lchildren.get(&new_path);
                logic = true;
            } else {
                let parent_children = parent_children_cell.borrow();
                next = parent_children.get(&new_path);
                logic = false;
            }
            if next.is_some() {
                let next_node = next.unwrap();
                let mut next_child = INodeZipper {
                    children: &next_node.children,
                    lchildren: &next_node.lchildren,
                    response: resp,
                };
                next_child = next_child.query_paths(paths, matching);
                let INodeZipper {
                    response: new_response, ..
                } = next_child;
                resp = new_response;
            }
        } else {
            resp.push(matching);
        }
        INodeZipper {
            children: parent_children_cell,
            lchildren: parent_lchildren_cell,
            response: resp,
        }
    }

    pub fn finish(self) -> Vec<SynMatching<'a>> {
        
        let INodeZipper {
            response, ..
        } = self;
        response
    }
}


impl<'a> FSNode<'a> {
    pub fn zipper(&'a self) -> NodeZipper<'a> {
          
        NodeZipper {
            parent: None,
            path_in_parent: None,
            logic_node: false,
            children: &self.children,
            lchildren: &self.lchildren,
        }
    }

    pub fn qzipper(&'a self, response: Vec<SynMatching<'a>>) -> INodeZipper<'a> {
        INodeZipper {
            children: &self.children,
            lchildren: &self.lchildren,
            response,
        }
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
