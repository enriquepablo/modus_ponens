use std::collections::HashMap;
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;

use crate::path::Path;

#[derive(Debug)]
struct Node<T> {
    data: T,
    children: HashMap<Path, Node<T>>,
}

impl<T> Node<T> {
    fn new(data: T) -> Node<T> {
        Node { data: data, children: HashMap::new() }
    }

    fn add_child(&mut self, path: Path, child: Node<T>) {
        self.children.insert(path, child);
    }
}

#[derive(Debug)]
struct NodeZipper<T> {
    node: Node<T>,
    parent: Option<Box<NodeZipper<T>>>,
    index_in_parent: usize,
}

impl<T> NodeZipper<T> {
    fn child(mut self, index: usize) -> NodeZipper<T> {
        // Remove the specified child from the node's children.
        // A NodeZipper shouldn't let its users inspect its parent,
        // since we mutate the parents
        // to move the focused nodes out of their list of children.
        // We use swap_remove() for efficiency.
        let child = self.node.children.swap_remove(index);

        // Return a new NodeZipper focused on the specified child.
        NodeZipper {
            node: child,
            parent: Some(Box::new(self)),
            index_in_parent: index,
        }
    }

    fn parent(self) -> NodeZipper<T> {
        // Destructure this NodeZipper
        let NodeZipper { node, parent, index_in_parent } = self;

        // Destructure the parent NodeZipper
        let NodeZipper {
            node: mut parent_node,
            parent: parent_parent,
            index_in_parent: parent_index_in_parent,
        } = *parent.unwrap();

        // Insert the node of this NodeZipper back in its parent.
        // Since we used swap_remove() to remove the child,
        // we need to do the opposite of that.
        parent_node.children.push(node);
        let len = parent_node.children.len();
        parent_node.children.swap(index_in_parent, len - 1);

        // Return a new NodeZipper focused on the parent.
        NodeZipper {
            node: parent_node,
            parent: parent_parent,
            index_in_parent: parent_index_in_parent,
        }
    }

    fn finish(mut self) -> Node<T> {
        while let Some(_) = self.parent {
            self = self.parent();
        }

        self.node
    }
}

impl<T> Node<T> {
    fn zipper(self) -> NodeZipper<T> {
        NodeZipper { node: self, parent: None, index_in_parent: 0 }
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
