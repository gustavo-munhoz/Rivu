pub trait SplitCriterion {
    fn get_range_of_merit(&self, pre_split_distribution: &Vec<f64>) -> f64;
    fn get_merit_of_split(
        &self,
        pre_split_distribution: &[f64],
        post_split_dists: &[Vec<f64>],
    ) -> f64;
}
