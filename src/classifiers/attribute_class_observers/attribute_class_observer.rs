pub trait AttributeClassObserver {
    fn observe_attribute_class(&mut self, att_val: f64, class_val: usize, weight: f64);
    fn probability_of_attribute_value_given_class(
        &self,
        att_val: f64,
        class_val: usize,
    ) -> Option<f64>;
}
