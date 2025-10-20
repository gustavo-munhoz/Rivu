use crate::classifiers::hoeffding_tree::hoeffding_tree::HoeffdingTree;
use crate::classifiers::hoeffding_tree::nodes::Node;
use crate::core::instances::Instance;

pub trait LearningNode: Node {
    fn learn_from_instance(&mut self, instance: &dyn Instance, hoeffding_tree: &HoeffdingTree);
}
