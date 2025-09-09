use crate::classifiers::attribute_class_observers::AttributeClassObserver;
use crate::classifiers::bayes::naive_bayes::NaiveBayes;
use crate::classifiers::hoeffding_tree::hoeffding_tree::HoeffdingTree;
use crate::classifiers::hoeffding_tree::nodes::FoundNode;
use crate::classifiers::hoeffding_tree::nodes::LearningNode;
use crate::classifiers::hoeffding_tree::nodes::Node;
use crate::classifiers::hoeffding_tree::nodes::SplitNode;
use crate::core::attributes::NominalAttribute;
use crate::core::instances::Instance;
use std::sync::Arc;

pub struct LearningNodeNB {
    observed_class_distribution: Vec<f64>,
    weight_seen_at_last_split_evaluation: f64,
    attribute_observers: Vec<Option<Box<dyn AttributeClassObserver>>>,
    is_initialized: bool,
}

impl LearningNodeNB {
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
}

impl Node for LearningNodeNB {
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
        if let Some(threshold) = hoeffding_tree.get_nb_threshold() {
            if (self.get_weight_seen() >= threshold) {
                return NaiveBayes::do_naive_bayes_prediction(
                    instance,
                    &self.observed_class_distribution,
                    &self.attribute_observers,
                );
            }
        }
        self.observed_class_distribution.clone()
    }
}

impl LearningNode for LearningNodeNB {
    fn learn_from_instance(&mut self, instance: Arc<dyn Instance>, hoeffding_tree: &HoeffdingTree) {
        if !self.is_initialized {
            self.attribute_observers = (0..instance.number_of_attributes()).map(|_| None).collect();
            self.is_initialized = true;
        }

        if let Some(class_index) = instance.class_value() {
            let weight = instance.weight();
            self.observed_class_distribution[class_index as usize] += weight
        }

        for i in 0..instance.number_of_attributes() - 1 {
            let instance_attribute_index =
                HoeffdingTree::model_attribute_index_to_instance_attribute_index(i, &instance);

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
