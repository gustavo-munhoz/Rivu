use crate::classifiers::Classifier;
use crate::core::instance_header::InstanceHeader;
use crate::core::instances::Instance;
use std::sync::Arc;

#[derive(Default)]
pub struct ClassifierNoneVotes;

impl Classifier for ClassifierNoneVotes {
    fn get_votes_for_instance(&self, instance: &dyn Instance) -> Option<Vec<f64>> {
        None
    }
    fn set_model_context(&mut self, header: Arc<InstanceHeader>) {}
    fn train_on_instance(&mut self, instance: Box<dyn Instance>) {}
}
