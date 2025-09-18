use crate::classifiers::attribute_class_observers::{
    AttributeClassObserver, GaussianNumericAttributeClassObserver, NominalAttributeClassObserver,
};
use crate::core::instances::Instance;
use std::sync::Arc;

pub struct HoeffdingTree {
    nb_threshold_option: Option<f64>,
}

impl HoeffdingTree {
    pub fn new() -> Self {
        Self {
            nb_threshold_option: None,
        }
    }

    pub fn set_nb_threshold(&mut self, threshold: f64) {
        self.nb_threshold_option = Some(threshold);
    }

    pub fn get_nb_threshold(&self) -> Option<f64> {
        self.nb_threshold_option
    }

    pub fn model_attribute_index_to_instance_attribute_index(
        index: usize,
        instance: &Arc<dyn Instance>,
    ) -> usize {
        let class_index = instance.class_index();
        if class_index > index {
            return index;
        }
        index + 1
    }

    pub fn new_nominal_class_observer(&self) -> Box<dyn AttributeClassObserver> {
        Box::new(NominalAttributeClassObserver::new())
    }

    pub fn new_numeric_class_observer(&self) -> Box<dyn AttributeClassObserver> {
        Box::new(GaussianNumericAttributeClassObserver::new())
    }
}
