use crate::classifiers::hoeffding_tree::hoeffding_tree::HoeffdingTree;
use crate::core::instances::Instance;
use std::sync::Arc;

pub trait LearningNode {
    fn learn_from_instance(&mut self, instance: Arc<dyn Instance>, hoeffding_tree: &HoeffdingTree);
}
