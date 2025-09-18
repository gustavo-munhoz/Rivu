use crate::classifiers::hoeffding_tree::hoeffding_tree::HoeffdingTree;
use crate::classifiers::hoeffding_tree::nodes::FoundNode;
use crate::classifiers::hoeffding_tree::nodes::LearningNode;
use crate::classifiers::hoeffding_tree::nodes::Node;
use crate::classifiers::hoeffding_tree::nodes::SplitNode;
use crate::core::instances::Instance;
use std::sync::Arc;

pub struct InactiveLearningNode {
    observed_class_distribution: Vec<f64>,
}

impl InactiveLearningNode {
    pub fn new(observed_class_distribution: Vec<f64>) -> Self {
        Self {
            observed_class_distribution,
        }
    }
}

impl Node for InactiveLearningNode {
    fn get_observed_class_distribution(&self) -> &Vec<f64> {
        &self.observed_class_distribution
    }

    fn is_leaf(&self) -> bool {
        true
    }

    fn filter_instance_to_leaf<'a>(
        &'a self,
        instance: Arc<dyn Instance>,
        parent: Option<&'a SplitNode>,
        parent_branch: usize,
    ) -> FoundNode<'a> {
        FoundNode::new(Some(self), parent, parent_branch)
    }

    fn get_observed_class_distribution_at_leaves_reachable_through_this_node(&self) -> Vec<f64> {
        self.observed_class_distribution.clone()
    }

    fn get_class_votes(&self, instance: &dyn Instance, hoeffding_tree: &HoeffdingTree) -> Vec<f64> {
        self.observed_class_distribution.clone()
    }
}

impl LearningNode for InactiveLearningNode {
    fn learn_from_instance(&mut self, instance: Arc<dyn Instance>, hoeffding_tree: &HoeffdingTree) {
        if let Some(value) = instance.class_value() {
            let weight = instance.weight();
            self.observed_class_distribution[value as usize] += weight;
        }
    }
}
