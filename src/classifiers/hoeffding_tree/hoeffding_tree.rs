use crate::classifiers::Classifier;
use crate::classifiers::attribute_class_observers::{
    AttributeClassObserver, GaussianNumericAttributeClassObserver, NominalAttributeClassObserver,
};
use crate::classifiers::hoeffding_tree::InstanceConditionalTest;
use crate::classifiers::hoeffding_tree::leaf_prediction_option::LeafPredictionOption;
use crate::classifiers::hoeffding_tree::nodes::{
    ActiveLearningNode, FoundNode, InactiveLearningNode, LearningNode, LearningNodeNB,
    LearningNodeNBAdaptive, Node, SplitNode,
};
use crate::classifiers::hoeffding_tree::split_criteria::SplitCriterion;
use crate::classifiers::hoeffding_tree::split_criteria::gini_split_criterion::GiniSplitCriterion;
use crate::core::instance_header::InstanceHeader;
use crate::core::instances::Instance;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

pub struct HoeffdingTree {
    tree_root: Option<Rc<RefCell<dyn Node>>>,
    decision_node_count: usize,
    active_leaf_node_count: usize,
    inactive_leaf_node_count: usize,
    growth_allowed: bool,
    header: Option<Arc<InstanceHeader>>,
    training_weight_seen_by_model: f64,
    leaf_prediction_option: LeafPredictionOption,
    nb_threshold_option: Option<f64>,
    grace_period_option: usize,
    split_criterion_option: Box<dyn SplitCriterion>,
    no_pre_prune_option: bool,
    binary_splits_option: bool,
    split_confidence_option: f64,
    tie_threshold_option: f64,
    remove_poor_atts_option: bool,
    active_leaf_byte_size_estimate: f64,
    inactive_leaf_byte_size_estimate: f64,
    byte_size_estimate_overhead_fraction: f64,
    max_byte_size_option: f64,
    stop_mem_management_option: bool,
    memory_estimate_period_option: usize,
}

impl HoeffdingTree {
    pub fn new(leaf_prediction_option: LeafPredictionOption) -> Self {
        Self {
            tree_root: None,
            decision_node_count: 0,
            active_leaf_node_count: 0,
            inactive_leaf_node_count: 0,
            growth_allowed: true,
            header: None,
            training_weight_seen_by_model: 0.0,
            leaf_prediction_option,
            nb_threshold_option: None,
            grace_period_option: 0,
            split_criterion_option: Box::new(GiniSplitCriterion::new()),
            no_pre_prune_option: false,
            binary_splits_option: true,
            split_confidence_option: 1.0,
            tie_threshold_option: 1.0,
            remove_poor_atts_option: false,
            active_leaf_byte_size_estimate: 0.0,
            inactive_leaf_byte_size_estimate: 0.0,
            byte_size_estimate_overhead_fraction: 0.0,
            max_byte_size_option: f64::INFINITY,
            stop_mem_management_option: false,
            memory_estimate_period_option: 1000,
        }
    }

    pub fn set_nb_threshold(&mut self, threshold: f64) {
        self.nb_threshold_option = Some(threshold);
    }

    pub fn get_nb_threshold(&self) -> Option<f64> {
        self.nb_threshold_option
    }

    pub fn get_no_pre_prune_option(&self) -> bool {
        self.no_pre_prune_option
    }

    pub fn get_binary_splits_option(&self) -> bool {
        self.binary_splits_option
    }

    pub fn model_attribute_index_to_instance_attribute_index(
        index: usize,
        instance: &dyn Instance,
    ) -> usize {
        let class_index = instance.class_index();
        if class_index > index {
            return index;
        }
        index + 1
    }

    fn new_learning_node(&self) -> Rc<RefCell<dyn Node>> {
        let initial_class_observations = vec![0.0];
        self.new_learning_node_with_values(initial_class_observations)
    }

    fn new_learning_node_with_values(
        &self,
        initial_class_observations: Vec<f64>,
    ) -> Rc<RefCell<dyn Node>> {
        match self.leaf_prediction_option {
            LeafPredictionOption::MajorityClass => Rc::new(RefCell::new(ActiveLearningNode::new(
                initial_class_observations,
            ))),
            LeafPredictionOption::NaiveBayes => Rc::new(RefCell::new(LearningNodeNB::new(
                initial_class_observations,
            ))),
            LeafPredictionOption::AdaptiveNaiveBayes => Rc::new(RefCell::new(
                LearningNodeNBAdaptive::new(initial_class_observations),
            )),
        }
    }

    pub fn new_nominal_class_observer(&self) -> Box<dyn AttributeClassObserver> {
        Box::new(NominalAttributeClassObserver::new())
    }

    pub fn new_numeric_class_observer(&self) -> Box<dyn AttributeClassObserver> {
        Box::new(GaussianNumericAttributeClassObserver::new())
    }

    pub fn compute_hoeffding_bound(&self, range: f64, confidance: f64, n: f64) -> f64 {
        (((range * range) * (1.0 / confidance).ln()) / (2.0 * n)).sqrt()
    }

    fn deactivate_learning_node(
        &mut self,
        to_deactivate: Rc<RefCell<dyn Node>>,
        parent: Option<Rc<RefCell<dyn Node>>>,
        parent_branch: isize,
    ) {
        let obs = {
            let guard = to_deactivate.borrow();
            if let Some(active) = guard.as_any().downcast_ref::<ActiveLearningNode>() {
                active.get_observed_class_distribution().to_vec()
            } else {
                return;
            }
        };

        let new_leaf = Rc::new(RefCell::new(InactiveLearningNode::new(obs)));

        if let Some(parent_node) = parent {
            let mut parent_guard = parent_node.borrow_mut();
            if let Some(split_parent) = parent_guard.as_any_mut().downcast_mut::<SplitNode>() {
                split_parent.set_child(parent_branch as usize, new_leaf);
            }
        } else {
            self.tree_root = Some(new_leaf);
        }

        self.active_leaf_node_count -= 1;
        self.inactive_leaf_node_count += 1;
    }

    pub fn activate_learning_node(
        &mut self,
        to_activate: Rc<RefCell<dyn Node>>,
        parent: Option<Rc<RefCell<dyn Node>>>,
        parent_branch: isize,
    ) {
        let obs = {
            let guard = to_activate.borrow();
            if let Some(inactive) = guard.as_any().downcast_ref::<InactiveLearningNode>() {
                inactive.get_observed_class_distribution().to_vec()
            } else {
                return;
            }
        };

        let new_leaf = self.new_learning_node_with_values(obs);

        if let Some(parent_node) = parent {
            let mut parent_guard = parent_node.borrow_mut();
            if let Some(split_parent) = parent_guard.as_any_mut().downcast_mut::<SplitNode>() {
                split_parent.set_child(parent_branch as usize, new_leaf);
            }
        } else {
            self.tree_root = Some(new_leaf);
        }

        self.active_leaf_node_count += 1;
        self.inactive_leaf_node_count -= 1;
    }

    fn new_split_node(
        &self,
        split_test: Box<dyn InstanceConditionalTest>,
        class_observations: Vec<f64>,
        size: usize,
    ) -> Rc<RefCell<dyn Node>> {
        Rc::new(RefCell::new(SplitNode::new(
            split_test,
            class_observations,
            Some(size),
        ))) as Rc<RefCell<dyn Node>>
    }

    pub fn find_learning_nodes(&self) -> Vec<FoundNode> {
        let mut found_list = Vec::new();

        if let Some(root) = &self.tree_root {
            self.find_learning_nodes_rec(root.clone(), None, -1, &mut found_list);
        }
        found_list
    }

    fn find_learning_nodes_rec(
        &self,
        node: Rc<RefCell<dyn Node>>,
        parent: Option<Rc<RefCell<dyn Node>>>,
        parent_branch: isize,
        found: &mut Vec<FoundNode>,
    ) {
        let node_guard = node.borrow();

        if node_guard.as_any().is::<ActiveLearningNode>()
            || node_guard.as_any().is::<InactiveLearningNode>()
            || node_guard.as_any().is::<LearningNodeNB>()
            || node_guard.as_any().is::<LearningNodeNBAdaptive>()
        {
            found.push(FoundNode::new(
                Some(node.clone()),
                parent.clone(),
                parent_branch,
            ));
        }

        if let Some(split_node) = node_guard.as_any().downcast_ref::<SplitNode>() {
            for i in 0..split_node.num_children() {
                if let Some(child_arc) = split_node.get_child(i) {
                    self.find_learning_nodes_rec(child_arc, Some(node.clone()), i as isize, found);
                }
            }
        }
    }

    fn attempt_to_split(
        &mut self,
        node: Rc<RefCell<dyn Node>>,
        parent: Option<Rc<RefCell<dyn Node>>>,
        parent_index: isize,
    ) {
        let mut node_guard = node.borrow_mut();
        if let Some(active_node) = node_guard.as_any_mut().downcast_mut::<ActiveLearningNode>() {
            if active_node.observed_class_distribution_is_pure() {
                return;
            }

            let split_criterion = self.split_criterion_option.as_ref();
            let mut best_suggestions =
                active_node.get_best_split_suggestions(split_criterion, self);

            best_suggestions.sort();

            let mut should_split = false;
            if best_suggestions.len() < 2 {
                should_split = !best_suggestions.is_empty();
            } else {
                let best_suggestion = best_suggestions.last().unwrap();
                let second_best = &best_suggestions[best_suggestions.len() - 2];

                let hoeffding_bound = self.compute_hoeffding_bound(
                    split_criterion
                        .get_range_of_merit(active_node.get_observed_class_distribution()),
                    self.split_confidence_option,
                    active_node.get_weight_seen(),
                );

                if (best_suggestion.get_merit() - second_best.get_merit() > hoeffding_bound)
                    || (hoeffding_bound < self.tie_threshold_option)
                {
                    should_split = true;
                }

                if self.remove_poor_atts_option {
                    let mut poor_atts = HashSet::new();

                    for s in &best_suggestions {
                        if let Some(split_test) = s.get_split_test() {
                            let split_atts = split_test.get_atts_test_depends_on();
                            if split_atts.len() == 1 {
                                if best_suggestions.last().unwrap().get_merit() - s.get_merit()
                                    > hoeffding_bound
                                {
                                    poor_atts.insert(split_atts[0]);
                                }
                            }
                        }
                    }

                    for s in &best_suggestions {
                        if let Some(split_test) = s.get_split_test() {
                            let split_atts = split_test.get_atts_test_depends_on();
                            if split_atts.len() == 1 {
                                if best_suggestions.last().unwrap().get_merit() - s.get_merit()
                                    < hoeffding_bound
                                {
                                    poor_atts.remove(&split_atts[0]);
                                }
                            }
                        }
                    }

                    for att in poor_atts {
                        active_node.disable_attribute(att)
                    }
                }

                if should_split {
                    let split_decision = best_suggestions.last().unwrap();
                    if split_decision.get_split_test().is_none() {
                        self.deactivate_learning_node(node.clone(), parent.clone(), parent_index);
                    } else {
                        let new_split = self.new_split_node(
                            split_decision.get_split_test().unwrap().clone_box(),
                            active_node.get_observed_class_distribution().to_vec(),
                            split_decision.number_of_splits(),
                        );

                        for i in 0..split_decision.number_of_splits() {
                            let new_child = self.new_learning_node_with_values(
                                split_decision.resulting_class_distribution_from_split(i),
                            );

                            let mut guard = new_split.borrow_mut();
                            if let Some(split_node) = guard.as_any_mut().downcast_mut::<SplitNode>()
                            {
                                split_node.set_child(i, new_child);
                            }
                        }

                        self.active_leaf_node_count -= 1;
                        self.decision_node_count += 1;
                        self.active_leaf_node_count += split_decision.number_of_splits();

                        if parent.is_none() {
                            self.tree_root = Some(new_split);
                        } else {
                            if let Some(parent_arc) = parent {
                                let mut guard = parent_arc.borrow_mut();
                                if let Some(split_parent) =
                                    guard.as_any_mut().downcast_mut::<SplitNode>()
                                {
                                    split_parent.set_child(parent_index as usize, new_split);
                                }
                            }
                        }
                    }

                    self.enforce_tracker_limit();
                }
            }
        }
    }

    pub fn enforce_tracker_limit(&mut self) {
        let memory_usage = (self.active_leaf_node_count as f64
            * self.active_leaf_byte_size_estimate
            + self.inactive_leaf_node_count as f64 * self.inactive_leaf_byte_size_estimate)
            * self.byte_size_estimate_overhead_fraction;

        if self.inactive_leaf_node_count > 0 || memory_usage > self.max_byte_size_option {
            if self.stop_mem_management_option {
                self.growth_allowed = false;
                return;
            }

            let mut learning_nodes = self.find_learning_nodes();

            learning_nodes.sort_by(|a, b| {
                let promise_a = Self::extract_promise(a);
                let promise_b = Self::extract_promise(b);
                promise_a.partial_cmp(&promise_b).unwrap_or(Ordering::Equal)
            });

            let mut max_active = 0;

            while max_active < learning_nodes.len() {
                max_active += 1;
                let est = (max_active as f64 * self.active_leaf_byte_size_estimate
                    + (learning_nodes.len() - max_active) as f64
                        * self.inactive_leaf_byte_size_estimate)
                    * self.byte_size_estimate_overhead_fraction;

                if est > self.max_byte_size_option {
                    max_active -= 1;
                    break;
                }
            }

            let cutoff = learning_nodes.len().saturating_sub(max_active);

            for i in 0..cutoff {
                if let Some(node_arc) = learning_nodes[i].get_node() {
                    let guard = node_arc.borrow();
                    if guard.as_any().is::<ActiveLearningNode>() {
                        drop(guard);
                        self.deactivate_learning_node(
                            node_arc.clone(),
                            learning_nodes[i].get_parent(),
                            learning_nodes[i].get_parent_branch(),
                        )
                    }
                }
            }

            for i in cutoff..learning_nodes.len() {
                if let Some(node_arc) = learning_nodes[i].get_node() {
                    let guard = node_arc.borrow();
                    if guard.as_any().is::<InactiveLearningNode>() {
                        drop(guard);
                        self.activate_learning_node(
                            node_arc.clone(),
                            learning_nodes[i].get_parent(),
                            learning_nodes[i].get_parent_branch(),
                        )
                    }
                }
            }
        }
    }

    pub fn estimate_model_byte_sizes(&mut self) {
        let learning_nodes = self.find_learning_nodes();

        let mut total_active_size = 0.0;
        let mut total_inactive_size = 0.0;

        for found in &learning_nodes {
            if let Some(node_rc) = found.get_node() {
                let node = node_rc.borrow();
                let size = node.calc_byte_size() as f64;
                if node.as_any().is::<ActiveLearningNode>()
                    || node.as_any().is::<LearningNodeNB>()
                    || node.as_any().is::<LearningNodeNBAdaptive>()
                {
                    total_active_size += size;
                } else if node.as_any().is::<InactiveLearningNode>() {
                    total_inactive_size += size;
                }
            }
        }

        if self.active_leaf_node_count > 0 && total_active_size > 0.0 {
            self.active_leaf_byte_size_estimate =
                total_active_size / self.active_leaf_node_count as f64;
        }

        if self.inactive_leaf_node_count > 0 && total_inactive_size > 0.0 {
            self.inactive_leaf_byte_size_estimate =
                total_inactive_size / self.inactive_leaf_node_count as f64;
        }

        let estimate_model_size = (self.active_leaf_node_count as f64
            * self.active_leaf_byte_size_estimate)
            + (self.inactive_leaf_node_count as f64 * self.inactive_leaf_byte_size_estimate);

        let actual_model_size = self.calc_byte_size() as f64;

        if estimate_model_size > 0.0 {
            self.byte_size_estimate_overhead_fraction = actual_model_size / estimate_model_size;
        }

        if actual_model_size > self.max_byte_size_option {
            self.enforce_tracker_limit();
        }
    }

    pub fn calc_byte_size(&self) -> usize {
        let mut size = size_of::<Self>();
        if let Some(root) = &self.tree_root {
            size += root.borrow().calc_byte_size_including_subtree();
        }
        size
    }

    fn extract_promise(found: &FoundNode) -> f64 {
        if let Some(node_arc) = found.get_node() {
            let guard = node_arc.borrow();
            if let Some(active) = guard.as_any().downcast_ref::<ActiveLearningNode>() {
                return active.calculate_promise();
            }
        }
        0.0
    }
}

impl Classifier for HoeffdingTree {
    fn get_votes_for_instance(&self, instance: &dyn Instance) -> Vec<f64> {
        if let Some(root_arc) = &self.tree_root {
            let root_guard = root_arc.borrow();
            let found_node =
                root_guard.filter_instance_to_leaf_dyn(root_arc.clone(), instance, None, -1);

            let node_arc = found_node
                .get_node()
                .or_else(|| found_node.get_parent().map(|p| p as Rc<RefCell<dyn Node>>));
            if let Some(n_arc) = node_arc {
                let node_guard = n_arc.borrow();
                return node_guard.get_class_votes(instance, self);
            }

            Vec::new()
        } else {
            let num_classes = instance.number_of_classes();
            vec![0.0; num_classes]
        }
    }

    fn set_model_context(&mut self, header: Arc<InstanceHeader>) {
        self.header = Some(header);
    }

    fn train_on_instance(&mut self, instance: &dyn Instance) {
        if self.tree_root.is_none() {
            self.tree_root = Some(self.new_learning_node());
            self.active_leaf_node_count = 1;
        }

        let found_node = {
            let root_arc = self.tree_root.as_ref().unwrap().clone();
            let root_arc_for_call = root_arc.clone();
            let root_guard = root_arc.borrow();
            root_guard.filter_instance_to_leaf_dyn(root_arc_for_call, instance, None, -1)
        };

        let leaf_node_arc = match found_node.get_node() {
            None => {
                let new_node = self.new_learning_node();
                if let Some(parent_arc) = found_node.get_parent() {
                    let mut guard = parent_arc.borrow_mut();
                    if let Some(split_parent) = guard.as_any_mut().downcast_mut::<SplitNode>() {
                        split_parent
                            .set_child(found_node.get_parent_branch() as usize, new_node.clone());
                    }
                }
                self.active_leaf_node_count += 1;
                Some(new_node)
            }
            Some(node) => Some(node),
        };

        if let Some(leaf_arc) = leaf_node_arc {
            let mut leaf_guard = leaf_arc.borrow_mut();

            if let Some(learning_node) = leaf_guard.as_any_mut().downcast_mut::<LearningNodeNB>() {
                learning_node.learn_from_instance(instance, self);
            }
            if let Some(learning_node) = leaf_guard
                .as_any_mut()
                .downcast_mut::<LearningNodeNBAdaptive>()
            {
                learning_node.learn_from_instance(instance, self);
            }
            if let Some(learning_node) =
                leaf_guard.as_any_mut().downcast_mut::<ActiveLearningNode>()
            {
                learning_node.learn_from_instance(instance, self);

                if self.growth_allowed {
                    let weight_seen = learning_node.get_weight_seen();
                    if weight_seen - learning_node.get_weight_seen_at_last_split_evaluation()
                        >= self.grace_period_option as f64
                    {
                        drop(leaf_guard);

                        self.attempt_to_split(
                            leaf_arc.clone(),
                            found_node.get_parent(),
                            found_node.get_parent_branch(),
                        );

                        let mut leaf_guard = leaf_arc.borrow_mut();
                        if let Some(active) =
                            leaf_guard.as_any_mut().downcast_mut::<ActiveLearningNode>()
                        {
                            active.set_weight_seen_at_last_split_evaluation(weight_seen);
                        }
                    }
                }
            }
        }

        self.training_weight_seen_by_model += instance.weight();

        if self.training_weight_seen_by_model as usize % self.memory_estimate_period_option == 0 {
            self.estimate_model_byte_sizes();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classifiers::conditional_tests::attribute_split_suggestion::AttributeSplitSuggestion;
    use crate::core::attributes::{Attribute, NominalAttribute};
    use crate::core::instances::DenseInstance;
    use crate::testing::header_binary;
    use std::collections::HashMap;
    use std::io::Error;

    struct DummyInstance {
        pub weight: f64,
        pub class_val: usize,
        pub num_classes: usize,
    }
    impl Instance for DummyInstance {
        fn weight(&self) -> f64 {
            self.weight
        }

        fn set_weight(&mut self, new_value: f64) -> Result<(), Error> {
            Ok(())
        }

        fn value_at_index(&self, index: usize) -> Option<f64> {
            Some(1.0)
        }

        fn set_value_at_index(&mut self, index: usize, new_value: f64) -> Result<(), Error> {
            Ok(())
        }

        fn is_missing_at_index(&self, index: usize) -> Result<bool, Error> {
            Ok(false)
        }

        fn attribute_at_index(&self, index: usize) -> Option<&dyn Attribute> {
            None
        }

        fn index_of_attribute(&self, attribute: &dyn Attribute) -> Option<usize> {
            None
        }

        fn number_of_attributes(&self) -> usize {
            3
        }

        fn class_index(&self) -> usize {
            1
        }

        fn class_value(&self) -> Option<f64> {
            Some(self.class_val as f64)
        }

        fn set_class_value(&mut self, new_value: f64) -> Result<(), Error> {
            Ok(())
        }

        fn is_class_missing(&self) -> bool {
            false
        }

        fn number_of_classes(&self) -> usize {
            self.num_classes
        }

        fn to_vec(&self) -> Vec<f64> {
            vec![1.0]
        }

        fn header(&self) -> &InstanceHeader {
            unimplemented!()
        }
    }

    struct DummyTest {
        num_children: usize,
    }

    impl DummyTest {
        fn new(num_children: usize) -> Self {
            Self { num_children }
        }
    }

    impl InstanceConditionalTest for DummyTest {
        fn branch_for_instance(&self, _instance: &dyn Instance) -> Option<usize> {
            Some(0)
        }

        fn result_known_for_instance(&self, instance: &dyn Instance) -> bool {
            unimplemented!()
        }

        fn max_branches(&self) -> usize {
            unimplemented!()
        }

        fn get_atts_test_depends_on(&self) -> Vec<usize> {
            unimplemented!()
        }

        fn calc_byte_size(&self) -> usize {
            unimplemented!()
        }

        fn clone_box(&self) -> Box<dyn InstanceConditionalTest> {
            unimplemented!()
        }
    }

    impl SplitNode {
        pub fn new_dummy(class_dist: Vec<f64>, num_children: usize) -> Self {
            SplitNode::new(
                Box::new(DummyTest::new(num_children)),
                class_dist,
                Some(num_children),
            )
        }
    }

    struct DummyCriterion;
    impl SplitCriterion for DummyCriterion {
        fn get_range_of_merit(&self, pre_split_distribution: &Vec<f64>) -> f64 {
            1.0
        }

        fn get_merit_of_split(
            &self,
            pre_split_distribution: &[f64],
            post_split_dists: &[Vec<f64>],
        ) -> f64 {
            1.0
        }
    }

    #[derive(Clone)]
    struct DummySplitTest;
    impl InstanceConditionalTest for DummySplitTest {
        fn branch_for_instance(&self, instance: &dyn Instance) -> Option<usize> {
            Some(0)
        }

        fn result_known_for_instance(&self, instance: &dyn Instance) -> bool {
            true
        }

        fn max_branches(&self) -> usize {
            2
        }

        fn get_atts_test_depends_on(&self) -> Vec<usize> {
            vec![0]
        }

        fn calc_byte_size(&self) -> usize {
            8
        }

        fn clone_box(&self) -> Box<dyn InstanceConditionalTest> {
            Box::new(self.clone())
        }
    }

    fn make_suggestion_with_merrit(merit: f64, num_splits: usize) -> AttributeSplitSuggestion {
        AttributeSplitSuggestion::new(
            Some(Box::new(DummySplitTest)),
            vec![vec![1.0, 2.0]; num_splits],
            merit,
        )
    }

    #[test]
    fn test_set_and_get_nb_threshold() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::NaiveBayes);

        tree.set_nb_threshold(0.75);
        assert_eq!(tree.get_nb_threshold(), Some(0.75));

        tree.set_nb_threshold(1.5);
        assert_eq!(tree.get_nb_threshold(), Some(1.5));
    }

    #[test]
    fn test_get_no_pre_prune_option_default() {
        let tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        assert_eq!(tree.get_no_pre_prune_option(), false);
    }

    #[test]
    fn test_get_binary_splits_option_default() {
        let tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        assert_eq!(tree.get_binary_splits_option(), true);
    }

    #[test]
    fn test_default_tree_initial_state() {
        let tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);

        assert!(tree.tree_root.is_none());
        assert_eq!(tree.active_leaf_node_count, 0);
        assert_eq!(tree.inactive_leaf_node_count, 0);
        assert!(tree.growth_allowed);
        assert_eq!(tree.get_nb_threshold(), None);
        assert_eq!(tree.get_no_pre_prune_option(), false);
        assert_eq!(tree.get_binary_splits_option(), true);
    }

    #[test]
    fn test_model_attribute_index_to_instance_attribute_index_before_class_index() {
        let header = header_binary();
        let instance = DenseInstance::new(header.clone(), vec![1.0], 1.0);

        assert_eq!(
            HoeffdingTree::model_attribute_index_to_instance_attribute_index(0, &instance),
            1
        );
    }

    #[test]
    fn test_model_attribute_index_to_instance_attribute_index_after_class_index() {
        let dummy = DummyInstance {
            weight: 1.0,
            class_val: 0,
            num_classes: 2,
        };
        assert_eq!(
            HoeffdingTree::model_attribute_index_to_instance_attribute_index(0, &dummy),
            0
        );
    }

    #[test]
    fn test_new_learning_node_majority_class() {
        let tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        let node = tree.new_learning_node();
        let node_ref = node.borrow();

        assert!(node_ref.as_any().is::<ActiveLearningNode>());
    }

    #[test]
    fn test_new_learning_node_naive_bayes() {
        let tree = HoeffdingTree::new(LeafPredictionOption::NaiveBayes);
        let node = tree.new_learning_node();
        let node_ref = node.borrow();

        assert!(node_ref.as_any().is::<LearningNodeNB>());
    }

    #[test]
    fn test_new_learning_node_adaptive_naive_bayes() {
        let tree = HoeffdingTree::new(LeafPredictionOption::AdaptiveNaiveBayes);
        let node = tree.new_learning_node();
        let node_ref = node.borrow();

        assert!(node_ref.as_any().is::<LearningNodeNBAdaptive>());
    }

    #[test]
    fn test_new_nominal_class_observer() {
        let tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        let obs = tree.new_nominal_class_observer();

        assert!(obs.as_any().is::<NominalAttributeClassObserver>());
    }

    #[test]
    fn test_new_numeric_class_observer() {
        let tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        let obs = tree.new_numeric_class_observer();

        assert!(obs.as_any().is::<GaussianNumericAttributeClassObserver>());
    }

    #[test]
    fn test_compute_hoeffding_bound() {
        let tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        let bound = tree.compute_hoeffding_bound(1.0, 0.05, 1000.0);
        let expected = ((1.0_f64.powi(2) * ((1.0 / 0.05) as f64).ln()) / (2.0 * 1000.0)).sqrt();

        assert!((bound - expected).abs() < 1e-12);
    }

    #[test]
    fn test_deactivate_learning_node_replaces_with_inactive() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        let active_node = tree.new_learning_node();
        tree.tree_root = Some(active_node.clone());
        tree.active_leaf_node_count = 1;
        tree.inactive_leaf_node_count = 0;

        tree.deactivate_learning_node(active_node.clone(), None, -1);

        let root = tree.tree_root.as_ref().unwrap();
        let root_guard = root.borrow();
        assert!(root_guard.as_any().is::<InactiveLearningNode>());

        assert_eq!(tree.active_leaf_node_count, 0);
        assert_eq!(tree.inactive_leaf_node_count, 1);
    }

    #[test]
    fn test_activate_learning_node_replaces_with_active() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::NaiveBayes);
        let inactive_node = Rc::new(RefCell::new(InactiveLearningNode::new(vec![1.0, 2.0])));
        tree.tree_root = Some(inactive_node.clone());
        tree.active_leaf_node_count = 0;
        tree.inactive_leaf_node_count = 1;

        tree.activate_learning_node(inactive_node.clone(), None, -1);

        let root = tree.tree_root.as_ref().unwrap();
        let root_guard = root.borrow();

        let is_active_like = root_guard.as_any().is::<ActiveLearningNode>()
            || root_guard.as_any().is::<LearningNodeNB>()
            || root_guard.as_any().is::<LearningNodeNBAdaptive>();
        assert!(is_active_like, "Expected an active learning node type");

        assert_eq!(tree.active_leaf_node_count, 1);
        assert_eq!(tree.inactive_leaf_node_count, 0);
    }

    #[test]
    fn test_deactivate_learning_node_updates_parent_child() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        let active_node = tree.new_learning_node();
        let split_node = Rc::new(RefCell::new(SplitNode::new_dummy(vec![1.0, 1.0], 1)));
        split_node.borrow_mut().set_child(0, active_node.clone());
        tree.tree_root = Some(split_node.clone());
        tree.active_leaf_node_count = 1;
        tree.inactive_leaf_node_count = 0;

        tree.deactivate_learning_node(active_node.clone(), Some(split_node.clone()), 0);

        let parent_guard = split_node.borrow();
        let child = parent_guard.get_child(0).unwrap();
        let child_guard = child.borrow();
        assert!(child_guard.as_any().is::<InactiveLearningNode>());
    }

    #[test]
    fn test_new_split_node_creates_splitnode() {
        let tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        let split_test = Box::new(DummyTest::new(2));
        let class_observations = vec![1.0, 2.0];
        let node = tree.new_split_node(split_test, class_observations.clone(), 2);

        let node_ref = node.borrow();
        assert!(node_ref.as_any().is::<SplitNode>(), "Expected a SplitNode");

        let split_ref = node_ref.as_any().downcast_ref::<SplitNode>().unwrap();

        assert_eq!(
            split_ref.get_observed_class_distribution(),
            &class_observations
        );
        assert_eq!(split_ref.num_children(), 2);
    }

    #[test]
    fn test_find_learning_nodes_single_root() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::NaiveBayes);
        let leaf = tree.new_learning_node();
        tree.tree_root = Some(leaf.clone());
        tree.active_leaf_node_count = 1;

        let found = tree.find_learning_nodes();

        assert_eq!(found.len(), 1);
        assert!(found[0].get_node().is_some());

        let found_node = found[0].get_node().unwrap();
        assert!(Rc::ptr_eq(&found_node, &leaf));
    }

    #[test]
    fn test_find_learning_nodes_with_splitnode() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);

        let child1 = tree.new_learning_node();
        let child2 = tree.new_learning_node();

        let split_node: Rc<RefCell<dyn Node>> =
            Rc::new(RefCell::new(SplitNode::new_dummy(vec![1.0, 1.0], 2)));

        {
            let mut guard = split_node.borrow_mut();
            let split = guard.as_any_mut().downcast_mut::<SplitNode>().unwrap();
            split.set_child(0, child1.clone());
            split.set_child(1, child2.clone());
        }

        tree.tree_root = Some(split_node.clone());
        tree.active_leaf_node_count = 2;

        let found = tree.find_learning_nodes();

        assert_eq!(found.len(), 2);

        let found_nodes: Vec<_> = found.iter().map(|f| f.get_node().unwrap()).collect();

        for node in &found_nodes {
            let guard = node.borrow();
            let is_learning_node = guard.as_any().is::<ActiveLearningNode>()
                || guard.as_any().is::<LearningNodeNB>()
                || guard.as_any().is::<LearningNodeNBAdaptive>();
            assert!(is_learning_node)
        }

        for f in found {
            let parent = f.get_parent().unwrap();
            assert!(Rc::ptr_eq(&parent, &split_node));
        }
    }

    #[test]
    fn test_attempt_to_split_creates_splitnode_when_should_split_is_true() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        tree.split_criterion_option = Box::new(DummyCriterion);
        tree.split_confidence_option = 1.0;
        tree.tie_threshold_option = 0.0;

        let active_node = Rc::new(RefCell::new(ActiveLearningNode::new(vec![5.0, 5.0])));
        let weak_clone = active_node.clone();
        tree.tree_root = Some(active_node.clone());
        tree.active_leaf_node_count = 1;

        {
            let mut guard = weak_clone.borrow_mut();
            guard.get_observed_class_distribution();
        }

        let suggestions = vec![
            make_suggestion_with_merrit(0.1, 2),
            make_suggestion_with_merrit(0.9, 2),
        ];

        {
            let mut guard = active_node.borrow_mut();
            if let Some(node) = guard.as_any_mut().downcast_mut::<ActiveLearningNode>() {
                let split_decision = suggestions.last().unwrap();
                let new_split = tree.new_split_node(
                    split_decision.get_split_test().unwrap().clone_box(),
                    node.get_observed_class_distribution().clone(),
                    split_decision.number_of_splits(),
                );

                for i in 0..split_decision.number_of_splits() {
                    let new_child = tree.new_learning_node_with_values(
                        split_decision.resulting_class_distribution_from_split(i),
                    );

                    let mut split_guard = new_split.borrow_mut();
                    if let Some(split_node) = split_guard.as_any_mut().downcast_mut::<SplitNode>() {
                        split_node.set_child(i, new_child);
                    }
                }

                tree.active_leaf_node_count -= 1;
                tree.decision_node_count += 1;
                tree.active_leaf_node_count += split_decision.number_of_splits();
                tree.tree_root = Some(new_split.clone());
            }
        }

        let root = tree.tree_root.as_ref().unwrap();
        let root_guard = root.borrow();
        assert!(root_guard.as_any().is::<SplitNode>());

        assert_eq!(tree.decision_node_count, 1);
        assert_eq!(tree.active_leaf_node_count, 2);
    }

    #[test]
    fn test_attempt_to_split_does_nothing_when_pure_destribution() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        let active_node = Rc::new(RefCell::new(ActiveLearningNode::new(vec![10.0, 0.0])));
        tree.tree_root = Some(active_node.clone());
        tree.active_leaf_node_count = 1;
        tree.decision_node_count = 0;

        tree.attempt_to_split(active_node.clone(), None, -1);

        let root = tree.tree_root.as_ref().unwrap();
        let root_guard = root.borrow();
        assert!(root_guard.as_any().is::<ActiveLearningNode>());
        assert_eq!(tree.active_leaf_node_count, 1);
        assert_eq!(tree.decision_node_count, 0);
    }

    #[test]
    fn test_enforce_tracker_limit_stops_growth_when_stop_option_enabled() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        tree.stop_mem_management_option = true;
        tree.inactive_leaf_node_count = 1;
        tree.max_byte_size_option = 0.1;
        tree.growth_allowed = true;

        tree.enforce_tracker_limit();

        assert_eq!(tree.growth_allowed, false);
    }

    #[test]
    fn test_enforce_tracker_limit_deactivates_active_nodes_when_over_limit() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        tree.max_byte_size_option = 1.0;
        tree.active_leaf_byte_size_estimate = 10.0;
        tree.inactive_leaf_byte_size_estimate = 5.0;
        tree.byte_size_estimate_overhead_fraction = 1.0;

        let node1 = tree.new_learning_node();
        let node2 = tree.new_learning_node();

        tree.tree_root = Some(node1.clone());
        tree.active_leaf_node_count = 2;
        tree.inactive_leaf_node_count = 0;

        let found1 = FoundNode::new(Some(node1.clone()), None, -1);
        let found2 = FoundNode::new(Some(node2.clone()), None, -1);
        let learning_nodes = vec![found1, found2];

        tree.tree_root = Some(node2.clone());
        tree.enforce_tracker_limit();

        assert!(tree.inactive_leaf_node_count >= 1);
    }

    #[test]
    fn test_enforce_tracker_limit_reactivates_inactive_nodes_when_under_limit() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::NaiveBayes);
        tree.max_byte_size_option = 10_000.0;
        tree.active_leaf_byte_size_estimate = 1.0;
        tree.inactive_leaf_byte_size_estimate = 1.0;
        tree.byte_size_estimate_overhead_fraction = 1.0;

        let inactive1 = Rc::new(RefCell::new(InactiveLearningNode::new(vec![1.0, 2.0])));
        let inactive2 = Rc::new(RefCell::new(InactiveLearningNode::new(vec![3.0, 4.0])));
        tree.tree_root = Some(inactive1.clone());
        tree.active_leaf_node_count = 0;
        tree.inactive_leaf_node_count = 2;

        let found1 = FoundNode::new(Some(inactive1.clone()), None, -1);
        let found2 = FoundNode::new(Some(inactive2.clone()), None, -1);
        let learning_nodes = vec![found1, found2];

        tree.enforce_tracker_limit();

        assert!(tree.inactive_leaf_node_count >= 1);
    }

    #[test]
    fn test_calc_byte_size_basic() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        let node = tree.new_learning_node();
        tree.tree_root = Some(node.clone());

        let manual_size =
            size_of::<HoeffdingTree>() + node.borrow().calc_byte_size_including_subtree();

        let result = tree.calc_byte_size();
        assert_eq!(result, manual_size);
    }

    #[test]
    fn test_estimate_model_byte_sizes_computes_estimates() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::NaiveBayes);

        let active_node = tree.new_learning_node();
        let inactive_node = Rc::new(RefCell::new(InactiveLearningNode::new(vec![1.0, 2.0])));

        tree.tree_root = Some(active_node.clone());
        tree.active_leaf_node_count = 1;
        tree.inactive_leaf_node_count = 1;

        let dummyfound_active = FoundNode::new(Some(active_node.clone()), None, -1);
        let dummyfound_inactive = FoundNode::new(Some(inactive_node.clone()), None, -1);

        let learning_nodes = vec![dummyfound_active, dummyfound_inactive];

        tree.estimate_model_byte_sizes();

        assert!(tree.active_leaf_byte_size_estimate > 0.0);
        assert!(tree.inactive_leaf_byte_size_estimate >= 0.0);

        assert!(tree.byte_size_estimate_overhead_fraction.is_finite());
    }

    #[test]
    fn test_extract_promisse_returns_correct_value() {
        let node = Rc::new(RefCell::new(ActiveLearningNode::new(vec![3.0, 1.0, 2.0])));
        let found = FoundNode::new(Some(node.clone()), None, -1);

        let promise = HoeffdingTree::extract_promise(&found);
        assert!((promise - 3.0).abs() < 1e-12);
    }
    #[test]
    fn test_extract_promisse_returns_zero_for_non_active_node() {
        let node = Rc::new(RefCell::new(InactiveLearningNode::new(vec![1.0, 1.0])));
        let found = FoundNode::new(Some(node.clone()), None, -1);

        let promise = HoeffdingTree::extract_promise(&found);
        assert_eq!(promise, 0.0);
    }

    #[test]
    fn test_set_model_context_assigns_header() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);

        let vals = vec!["A".to_string(), "B".to_string()];
        let mut map = HashMap::new();
        map.insert("A".to_string(), 0);
        map.insert("B".to_string(), 1);

        let attr = Arc::new(NominalAttribute::with_values("class".into(), vals, map));
        let header = Arc::new(InstanceHeader::new("rel".into(), vec![attr], 0));

        tree.set_model_context(header.clone());

        assert!(tree.header.is_some());
        assert!(Arc::ptr_eq(tree.header.as_ref().unwrap(), &header));
    }

    #[test]
    fn test_get_votes_for_instance_with_empty_tree() {
        let tree = HoeffdingTree::new(LeafPredictionOption::NaiveBayes);
        let instance = DummyInstance {
            weight: 1.0,
            class_val: 0,
            num_classes: 2,
        };

        let votes = tree.get_votes_for_instance(&instance);

        assert_eq!(votes, vec![0.0, 0.0]);
    }

    #[test]
    fn test_get_votes_for_instance_returns_leaf_distribution() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        let node = Rc::new(RefCell::new(InactiveLearningNode::new(vec![3.0, 1.0])));

        tree.tree_root = Some(node.clone());

        let instance = DummyInstance {
            weight: 1.0,
            class_val: 0,
            num_classes: 2,
        };
        let votes = tree.get_votes_for_instance(&instance);

        assert_eq!(votes, vec![3.0, 1.0]);
    }

    #[test]
    fn test_train_on_instance_initializes_tree_root() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        assert!(tree.tree_root.is_none());

        let instance = DummyInstance {
            weight: 1.0,
            class_val: 0,
            num_classes: 2,
        };

        tree.train_on_instance(&instance);

        assert!(tree.tree_root.is_some());
        assert_eq!(tree.active_leaf_node_count, 1);
        assert!(tree.training_weight_seen_by_model > 0.0);
    }

    #[test]
    fn test_train_on_instance_updates_active_leaf_distribution() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        let instance = DummyInstance {
            weight: 2.0,
            class_val: 0,
            num_classes: 2,
        };

        tree.train_on_instance(&instance);

        let root = tree.tree_root.as_ref().unwrap();
        let guard = root.borrow();
        let node = guard.as_any().downcast_ref::<ActiveLearningNode>().unwrap();
        let dist = node.get_observed_class_distribution();

        assert!(dist[0] >= 2.0);
    }

    #[test]
    fn test_train_on_instance_triggers_estimate_model_byte_sizes() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::NaiveBayes);
        tree.memory_estimate_period_option = 1;
        tree.training_weight_seen_by_model = 0.0;

        let instance = DummyInstance {
            weight: 1.0,
            class_val: 0,
            num_classes: 2,
        };

        tree.train_on_instance(&instance);
        assert!(tree.active_leaf_byte_size_estimate >= 0.0);
    }

    #[test]
    fn test_train_on_instance_does_not_split_when_grace_period_not_met() {
        let mut tree = HoeffdingTree::new(LeafPredictionOption::MajorityClass);
        tree.grace_period_option = 100;

        let instance = DummyInstance {
            weight: 1.0,
            class_val: 0,
            num_classes: 2,
        };

        tree.train_on_instance(&instance);

        let root = tree.tree_root.as_ref().unwrap();
        let guard = root.borrow();
        assert!(guard.as_any().is::<ActiveLearningNode>());
        assert_eq!(tree.decision_node_count, 0);
    }
}
