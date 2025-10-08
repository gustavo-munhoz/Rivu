use crate::core::instances::Instance;
use std::sync::Arc;

pub trait InstanceConditionalTest: Send + Sync {
    fn branch_for_instance(&self, instance: &dyn Instance) -> Option<usize>;
    fn result_known_for_instance(&self, instance: &dyn Instance) -> bool;
    fn max_branches(&self) -> usize;
    fn get_atts_test_depends_on(&self) -> Vec<usize>;

    fn clone_box(&self) -> Box<dyn InstanceConditionalTest + Send + Sync>;
}

impl Clone for Box<dyn InstanceConditionalTest + Send + Sync> {
    fn clone(&self) -> Box<dyn InstanceConditionalTest + Send + Sync> {
        self.clone_box()
    }
}
