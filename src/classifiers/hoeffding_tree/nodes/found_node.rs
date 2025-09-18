use crate::classifiers::hoeffding_tree::nodes::node::Node;
use crate::classifiers::hoeffding_tree::nodes::split_node::SplitNode;
use std::sync::Arc;

pub struct FoundNode<'a> {
    node: Option<&'a dyn Node>,
    parent: Option<&'a SplitNode>,
    parent_branch: usize,
}

impl<'a> FoundNode<'a> {
    pub fn new(
        node: Option<&'a dyn Node>,
        parent: Option<&'a SplitNode>,
        parent_branch: usize,
    ) -> Self {
        Self {
            node,
            parent,
            parent_branch,
        }
    }

    pub fn get_node(&self) -> &Option<&'a dyn Node> {
        &self.node
    }

    pub fn get_parent(&self) -> &Option<&'a SplitNode> {
        &self.parent
    }

    pub fn get_parent_branch(&self) -> usize {
        self.parent_branch
    }
}
