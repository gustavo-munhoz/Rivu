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
use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

pub struct HoeffdingTree {
    tree_root: Option<Arc<RwLock<dyn Node>>>,
    decision_node_count: usize,
    active_leaf_node_count: usize,
    inactive_leaf_node_count: usize,
    growth_allowed: bool,
    header: Option<Arc<InstanceHeader>>,
    leaf_prediction_option: LeafPredictionOption,
    nb_threshold_option: Option<f64>,
    grace_period_option: usize,
    split_criterion_option: Box<dyn SplitCriterion + Send + Sync>,
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

    fn new_learning_node(&self) -> Arc<RwLock<dyn Node>> {
        let initial_class_observations = vec![0.0];
        self.new_learning_node_with_values(initial_class_observations)
    }
    fn new_learning_node_with_values(
        &self,
        initial_class_observations: Vec<f64>,
    ) -> Arc<RwLock<dyn Node>> {
        match self.leaf_prediction_option {
            LeafPredictionOption::MajorityClass => Arc::new(RwLock::new(ActiveLearningNode::new(
                initial_class_observations,
            ))),
            LeafPredictionOption::NaiveBayes => {
                Arc::new(RwLock::new(LearningNodeNB::new(initial_class_observations)))
            }
            LeafPredictionOption::AdaptiveNaiveBayes => Arc::new(RwLock::new(
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
        to_deactivate: Arc<RwLock<dyn Node>>,
        parent: Option<Arc<RwLock<dyn Node>>>,
        parent_branch: isize,
    ) {
        let obs = {
            let guard = to_deactivate.read().unwrap();
            if let Some(active) = guard.as_any().downcast_ref::<ActiveLearningNode>() {
                active.get_observed_class_distribution().to_vec()
            } else {
                return;
            }
        };

        let new_leaf = Arc::new(RwLock::new(InactiveLearningNode::new(obs)));

        if let Some(parent_node) = parent {
            let mut parent_guard = parent_node.write().unwrap();
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
        to_activate: Arc<RwLock<dyn Node>>,
        parent: Option<Arc<RwLock<dyn Node>>>,
        parent_branch: isize,
    ) {
        let obs = {
            let guard = to_activate.read().unwrap();
            if let Some(inactive) = guard.as_any().downcast_ref::<InactiveLearningNode>() {
                inactive.get_observed_class_distribution().to_vec()
            } else {
                return;
            }
        };

        let new_leaf = self.new_learning_node_with_values(obs);

        if let Some(parent_node) = parent {
            let mut parent_guard = parent_node.write().unwrap();
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
        split_test: Box<dyn InstanceConditionalTest + Send + Sync>,
        class_observations: Vec<f64>,
        size: usize,
    ) -> Arc<RwLock<dyn Node>> {
        Arc::new(RwLock::new(SplitNode::new(
            split_test,
            class_observations,
            Some(size),
        ))) as Arc<RwLock<dyn Node>>
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
        node: Arc<RwLock<dyn Node>>,
        parent: Option<Arc<RwLock<dyn Node>>>,
        parent_branch: isize,
        found: &mut Vec<FoundNode>,
    ) {
        let node_guard = node.read().unwrap();

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
        node: Arc<RwLock<dyn Node>>,
        parent: Option<Arc<RwLock<dyn Node>>>,
        parent_index: isize,
    ) {
        let mut node_guard = node.write().unwrap();
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

                            let mut guard = new_split.write().unwrap();
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
                                let mut guard = parent_arc.write().unwrap();
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
                    let guard = node_arc.read().unwrap();
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
                    let guard = node_arc.read().unwrap();
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

    fn extract_promise(found: &FoundNode) -> f64 {
        if let Some(node_arc) = found.get_node() {
            let guard = node_arc.read().unwrap();
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
            let root_guard = root_arc.read().unwrap();
            let found_node =
                root_guard.filter_instance_to_leaf_dyn(root_arc.clone(), instance, None, -1);

            let node_arc = found_node
                .get_node()
                .or_else(|| found_node.get_parent().map(|p| p as Arc<RwLock<dyn Node>>));
            if let Some(n_arc) = node_arc {
                let node_guard = n_arc.read().unwrap();
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
            let root_guard = root_arc.read().unwrap();
            root_guard.filter_instance_to_leaf_dyn(root_arc_for_call, instance, None, -1)
        };

        let leaf_node_arc = match found_node.get_node() {
            None => {
                let new_node = self.new_learning_node();
                if let Some(parent_arc) = found_node.get_parent() {
                    let mut guard = parent_arc.write().unwrap();
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
            let mut leaf_guard = leaf_arc.write().unwrap();

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
                        self.attempt_to_split(
                            leaf_arc.clone(),
                            found_node.get_parent(),
                            found_node.get_parent_branch(),
                        );
                        learning_node.set_weight_seen_at_last_split_evaluation(weight_seen);
                    }
                }
            }
        }
    }
}
