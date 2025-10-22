use crate::classifiers::attribute_class_observers::AttributeClassObserver;
use crate::classifiers::attribute_class_observers::null_attribute_class_observer::NullAttributeClassObserver;
use crate::classifiers::conditional_tests::attribute_split_suggestion::AttributeSplitSuggestion;
use crate::classifiers::hoeffding_tree::hoeffding_tree::HoeffdingTree;
use crate::classifiers::hoeffding_tree::nodes::LearningNode;
use crate::classifiers::hoeffding_tree::nodes::found_node::FoundNode;
use crate::classifiers::hoeffding_tree::nodes::node::Node;
use crate::classifiers::hoeffding_tree::split_criteria::SplitCriterion;
use crate::core::attributes::NominalAttribute;
use crate::core::instances::Instance;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ActiveLearningNode {
    observed_class_distribution: Vec<f64>,
    weight_seen_at_last_split_evaluation: f64,
    attribute_observers: Vec<Option<Box<dyn AttributeClassObserver>>>,
    is_initialized: bool,
}

impl ActiveLearningNode {
    pub fn new(observed_class_distribution: Vec<f64>) -> Self {
        let weight_seen = observed_class_distribution.iter().sum();
        Self {
            observed_class_distribution,
            weight_seen_at_last_split_evaluation: weight_seen,
            attribute_observers: Vec::new(),
            is_initialized: false,
        }
    }

    pub fn get_weight_seen(&self) -> f64 {
        self.observed_class_distribution.iter().sum()
    }

    pub fn get_weight_seen_at_last_split_evaluation(&self) -> f64 {
        self.weight_seen_at_last_split_evaluation
    }

    pub fn set_weight_seen_at_last_split_evaluation(&mut self, weight: f64) {
        self.weight_seen_at_last_split_evaluation = weight;
    }

    pub fn num_non_zero_entries(vec: &Vec<f64>) -> usize {
        vec.iter().filter(|&&x| x != 0.0).count()
    }

    pub fn get_best_split_suggestions(
        &self,
        criterion: &dyn SplitCriterion,
        ht: &HoeffdingTree,
    ) -> Vec<AttributeSplitSuggestion> {
        let mut best_suggestions: Vec<AttributeSplitSuggestion> = Vec::new();
        let pre_split_distribution = self.observed_class_distribution.clone();
        if !ht.get_no_pre_prune_option() {
            let merit = criterion
                .get_merit_of_split(&pre_split_distribution, &[pre_split_distribution.clone()]);
            best_suggestions.push(AttributeSplitSuggestion::new(
                None,
                vec![pre_split_distribution.clone()],
                merit,
            ));
        }

        for (i, obs_opt) in self.attribute_observers.iter().enumerate() {
            if let Some(obs) = obs_opt {
                if let Some(best_suggestion) = obs.get_best_evaluated_split_suggestion(
                    criterion,
                    &pre_split_distribution,
                    i,
                    ht.get_binary_splits_option(),
                ) {
                    best_suggestions.push(best_suggestion)
                }
            }
        }
        best_suggestions
    }

    pub fn disable_attribute(&mut self, attribute_index: usize) {
        self.attribute_observers[attribute_index] =
            Some(Box::new(NullAttributeClassObserver::new()));
    }

    pub fn calculate_promise(&self) -> f64 {
        let total_seen: f64 = self.observed_class_distribution.iter().sum();

        if total_seen > 0.0 {
            let max_value = self
                .observed_class_distribution
                .iter()
                .cloned()
                .fold(f64::MIN, f64::max);

            total_seen - max_value
        } else {
            0.0
        }
    }
}

impl Node for ActiveLearningNode {
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

        total += size_of::<Vec<Option<Box<dyn AttributeClassObserver>>>>();
        for obs_opt in &self.attribute_observers {
            total += size_of::<Option<Box<dyn AttributeClassObserver>>>();
            if let Some(obs) = obs_opt {
                total += size_of::<Box<dyn AttributeClassObserver>>();
                total += obs.estimate_size_bytes();
            }
        }

        total += size_of::<f64>();
        total += size_of::<bool>();

        total
    }

    fn calc_byte_size_including_subtree(&self) -> usize {
        self.calc_byte_size()
    }
}

impl LearningNode for ActiveLearningNode {
    fn learn_from_instance(&mut self, instance: &dyn Instance, hoeffding_tree: &HoeffdingTree) {
        if !self.is_initialized {
            self.attribute_observers = (0..instance.number_of_attributes()).map(|_| None).collect();
            self.is_initialized = true;
        }

        if let Some(class_index) = instance.class_value() {
            let weight = instance.weight();
            let idx = class_index as usize;
            if idx >= self.observed_class_distribution.len() {
                self.observed_class_distribution.resize(idx + 1, 0.0);
            }
            self.observed_class_distribution[idx] += weight;
        }

        for i in 0..instance.number_of_attributes() - 1 {
            let instance_attribute_index =
                HoeffdingTree::model_attribute_index_to_instance_attribute_index(i, instance);

            if self.attribute_observers[i].is_none() {
                if let Some(attribute) = instance.attribute_at_index(instance_attribute_index) {
                    let observer: Box<dyn AttributeClassObserver> =
                        if attribute.as_any().is::<NominalAttribute>() {
                            hoeffding_tree.new_nominal_class_observer()
                        } else {
                            hoeffding_tree.new_numeric_class_observer()
                        };
                    self.attribute_observers[i] = Some(observer);
                }
            }

            if let Some(observer) = self.attribute_observers[i].as_mut() {
                if let (Some(class_index), Some(value)) = (
                    instance.class_value(),
                    instance.value_at_index(instance_attribute_index),
                ) {
                    observer.observe_attribute_class(
                        value,
                        class_index as usize,
                        instance.weight(),
                    );
                }
            }
        }
    }
}
