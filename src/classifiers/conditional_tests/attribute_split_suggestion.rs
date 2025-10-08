use crate::classifiers::hoeffding_tree::InstanceConditionalTest;
use std::cmp::Ordering;

pub struct AttributeSplitSuggestion {
    split_test: Option<Box<dyn InstanceConditionalTest>>,
    resulting_class_distribution: Vec<Vec<f64>>,
    merit: f64,
}

impl AttributeSplitSuggestion {
    pub fn new(
        split_test: Option<Box<dyn InstanceConditionalTest>>,
        resulting_class_distribution: Vec<Vec<f64>>,
        merit: f64,
    ) -> Self {
        Self {
            split_test,
            resulting_class_distribution,
            merit,
        }
    }

    pub fn get_split_test(&self) -> Option<&dyn InstanceConditionalTest> {
        self.split_test.as_deref()
    }

    pub fn get_resulting_class_distribution(&self) -> &Vec<Vec<f64>> {
        &self.resulting_class_distribution
    }

    pub fn get_merit(&self) -> f64 {
        self.merit
    }

    pub fn number_of_splits(&self) -> usize {
        self.resulting_class_distribution.len()
    }

    pub fn resulting_class_distribution_from_split(&self, split_index: usize) -> Vec<f64> {
        self.resulting_class_distribution[split_index].clone()
    }
}

impl PartialEq for AttributeSplitSuggestion {
    fn eq(&self, other: &Self) -> bool {
        self.merit == other.merit
    }
}

impl Eq for AttributeSplitSuggestion {}

impl PartialOrd for AttributeSplitSuggestion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.merit.partial_cmp(&other.merit)
    }
}

impl Ord for AttributeSplitSuggestion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.merit.partial_cmp(&other.merit).unwrap()
    }
}
