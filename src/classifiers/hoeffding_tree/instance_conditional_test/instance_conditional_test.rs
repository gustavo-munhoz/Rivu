use crate::core::instances::Instance;
use std::sync::Arc;

pub trait InstanceConditionalTest {
    fn branch_for_instance(&self, instance: Arc<dyn Instance>) -> Option<usize>;
    fn result_known_for_instance(&self, instance: Arc<dyn Instance>) -> bool;
    fn max_branches(&self) -> usize;
    fn get_atts_test_depends_on(&self) -> Vec<usize>;
}
