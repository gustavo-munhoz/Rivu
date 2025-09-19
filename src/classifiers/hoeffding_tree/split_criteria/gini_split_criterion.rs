use crate::classifiers::hoeffding_tree::split_criteria::split_criterion::SplitCriterion;

pub struct GiniSplitCriterion {}

impl GiniSplitCriterion {
    pub fn new() -> Self {
        Self {}
    }

    pub fn compute_gini(&self, distribution: &Vec<f64>, distribution_sum_of_weights: f64) -> f64 {
        let mut gini = 1.0;
        for i in distribution {
            let rel_freq = i / distribution_sum_of_weights;
            gini -= rel_freq.powf(2.0);
        }
        gini
    }
}

impl SplitCriterion for GiniSplitCriterion {
    fn get_range_of_merit(&self, pre_split_distribution: &Vec<f64>) -> f64 {
        1.0
    }
}
