use crate::classifiers::hoeffding_tree::nodes::node::Node;
use crate::classifiers::hoeffding_tree::nodes::split_node::SplitNode;
use std::sync::{Arc, RwLock};

pub struct FoundNode {
    node: Option<Arc<RwLock<dyn Node>>>,
    pub parent: Option<Arc<RwLock<dyn Node>>>,
    parent_branch: isize,
}

impl FoundNode {
    pub fn new(
        node: Option<Arc<RwLock<dyn Node>>>,
        parent: Option<Arc<RwLock<dyn Node>>>,
        parent_branch: isize,
    ) -> Self {
        Self {
            node,
            parent,
            parent_branch,
        }
    }

    pub fn get_node(&self) -> Option<Arc<RwLock<dyn Node>>> {
        self.node.clone()
    }

    pub fn get_parent(&self) -> Option<Arc<RwLock<dyn Node>>> {
        self.parent.clone()
    }

    pub fn get_parent_branch(&self) -> isize {
        self.parent_branch
    }
}
