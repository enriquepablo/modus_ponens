use std::clone::Clone;
use std::collections::HashMap;

use crate::path::SynPath;
use crate::matching::SynMatching;

#[derive(Debug, PartialEq)]
pub struct FSNode {
    children: HashMap<SynPath, FSNode>,
    lchildren: HashMap<SynPath, FSNode>,
}

impl<'a> FSNode {
    pub fn new() -> FSNode {
        FSNode { 
            children: HashMap::new(),
            lchildren: HashMap::new(),
        }
    }

    fn add_child(&mut self, path: SynPath) {
        let child: FSNode = FSNode::new();
        self.children.insert(path, child);
    }

    fn add_lchild(&mut self, path: SynPath) {
        let child: FSNode = FSNode::new();
        self.lchildren.insert(path, child);
    }

    fn get_leaf(self, paths: &'a [&SynPath]) -> Option<NodeZipper<'a>> {
        let zipper = self.zipper();
        zipper.get_leaf(paths)
    }
}

#[derive(Debug)]
pub struct NodeZipper<'a> {
    parent: Option<Box<NodeZipper<'a>>>,
    path_in_parent: Option<&'a SynPath>,
    logic_node: bool,
    children: HashMap<SynPath, FSNode>,
    lchildren: HashMap<SynPath, FSNode>,
}

impl<'a> NodeZipper<'a> {
    
    fn get(mut self, path: &'a SynPath) -> Option<NodeZipper<'a>> {
        // Destructure this NodeZipper
        let NodeZipper {
            parent,
            path_in_parent,
            logic_node,
            mut children,
            mut lchildren,
        } = self;

        let mut child = children.remove(path);

        if child.is_none() {
            child = lchildren.remove(path);
        }
        // Return a new NodeZipper focused on the specified child.
        if child.is_none() {
            None
        } else {
            let FSNode {
                children: child_children,
                lchildren: child_lchildren,
            } = child.unwrap();
            self = NodeZipper {
                parent,
                path_in_parent,
                logic_node,
                children,
                lchildren,
            };
            Some(NodeZipper {
                parent: Some(Box::new(self)),
                path_in_parent: Some(path),
                logic_node: false,
                children: child_children,
                lchildren: child_lchildren,
            })
        }
    }

    fn get_leaf(self, paths: &'a [&SynPath]) -> Option<NodeZipper<'a>> {
        let mut node = Some(self);
        for path in paths {
            node = node.expect("node").get(path);
            if node.is_none() {
                return None
            }
        }
        node
    }

    fn get_parent(self) -> Option<NodeZipper<'a>> {
        // Destructure this NodeZipper
        let NodeZipper {
            parent,
            path_in_parent,
            logic_node,
            children,
            lchildren,
        } = self;

        // Destructure the parent NodeZipper
        let NodeZipper {
            parent: parent_parent,
            path_in_parent: parent_path_in_parent,
            logic_node: parent_logic_node,
            children: mut parent_children,
            lchildren: mut parent_lchildren,
        } = *parent.unwrap();

        // Insert the node of this NodeZipper back in its parent.
        if path_in_parent.is_none() {
            None
        } else {
            let node = FSNode {children, lchildren};
            let ppc = path_in_parent.expect("path").clone();    
            if logic_node {
                parent_lchildren.insert(ppc, node);
            } else {
                parent_children.insert(ppc, node);
            }
            // Return a new NodeZipper focused on the parent.
            Some(NodeZipper {
                parent: parent_parent,
                path_in_parent: parent_path_in_parent,
                logic_node: parent_logic_node,
                children: parent_children,
                lchildren: parent_lchildren,
            })
        }
    }

    pub fn finish(mut self) -> Box<FSNode> {
        while let Some(_) = self.parent {
            self = self.get_parent().expect("parent node");
        }
        let NodeZipper {
            children,
            lchildren, ..
        } = self;

        Box::new(FSNode {
            children,
            lchildren,
        })
    }

    fn ancestor(mut self, n: usize) -> NodeZipper<'a> {
        for _ in 0..n {
            self = self.get_parent().expect("parent node");
        }
        self
    }
    
    fn has_child(self, path: &'a SynPath, l: bool) -> bool {
        // Remove the specified child from the node's children.
        // A NodeZipper shouldn't let its users inspect its parent,
        // since we mutate the parents
        // to move the focused nodes out of their list of children.
        // We use swap_remove() for efficiency.
        if l {
            self.lchildren.contains_key(path)
        } else {
            self.children.contains_key(path)
        }
    }
    
    fn get_child(mut self, path: &'a SynPath, l: bool) -> (Option<NodeZipper<'a>>, Option<NodeZipper<'a>>) {
        // Remove the specified child from the node's children.
        // A NodeZipper shouldn't let its users inspect its parent,
        // since we mutate the parents
        // to move the focused nodes out of their list of children.
        // We use swap_remove() for efficiency.
        let child: Option<FSNode>;
        if l {
            child = self.lchildren.remove(path);
        } else {
            child = self.children.remove(path);
        }

        // Return a new NodeZipper focused on the specified child.
        if child.is_none() {
            (None, Some(self))
        } else {
            let FSNode {
                children: child_children,
                lchildren: child_lchildren,
            } = child.unwrap();

            (Some(NodeZipper {
                parent: Some(Box::new(self)),
                path_in_parent: Some(path),
                logic_node: true,
                children: child_children,
                lchildren: child_lchildren,
            }), None)
        }
    }

    pub fn follow_and_create_paths(self, paths: &'a [&SynPath]) -> NodeZipper<'a> {
        let mut parent = self;
        let mut child: NodeZipper;
        let mut child_index = 0;
        let mut path_index: i16 = -1;
        for path in paths {
            path_index += 1;
            if path.in_var_range() {
                   let (opt_child, opt_node) = parent.get_child(path, true);
                let mut new_paths = path.paths_after(&paths, true);
                if opt_child.is_some() {
                    child = opt_child.expect("node");
                    if !path.is_leaf() {
                        child = child.follow_and_create_paths(new_paths);
                        parent = child.get_parent().expect("we set the parent");
                        continue;
                    }
                } else if path.is_leaf() {
                    let i = path_index as usize;
                    parent = opt_node.expect("node").create_paths(&paths[i..]);
                    return parent.ancestor(child_index);
                } else {
                    new_paths = &paths[(paths.len() - new_paths.len() - 1)..];
                    parent = opt_node.expect("node").create_paths(&new_paths);
                    continue;
                }
            } else {
                   let (opt_child, opt_node) = parent.get_child(path, false);
                if opt_child.is_none() {
                    let i = path_index as usize;
                    parent = opt_node.expect("node").create_paths(&paths[i..]);
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

    fn create_paths(self, paths: &'a [&SynPath]) -> NodeZipper<'a> {
        let mut parent = self;
        let mut child: NodeZipper;
        let mut child_index = 0;
        for path in paths {
            child = NodeZipper {
                parent: Some(Box::new(parent)),
                path_in_parent: Some(path),
                logic_node: path.in_var_range(),
                children: HashMap::new(),
                lchildren: HashMap::new(),
            };
            if path.in_var_range() && !path.is_leaf() {
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
    children: &'a HashMap<SynPath, FSNode>,
    lchildren: &'a HashMap<SynPath, FSNode>,
    response: Box<Vec<SynMatching<'a>>>,
}

impl<'a> INodeZipper<'a> {
    
    fn add_response(self, matching: SynMatching<'a>) -> INodeZipper<'a> {
        // Destructure this NodeZipper
        let INodeZipper {
            children,
            lchildren,
            mut response,
        } = self;
        response.push(matching);
        INodeZipper {
            children,
            lchildren,
            response,
        }
    }
    
    pub fn query_paths(self,
                   all_paths: Vec<&'a SynPath>,
                   matching: SynMatching<'a>,
                   ) -> INodeZipper<'a> {

        let INodeZipper {
            children: parent_children,
            lchildren: parent_lchildren,
            response: parent_response,
        } = self;
        let mut resp = parent_response;
        let split_paths = all_paths.split_first();
        if split_paths.is_some() {
            let mut subs_path: Option<SynPath> = None;
            let (path, paths) = split_paths.unwrap();
            if path.value.is_var {
                if !matching.contains_key(&path.value) {
                    let mut child: INodeZipper;
                    for (child_path, child_node) in parent_lchildren.iter() {
                        child = INodeZipper {
                            children: &child_node.children,
                            lchildren: &child_node.lchildren,
                            response: resp,
                        };
                        let mut new_matching = matching.clone();
                        new_matching.insert(&path.value, &child_path.value);
                        child = child.query_paths(paths.to_vec(), new_matching);
                        let INodeZipper {
                            response: new_response, ..
                        } = child;
                        resp = new_response;
                    }
                    return INodeZipper {
                        children: parent_children,
                        lchildren: parent_lchildren,
                        response: resp,
                    };
                } else {
                    let new_path = path.substitute(&matching);
                    subs_path = Some(new_path);
                }
            }
            let next_node: Option<&FSNode>;
            if subs_path.is_some() {
                let new_path = &subs_path.unwrap();
                if new_path.in_var_range() {
                    next_node = parent_lchildren.get(new_path);
                } else {
                    next_node = parent_children.get(new_path);
                }
            } else {
                if path.in_var_range() {
                    next_node = parent_lchildren.get(&path);
                } else {
                    next_node = parent_children.get(&path);
                }
            }
            if next_node.is_some() {
                let mut next_child = INodeZipper {
                    children: &next_node.expect("node").children,
                    lchildren: &next_node.expect("node").lchildren,
                    response: resp,
                };
                next_child = next_child.query_paths(paths.to_vec(), matching);
                let INodeZipper {
                    response: new_response, ..
                } = next_child;
                resp = new_response;
            }
        } else {
            resp.push(matching);
        }
        INodeZipper {
            children: parent_children,
            lchildren: parent_lchildren,
            response: resp,
        }
    }

    pub fn finish(self) -> Box<Vec<SynMatching<'a>>> {
        
        let INodeZipper {
            response, ..
        } = self;
        response
    }
}


impl<'a> FSNode {
    pub fn zipper(self) -> NodeZipper<'a> {
        let FSNode {
            children: child_children,
            lchildren: child_lchildren,
        } = self;
          
        NodeZipper {
            parent: None,
            path_in_parent: None,
            logic_node: false,
            children: child_children,
            lchildren: child_lchildren,
        }
    }

    pub fn qzipper(&'a self, response: Box<Vec<SynMatching<'a>>>) -> INodeZipper<'a> {
        INodeZipper {
            children: &self.children,
            lchildren: &self.lchildren,
            response,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment::SynSegment;

    #[test]
    fn factset_1() {
        let mut factset = FSNode::new();

        let segm11 = SynSegment::new("rule-name1", "(text)", false);
        let segms1 = vec![segm11];
        let path1 = SynPath::new(segms1);
        let cpath1 = path1.clone();

        let node1 = FSNode::new();
        
        factset.children.insert(cpath1, node1);

        let segm21 = SynSegment::new("rule-name1", "(text)", false);
        let segm22 = SynSegment::new("rule-name2", "(", true);
        let segms2 = vec![segm21, segm22];
        let path2 = SynPath::new(segms2);
        let cpath2 = path2.clone();

        let node2 = FSNode::new();
        let rnode1 = factset.children.get_mut(&path1).expect("path");
        rnode1.children.insert(cpath2, node2);

        let segm31 = SynSegment::new("rule-name1", "(text)", false);
        let segm32 = SynSegment::new("rule-name3", "text", true);
        let segms3 = vec![segm31, segm32];
        let path3 = SynPath::new(segms3);
        let cpath3 = path3.clone();

        let node3 = FSNode::new();
        let rnode2 = rnode1.children.get_mut(&path2).expect("path");
        rnode2.children.insert(cpath3, node3);

        let segm41 = SynSegment::new("rule-name1", "(text)", false);
        let segm42 = SynSegment::new("rule-name4", ")", true);
        let segms4 = vec![segm41, segm42];
        let path4 = SynPath::new(segms4);
        let cpath4 = path4.clone();

        let node4 = FSNode::new();
        let rnode3 = rnode2.children.get_mut(&path3).expect("path");
        rnode3.children.insert(cpath4, node4);

        let paths = vec![&path1, &path2, &path3, &path4];

        let leaf = factset.get_leaf(&paths);

        assert!(!leaf.is_none());
    }
}
