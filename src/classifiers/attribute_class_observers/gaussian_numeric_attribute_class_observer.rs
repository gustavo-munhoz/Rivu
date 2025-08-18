use crate::classifiers::attribute_class_observers::attribute_class_observer::AttributeClassObserver;
use crate::core::estimators::gaussian_estimator::GaussianEstimator;
pub struct GaussianNumericAttributeClassObserver {
    min_value_observed_per_class: Vec<f64>,
    max_value_observed_per_class: Vec<f64>,
    attribute_value_distribution_per_class: Vec<Option<GaussianEstimator>>,
}

impl GaussianNumericAttributeClassObserver {
    pub fn new() -> Self {
        GaussianNumericAttributeClassObserver {
            min_value_observed_per_class: Vec::new(),
            max_value_observed_per_class: Vec::new(),
            attribute_value_distribution_per_class: Vec::new(),
        }
    }

    #[inline]
    fn ensure_class(&mut self, class_val: usize) {
        if class_val >= self.attribute_value_distribution_per_class.len() {
            let new_len = class_val + 1;
            self.attribute_value_distribution_per_class
                .resize_with(new_len, || None);
            self.min_value_observed_per_class
                .resize(new_len, f64::INFINITY);
            self.max_value_observed_per_class
                .resize(new_len, f64::NEG_INFINITY);
        }
    }
}

impl AttributeClassObserver for GaussianNumericAttributeClassObserver {
    fn observe_attribute_class(&mut self, att_val: f64, class_val: usize, weight: f64) {
        if att_val.is_nan() {
            return;
        }
        let w = if weight.is_finite() {
            weight.max(0.0)
        } else {
            0.0
        };
        if w == 0.0 {
            return;
        }

        self.ensure_class(class_val);

        let est = self.attribute_value_distribution_per_class[class_val]
            .get_or_insert_with(GaussianEstimator::new);

        if att_val < self.min_value_observed_per_class[class_val] {
            self.min_value_observed_per_class[class_val] = att_val;
        }
        if att_val > self.max_value_observed_per_class[class_val] {
            self.max_value_observed_per_class[class_val] = att_val;
        }

        est.add_observation(att_val, w);
    }

    fn probability_of_attribute_value_given_class(
        &self,
        att_val: f64,
        class_val: usize,
    ) -> Option<f64> {
        if att_val.is_nan() {
            return None;
        }
        match self.attribute_value_distribution_per_class.get(class_val) {
            Some(Some(est)) => Some(est.probability_density(att_val)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const EPS: f64 = 1e-9;

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn starts_empty_returns_none() {
        let obs = GaussianNumericAttributeClassObserver::new();
        assert!(
            obs.probability_of_attribute_value_given_class(0.0, 0)
                .is_none()
        );
    }

    #[test]
    fn lazy_init_and_pdf_reasonable() {
        let mut obs = GaussianNumericAttributeClassObserver::new();

        obs.observe_attribute_class(1.0, 0, 1.0);
        obs.observe_attribute_class(3.0, 0, 1.0);
        obs.observe_attribute_class(2.0, 0, 1.0);

        let p_center = obs
            .probability_of_attribute_value_given_class(2.0, 0)
            .unwrap();
        let p_far1 = obs
            .probability_of_attribute_value_given_class(0.0, 0)
            .unwrap();
        let p_far2 = obs
            .probability_of_attribute_value_given_class(5.0, 0)
            .unwrap();
        assert!(p_center > p_far1);
        assert!(p_center > p_far2);

        assert!(
            obs.probability_of_attribute_value_given_class(2.0, 1)
                .is_none()
        );
    }

    #[test]
    fn updates_min_max_per_class() {
        let mut obs = GaussianNumericAttributeClassObserver::new();

        obs.observe_attribute_class(10.0, 2, 1.0);
        obs.observe_attribute_class(8.0, 2, 1.0);
        obs.observe_attribute_class(12.0, 2, 1.0);

        let p8 = obs
            .probability_of_attribute_value_given_class(8.0, 2)
            .unwrap();
        let p12 = obs
            .probability_of_attribute_value_given_class(12.0, 2)
            .unwrap();
        let p5 = obs
            .probability_of_attribute_value_given_class(5.0, 2)
            .unwrap();
        let p15 = obs
            .probability_of_attribute_value_given_class(15.0, 2)
            .unwrap();
        assert!(p8 > p5);
        assert!(p12 > p15);
    }

    #[test]
    fn ignores_nan_and_zero_weight() {
        let mut obs = GaussianNumericAttributeClassObserver::new();

        obs.observe_attribute_class(f64::NAN, 0, 1.0);
        assert!(
            obs.probability_of_attribute_value_given_class(0.0, 0)
                .is_none()
        );

        obs.observe_attribute_class(10.0, 0, 0.0);
        assert!(
            obs.probability_of_attribute_value_given_class(10.0, 0)
                .is_none()
        );

        obs.observe_attribute_class(10.0, 0, 2.0);
        let p = obs
            .probability_of_attribute_value_given_class(10.0, 0)
            .unwrap();
        assert!(approx_eq(p, 1.0, EPS));
        let p_off = obs
            .probability_of_attribute_value_given_class(9.999_999_999, 0)
            .unwrap();
        assert!(approx_eq(p_off, 0.0, EPS));
    }

    #[test]
    fn class_index_out_of_bounds_returns_none() {
        let mut obs = GaussianNumericAttributeClassObserver::new();
        obs.observe_attribute_class(1.0, 0, 1.0);
        assert!(
            obs.probability_of_attribute_value_given_class(1.0, 5)
                .is_none()
        );
    }

    #[test]
    fn pdf_monotonic_around_mean_for_simple_case() {
        let mut obs = GaussianNumericAttributeClassObserver::new();
        obs.observe_attribute_class(-1.0, 0, 1.0);
        obs.observe_attribute_class(0.0, 0, 1.0);
        obs.observe_attribute_class(1.0, 0, 1.0);

        let p0 = obs
            .probability_of_attribute_value_given_class(0.0, 0)
            .unwrap();
        let p1 = obs
            .probability_of_attribute_value_given_class(1.0, 0)
            .unwrap();
        let p2 = obs
            .probability_of_attribute_value_given_class(2.0, 0)
            .unwrap();

        assert!(p0 > p1);
        assert!(p1 > p2);
    }
}
