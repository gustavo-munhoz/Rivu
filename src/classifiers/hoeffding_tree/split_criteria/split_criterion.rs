pub trait SplitCriterion {
    fn get_range_of_merit(&self, pre_split_distribution: &Vec<f64>) -> f64;
}
