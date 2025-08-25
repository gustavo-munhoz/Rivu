use crate::classifiers::hoeffding_tree::instance_conditional_test::instance_conditional_test::InstanceConditionalTest;
use crate::core::instances::Instance;
use std::sync::Arc;

struct NominalAttributeBinaryTest {
    attribute_index: usize,
    attribute_value: usize,
}

impl NominalAttributeBinaryTest {
    pub fn new(attribute_index: usize, attribute_value: usize) -> Self {
        Self {
            attribute_index,
            attribute_value,
        }
    }
}

impl InstanceConditionalTest for NominalAttributeBinaryTest {
    fn branch_for_instance(&self, instance: Arc<dyn Instance>) -> Option<usize> {
        let index = if self.attribute_index < instance.class_index() {
            self.attribute_index
        } else {
            self.attribute_index + 1
        };

        if instance.is_missing_at_index(index).ok()? {
            return None;
        }

        let value = instance.value_at_index(index)?;

        Some((value as usize != self.attribute_value) as usize)
    }

    fn result_known_for_instance(&self, instance: Arc<dyn Instance>) -> bool {
        self.branch_for_instance(instance).is_some_and(|b| b == 0)
    }

    fn max_branches(&self) -> usize {
        2
    }

    fn get_atts_test_depends_on(&self) -> Vec<usize> {
        vec![self.attribute_index]
    }
}
