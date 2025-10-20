use crate::classifiers::hoeffding_tree::instance_conditional_test::instance_conditional_test::InstanceConditionalTest;
use crate::core::instances::Instance;

#[derive(Clone)]
pub struct NumericAttributeBinaryTest {
    attribute_index: usize,
    attribute_value: f64,
    equals_passes_test: bool,
}

impl NumericAttributeBinaryTest {
    pub fn new(attribute_index: usize, attribute_value: f64, equals_passes_test: bool) -> Self {
        Self {
            attribute_index,
            attribute_value,
            equals_passes_test,
        }
    }
}

impl InstanceConditionalTest for NumericAttributeBinaryTest {
    fn branch_for_instance(&self, instance: &dyn Instance) -> Option<usize> {
        let value = instance.value_at_index(self.attribute_index)?;

        if value == self.attribute_value as f64 {
            return Some(if self.equals_passes_test { 0 } else { 1 });
        }
        if value < self.attribute_value as f64 {
            return Some(0);
        }
        Some(1)
    }

    fn result_known_for_instance(&self, instance: &dyn Instance) -> bool {
        self.branch_for_instance(instance).is_some_and(|b| b == 0)
    }

    fn max_branches(&self) -> usize {
        2
    }

    fn get_atts_test_depends_on(&self) -> Vec<usize> {
        vec![self.attribute_index]
    }

    fn calc_byte_size(&self) -> usize {
        size_of::<Self>()
    }

    fn clone_box(&self) -> Box<dyn InstanceConditionalTest> {
        Box::new(self.clone())
    }
}
