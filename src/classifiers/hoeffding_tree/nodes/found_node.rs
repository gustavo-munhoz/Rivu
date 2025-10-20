use crate::classifiers::hoeffding_tree::nodes::node::Node;
use std::cell::RefCell;
use std::rc::Rc;

pub struct FoundNode {
    node: Option<Rc<RefCell<dyn Node>>>,
    pub parent: Option<Rc<RefCell<dyn Node>>>,
    parent_branch: isize,
}

impl FoundNode {
    pub fn new(
        node: Option<Rc<RefCell<dyn Node>>>,
        parent: Option<Rc<RefCell<dyn Node>>>,
        parent_branch: isize,
    ) -> Self {
        Self {
            node,
            parent,
            parent_branch,
        }
    }

    pub fn get_node(&self) -> Option<Rc<RefCell<dyn Node>>> {
        self.node.clone()
    }

    pub fn get_parent(&self) -> Option<Rc<RefCell<dyn Node>>> {
        self.parent.clone()
    }

    pub fn get_parent_branch(&self) -> isize {
        self.parent_branch
    }
}
