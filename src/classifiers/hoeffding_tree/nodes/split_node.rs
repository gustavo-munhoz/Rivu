use crate::classifiers::hoeffding_tree::hoeffding_tree::HoeffdingTree;
use crate::classifiers::hoeffding_tree::instance_conditional_test::InstanceConditionalTest;
use crate::classifiers::hoeffding_tree::nodes::found_node::FoundNode;
use crate::classifiers::hoeffding_tree::nodes::node::Node;
use crate::core::instances::Instance;
use std::sync::Arc;

pub struct SplitNode {
    observed_class_distribution: Vec<f64>,
    split_test: Box<dyn InstanceConditionalTest>,
    children: Vec<Option<Arc<dyn Node>>>,
}

impl SplitNode {
    pub fn new(
        split_test: Box<dyn InstanceConditionalTest>,
        observed_class_distribution: Vec<f64>,
        initial_children_len: Option<usize>,
    ) -> Self {
        let children = match initial_children_len {
            Some(len) => (0..len).map(|_| None).collect(),
            None => Vec::new(),
        };
        Self {
            observed_class_distribution,
            split_test,
            children,
        }
    }

    pub fn set_child(&mut self, index: usize, child: Arc<dyn Node>) {
        if index >= self.children.len() {
            self.children.resize_with(index + 1, || None);
        }
        self.children[index] = Some(child);
    }

    pub fn get_child(&self, index: usize) -> Option<&Arc<dyn Node>> {
        self.children.get(index).and_then(|opt| opt.as_ref())
    }

    fn add_in_place(dst: &mut [f64], src: &[f64]) {
        debug_assert_eq!(dst.len(), src.len(), "class_distribution length mismatch");
        for (d, s) in dst.iter_mut().zip(src.iter()) {
            *d += *s;
        }
    }

    fn instance_child_index(&self, instance: Arc<dyn Instance>) -> Option<usize> {
        self.split_test.branch_for_instance(instance)
    }
}

impl Node for SplitNode {
    fn get_observed_class_distribution(&self) -> &Vec<f64> {
        &self.observed_class_distribution
    }

    fn is_leaf(&self) -> bool {
        false
    }

    fn filter_instance_to_leaf<'a>(
        &'a self,
        instance: Arc<dyn Instance>,
        parent: Option<&'a SplitNode>,
        parent_branch: usize,
    ) -> FoundNode<'a> {
        if let Some(child_index) = self.instance_child_index(instance.clone()) {
            if let Some(child) = self.get_child(child_index) {
                return child.filter_instance_to_leaf(instance, Some(self), child_index);
            }
            return FoundNode::new(None, Some(self), child_index);
        }
        FoundNode::new(Some(self), parent, parent_branch)
    }

    fn get_observed_class_distribution_at_leaves_reachable_through_this_node(&self) -> Vec<f64> {
        let mut sum_observed_class_distribution_at_leaves =
            vec![0.0; self.observed_class_distribution.len()];
        for child_opt in &self.children {
            if let Some(child) = child_opt.as_ref() {
                let child_dist =
                    child.get_observed_class_distribution_at_leaves_reachable_through_this_node();
                Self::add_in_place(&mut sum_observed_class_distribution_at_leaves, &child_dist)
            }
        }
        sum_observed_class_distribution_at_leaves
    }

    fn get_class_votes(&self, instance: &dyn Instance, hoeffding_tree: &HoeffdingTree) -> Vec<f64> {
        self.observed_class_distribution.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classifiers::hoeffding_tree::instance_conditional_test::InstanceConditionalTest;
    use crate::classifiers::hoeffding_tree::nodes::InactiveLearningNode;
    use crate::core::attributes::NominalAttribute;
    use crate::core::instance_header::InstanceHeader;
    use crate::core::instances::dense_instance::DenseInstance;
    use std::sync::Arc;

    struct DummyTest {
        branch: Option<usize>,
    }

    impl InstanceConditionalTest for DummyTest {
        fn branch_for_instance(&self, _instance: Arc<dyn Instance>) -> Option<usize> {
            self.branch
        }

        fn result_known_for_instance(&self, _instance: Arc<dyn Instance>) -> bool {
            self.branch.is_some()
        }

        fn max_branches(&self) -> usize {
            2
        }

        fn get_atts_test_depends_on(&self) -> Vec<usize> {
            vec![0]
        }
    }

    fn make_header() -> Arc<InstanceHeader> {
        use crate::core::attributes::AttributeRef;
        use std::collections::HashMap;

        let mut label_to_index1 = HashMap::new();
        label_to_index1.insert("a".to_string(), 0);
        label_to_index1.insert("b".to_string(), 1);

        let att1: AttributeRef = Arc::new(NominalAttribute::with_values(
            "att1".to_string(),
            vec!["a".to_string(), "b".to_string()],
            label_to_index1,
        ));

        let mut label_to_index_class = HashMap::new();
        label_to_index_class.insert("c0".to_string(), 0);
        label_to_index_class.insert("c1".to_string(), 1);

        let class_att: AttributeRef = Arc::new(NominalAttribute::with_values(
            "class".to_string(),
            vec!["c0".to_string(), "c1".to_string()],
            label_to_index_class,
        ));

        let attributes: Vec<AttributeRef> = vec![att1, class_att];

        Arc::new(InstanceHeader::new("relation".to_string(), attributes, 1))
    }

    fn make_instance(class_value: f64) -> Arc<dyn Instance> {
        let header = make_header();
        let mut values = vec![0.0; header.number_of_attributes()];
        values[header.class_index()] = class_value;
        Arc::new(DenseInstance::new(header, values, 1.0))
    }

    #[test]
    fn test_new_split_node_initializes_children() {
        let test = Box::new(DummyTest { branch: None });
        let node = SplitNode::new(test, vec![1.0, 2.0], Some(3));
        assert_eq!(node.children.len(), 3);
        assert!(node.children.iter().all(|c| c.is_none()));
    }

    #[test]
    fn test_set_and_get_child_with_real_node() {
        let test = Box::new(DummyTest { branch: Some(0) });
        let mut node = SplitNode::new(test, vec![1.0, 2.0], Some(1));

        let leaf = Arc::new(InactiveLearningNode::new(vec![5.0, 5.0]));
        node.set_child(0, leaf.clone());

        let retrieved = node.get_child(0).unwrap();
        assert_eq!(retrieved.get_observed_class_distribution(), &vec![5.0, 5.0]);
    }

    #[test]
    fn test_distribution_sum_with_real_nodes() {
        let test = Box::new(DummyTest { branch: None });
        let mut node = SplitNode::new(test, vec![1.0, 2.0], Some(2));

        let leaf1 = Arc::new(InactiveLearningNode::new(vec![2.0, 3.0]));
        let leaf2 = Arc::new(InactiveLearningNode::new(vec![4.0, 1.0]));
        node.set_child(0, leaf1);
        node.set_child(1, leaf2);

        let summed = node.get_observed_class_distribution_at_leaves_reachable_through_this_node();
        assert_eq!(summed, vec![6.0, 4.0]); // (2+4 , 3+1)
    }

    #[test]
    fn test_filter_instance_to_leaf_routes_to_real_node() {
        let test = Box::new(DummyTest { branch: Some(0) });
        let mut node = SplitNode::new(test, vec![1.0, 2.0], Some(1));

        let leaf = Arc::new(InactiveLearningNode::new(vec![3.0, 7.0]));
        node.set_child(0, leaf.clone());

        let inst = make_instance(1.0);
        let found = node.filter_instance_to_leaf(inst, None, 0);

        assert!(found.get_node().is_some());
        assert_eq!(
            found.get_node().unwrap().get_observed_class_distribution(),
            &vec![3.0, 7.0]
        );
    }
}
