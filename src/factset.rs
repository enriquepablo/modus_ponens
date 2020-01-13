use std::collections::HashMap;
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;

use crate::path::Path;

pub struct FSNode<'a> {
    path: Option<&'a Path<'a>>,
    logic_children: HashMap<Path<'a>, Rc<RefCell<FSNode<'a>>>>,
    nonlogic_children: HashMap<Path<'a>, Rc<RefCell<FSNode<'a>>>>,
}

impl<'a> fmt::Debug for FSNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path;
        if self.path.is_none() {
            path = "ROOT".to_string();
        } else {
            path = format!("{}", self.path.expect("no path"));
        }
        write!(f, "Node {{ path: {}, logic: {}, nonlogic: {} }}",
               path, self.logic_children.len(), self.nonlogic_children.len())
    }
}

impl<'a> PartialEq for FSNode<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl<'a> Eq for FSNode<'a> {}

impl<'a> FSNode<'a> {

    fn get_fact_leaf(&'a self, paths: &'a Vec<&Path>) -> Option<&'a FSNode<'a>> {
        let mut parent: Option<&'a FSNode> = None;
        let mut preparent;
        let mut reparent;
        let plen = paths.len();
        if plen > 0 {
            parent = Some(self);
            for i in 0..paths.len() {
                if parent.is_none() {
                    break;
                } else {
                    let path = paths[i];
                    preparent = parent.expect("node").nonlogic_children.get(path);
                    if preparent.is_none() {
                        preparent = parent.expect("node").logic_children.get(path);
                    }
                    if !preparent.is_none() {
                        reparent = Rc::clone(preparent.expect("node"));
                        parent = Some(&reparent.borrow_mut());
                    } else {
                        parent = None;
                    }
                }
            }
        }
        parent
    }

    fn get_fact_leaf(&'a self, paths: &'a Vec<&Path>) -> Option<&'a FSNode<'a>> {
        let plen = paths.len();
        if plen > 0 {
            let path = paths[0];
            let mut child = self.nonlogic_children.get(path);
            if child.is_none() {
                child = self.logic_children.get(path);
            }
            if !child.is_none() {
                let child_node = &child.expect("rc").borrow_mut();
                child_node.get_fact_leaf(&paths[1..].to_vec())
            } else {
                None
            }
        } else {
            Some(self)
        }
    }

    fn follow_or_create_paths(&mut self, paths: &'a [&Path]) {
        if paths.len() > 0 {
            let mut node = self;
            for i in 0..paths.len() {
                let path = paths[i];
                let next;
                if path.value.in_var_range() {
                    next = node.logic_children.get_mut(path);
                    if !path.is_leaf() {
                        let new_paths = path.paths_after(paths, true);
                        if next.is_none() {
                            let mut next_paths = new_paths.clone();
                            next_paths.insert(0, path);
                            node.create_paths(&next_paths);
                        } else {
                            node.follow_or_create_paths(&new_paths);
                        }
                        continue;
                    }
                } else {
                    next = node.nonlogic_children.get_mut(path);
                }
                if next.is_none() {
                    node.create_paths(&paths[i..]);
                    break;
                }
                node = &mut next.expect("node").borrow_mut();
            }
        }
    }

    fn create_paths(&mut self, paths: &'a [&Path]) {
        let mut parent = self;
        for path in paths {
            let new_node = Rc::new(RefCell::new(FSNode {
                path: Some(&path),
                logic_children: HashMap::new(),
                nonlogic_children: HashMap::new(),
            }));
            if path.value.in_var_range() {
                parent.logic_children.insert(**path, new_node);
                if !path.is_leaf() {
                    let new_paths = path.paths_after(paths, true);
                    let mut young_parent = new_node.borrow_mut();
                    young_parent.create_paths(&new_paths);
                    continue;
                }
            } else {
                parent.nonlogic_children.insert(**path, new_node);
            }
        parent = &mut new_node.borrow_mut();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment::Segment;

    #[test]
    fn factset_1() {
        let mut factset = FSNode {
            path: None,
            logic_children: HashMap::new(),
            nonlogic_children: HashMap::new(),
        };

        let segm11 = Segment::make_segment("rule-name1", "(text)", 0, false);
        let segms1 = vec![segm11];
        let path1 = Path::make_path(&segms1);
        let cpath1 = path1.clone();

        let node1 = Rc::new(RefCell::new(FSNode {
            path: Some(&path1),
            logic_children: HashMap::new(),
            nonlogic_children: HashMap::new(),
        }));
        {
            factset.nonlogic_children.insert(cpath1, node1);
        }

        let segm21 = Segment::make_segment("rule-name1", "(text)", 0, false);
        let segm22 = Segment::make_segment("rule-name2", "(", 0, true);
        let segms2 = vec![segm21, segm22];
        let path2 = Path::make_path(&segms2);
        let cpath2 = path2.clone();

        let node2 = Rc::new(RefCell::new(FSNode {
            path: Some(&path2),
            logic_children: HashMap::new(),
            nonlogic_children: HashMap::new(),
        }));
        let rnode1 = factset.nonlogic_children.get_mut(&path1).expect("path");
        rnode1.borrow_mut().nonlogic_children.insert(cpath2, node2);

        let segm31 = Segment::make_segment("rule-name1", "(text)", 0, false);
        let segm32 = Segment::make_segment("rule-name3", "text", 0, true);
        let segms3 = vec![segm31, segm32];
        let path3 = Path::make_path(&segms3);
        let cpath3 = path3.clone();

        let node3 = Rc::new(RefCell::new(FSNode {
            path: Some(&path3),
            logic_children: HashMap::new(),
            nonlogic_children: HashMap::new(),
        }));
        let rnode2 = rnode1.borrow_mut().nonlogic_children.get_mut(&path2).expect("path");
        rnode2.borrow_mut().nonlogic_children.insert(cpath3, node3);

        let segm41 = Segment::make_segment("rule-name1", "(text)", 0, false);
        let segm42 = Segment::make_segment("rule-name4", ")", 0, true);
        let segms4 = vec![segm41, segm42];
        let path4 = Path::make_path(&segms4);
        let cpath4 = path4.clone();

        let node4 = Rc::new(RefCell::new(FSNode {
            path: Some(&path4),
            logic_children: HashMap::new(),
            nonlogic_children: HashMap::new(),
        }));
        let rnode3 = rnode2.borrow_mut().nonlogic_children.get_mut(&path3).expect("path");
        rnode3.borrow_mut().nonlogic_children.insert(cpath4, node4);

        let paths = vec![&path1, &path2, &path3, &path4];

        let leaf = factset.get_fact_leaf(&paths);

        assert_eq!(leaf.expect("node").path.expect("path"), &path4);
    }
}
