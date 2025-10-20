use crate::classifiers::hoeffding_tree::hoeffding_tree::HoeffdingTree;
use crate::classifiers::hoeffding_tree::nodes::FoundNode;
use crate::classifiers::hoeffding_tree::nodes::LearningNode;
use crate::classifiers::hoeffding_tree::nodes::Node;
use crate::core::instances::Instance;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

pub struct InactiveLearningNode {
    observed_class_distribution: Vec<f64>,
}

impl InactiveLearningNode {
    pub fn new(observed_class_distribution: Vec<f64>) -> Self {
        Self {
            observed_class_distribution,
        }
    }

    pub fn num_non_zero_entries(vec: &Vec<f64>) -> usize {
        vec.iter().filter(|&&x| x != 0.0).count()
    }
}

impl Node for InactiveLearningNode {
    fn get_observed_class_distribution(&self) -> &Vec<f64> {
        &self.observed_class_distribution
    }

    fn is_leaf(&self) -> bool {
        true
    }

    fn filter_instance_to_leaf(
        self_arc: Rc<RefCell<Self>>,
        instance: &dyn Instance,
        parent: Option<Rc<RefCell<dyn Node>>>,
        parent_branch: isize,
    ) -> FoundNode {
        FoundNode::new(Some(self_arc), parent, parent_branch)
    }

    fn filter_instance_to_leaf_dyn(
        &self,
        self_arc_dyn: Rc<RefCell<dyn Node>>,
        _instance: &dyn Instance,
        parent: Option<Rc<RefCell<dyn Node>>>,
        parent_branch: isize,
    ) -> FoundNode {
        FoundNode::new(Some(self_arc_dyn), parent, parent_branch)
    }

    fn get_observed_class_distribution_at_leaves_reachable_through_this_node(&self) -> Vec<f64> {
        self.observed_class_distribution.clone()
    }

    fn get_class_votes(&self, instance: &dyn Instance, hoeffding_tree: &HoeffdingTree) -> Vec<f64> {
        self.observed_class_distribution.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn observed_class_distribution_is_pure(&self) -> bool {
        Self::num_non_zero_entries(&self.observed_class_distribution) < 2
    }
    fn calc_byte_size(&self) -> usize {
        let mut total = size_of::<Self>();

        total += size_of::<Vec<f64>>();
        total += self.observed_class_distribution.len() * size_of::<f64>();

        total
    }

    fn calc_byte_size_including_subtree(&self) -> usize {
        self.calc_byte_size()
    }
}

impl LearningNode for InactiveLearningNode {
    fn learn_from_instance(&mut self, instance: &dyn Instance, hoeffding_tree: &HoeffdingTree) {
        if let Some(value) = instance.class_value() {
            let weight = instance.weight();
            self.observed_class_distribution[value as usize] += weight;
        }
    }
}
