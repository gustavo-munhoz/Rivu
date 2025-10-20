use crate::classifiers::hoeffding_tree::instance_conditional_test::instance_conditional_test::InstanceConditionalTest;
use crate::core::instances::Instance;

#[derive(Clone)]
pub struct NominalAttributeMultiwayTest {
    attribute_index: usize,
}

impl NominalAttributeMultiwayTest {
    pub fn new(attribute_index: usize) -> Self {
        Self { attribute_index }
    }
}

impl InstanceConditionalTest for NominalAttributeMultiwayTest {
    fn branch_for_instance(&self, instance: &dyn Instance) -> Option<usize> {
        if instance
            .is_missing_at_index(self.attribute_index)
            .unwrap_or(true)
        {
            return None;
        }

        Some(instance.value_at_index(self.attribute_index)? as usize)
    }

    fn result_known_for_instance(&self, instance: &dyn Instance) -> bool {
        self.branch_for_instance(instance).is_some_and(|b| b == 0)
    }

    fn max_branches(&self) -> usize {
        usize::MAX
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
