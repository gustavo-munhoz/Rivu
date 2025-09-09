use crate::classifiers::hoeffding_tree::hoeffding_tree::HoeffdingTree;
use crate::classifiers::hoeffding_tree::nodes::found_node::FoundNode;
use crate::classifiers::hoeffding_tree::nodes::split_node::SplitNode;
use crate::core::instances::Instance;
use std::sync::Arc;

pub trait Node {
    fn get_observed_class_distribution(&self) -> &Vec<f64>;
    fn is_leaf(&self) -> bool;
    fn filter_instance_to_leaf<'a>(
        &'a self,
        instance: Arc<dyn Instance>,
        parent: Option<&'a SplitNode>,
        parent_branch: usize,
    ) -> FoundNode<'a>;
    fn get_observed_class_distribution_at_leaves_reachable_through_this_node(&self) -> Vec<f64>;
    fn get_class_votes(&self, instance: &dyn Instance, hoeffding_tree: &HoeffdingTree) -> Vec<f64>;
}
