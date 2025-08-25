use crate::classifiers::attribute_class_observers::{
    AttributeClassObserver, GaussianNumericAttributeClassObserver, NominalAttributeClassObserver,
};
use crate::classifiers::classifier::Classifier;
use crate::core::attributes::NominalAttribute;
use crate::core::instance_header::InstanceHeader;
use crate::core::instances::Instance;
use std::sync::Arc;

pub struct NaiveBayes {
    header: Option<Arc<InstanceHeader>>,
    observed_class_distribution: Vec<f64>,
    attribute_observers: Vec<Option<Box<dyn AttributeClassObserver>>>,
}

impl NaiveBayes {
    pub fn new() -> Self {
        Self {
            header: None,
            observed_class_distribution: Vec::new(),
            attribute_observers: Vec::new(),
        }
    }

    #[inline]
    fn ensure_observers_length(&mut self, num_model_atts: usize) {
        if self.attribute_observers.len() < num_model_atts {
            self.attribute_observers
                .resize_with(num_model_atts, || None);
        }
    }

    #[inline]
    fn new_nominal_observer(&self) -> Box<dyn AttributeClassObserver> {
        Box::new(NominalAttributeClassObserver::new())
    }

    #[inline]
    fn new_numeric_observer(&self) -> Box<dyn AttributeClassObserver> {
        Box::new(GaussianNumericAttributeClassObserver::new())
    }

    #[inline]
    fn model_att_index_to_instance_att_index(model_idx: usize, class_idx: usize) -> usize {
        if class_idx > model_idx {
            model_idx
        } else {
            model_idx + 1
        }
    }
}

impl Classifier for NaiveBayes {
    fn get_votes_for_instance(&self, instance: Box<dyn Instance>) -> Option<Vec<f64>> {
        let mut votes = vec![0.0; self.observed_class_distribution.len()];
        let observed_class_sum: f64 = self.observed_class_distribution.iter().copied().sum();

        for class_index in 0..votes.len() {
            let mut score = self.observed_class_distribution[class_index] / observed_class_sum;

            for att_index in 0..(instance.number_of_attributes() - 1) {
                let inst_att_index =
                    Self::model_att_index_to_instance_att_index(att_index, instance.class_index());

                let is_missing = instance.is_missing_at_index(inst_att_index).unwrap_or(true);

                if is_missing {
                    continue;
                };

                let Some(Some(obs)) = self.attribute_observers.get(att_index) else {
                    continue;
                };

                let Some(x) = instance.value_at_index(inst_att_index) else {
                    continue;
                };

                let p = obs
                    .probability_of_attribute_value_given_class(x, class_index)
                    .unwrap_or(0.0);

                score *= p;
            }
            votes[class_index] = score;
        }
        Some(votes)
    }

    fn set_model_context(&mut self, header: Arc<InstanceHeader>) {
        let num_classes = header.number_of_classes();
        let num_model_atts = header.number_of_attributes().saturating_sub(1);

        self.header = Some(header);

        self.observed_class_distribution = vec![0.0; num_classes];

        self.attribute_observers.clear();
        self.attribute_observers
            .resize_with(num_model_atts, || None);
    }

    fn train_on_instance(&mut self, instance: Box<dyn Instance>) {
        let header = match self.header.as_ref() {
            Some(header) => header.clone(),
            None => return,
        };

        let w = instance.weight().max(0.0);
        if w == 0.0 {
            return;
        }

        let class_val = match instance.class_value() {
            Some(c) => c as usize,
            None => return,
        };

        if class_val >= self.observed_class_distribution.len() {
            self.observed_class_distribution.resize(class_val + 1, 0.0);
        }
        self.observed_class_distribution[class_val] += w;

        let class_idx = header.class_index();
        let num_model_atts = instance.number_of_attributes().saturating_sub(1);

        self.ensure_observers_length(num_model_atts);

        for m in 0..num_model_atts {
            let inst_idx = Self::model_att_index_to_instance_att_index(m, class_idx);

            if self.attribute_observers[m].is_none() {
                let is_nominal = header.attributes[inst_idx]
                    .as_any()
                    .is::<NominalAttribute>();

                let obs: Box<dyn AttributeClassObserver> = if is_nominal {
                    self.new_nominal_observer()
                } else {
                    self.new_numeric_observer()
                };
                self.attribute_observers[m] = Some(obs);
            }

            let is_missing = instance.is_missing_at_index(inst_idx).unwrap_or(true);

            if is_missing {
                continue;
            }

            if let Some(x) = instance.value_at_index(inst_idx) {
                if let Some(obs) = self.attribute_observers[m].as_mut() {
                    obs.observe_attribute_class(x, class_val, w);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::attributes::{Attribute, AttributeRef};
    use std::collections::HashMap;
    use std::io::Error;

    struct TestInstance {
        values: Vec<f64>,
        class_idx: usize,
        class_val: Option<f64>,
        weight: f64,
    }

    impl TestInstance {
        fn new(values: Vec<f64>, class_idx: usize, class_val: Option<f64>, weight: f64) -> Self {
            Self {
                values,
                class_idx,
                class_val,
                weight,
            }
        }
    }

    impl Instance for TestInstance {
        fn number_of_attributes(&self) -> usize {
            self.values.len()
        }
        fn class_index(&self) -> usize {
            self.class_idx
        }
        fn weight(&self) -> f64 {
            self.weight
        }
        fn class_value(&self) -> Option<f64> {
            self.class_val
        }
        fn is_missing_at_index(&self, index: usize) -> Result<bool, Error> {
            if index < self.values.len() {
                Ok(self.values[index].is_nan())
            } else {
                Err(Error::new(std::io::ErrorKind::InvalidInput, "oob"))
            }
        }
        fn value_at_index(&self, index: usize) -> Option<f64> {
            self.values.get(index).copied()
        }

        fn set_weight(&mut self, new_value: f64) -> Result<(), Error> {
            panic!("not implemented")
        }

        fn set_value_at_index(&mut self, index: usize, new_value: f64) -> Result<(), Error> {
            panic!("not implemented")
        }

        fn attribute_at_index(&self, index: usize) -> Option<&dyn Attribute> {
            panic!("not implemented")
        }

        fn index_of_attribute(&self, attribute: &dyn Attribute) -> Option<usize> {
            panic!("not implemented")
        }

        fn set_class_value(&mut self, new_value: f64) -> Result<(), Error> {
            panic!("not implemented")
        }

        fn is_class_missing(&self) -> bool {
            panic!("not implemented")
        }

        fn number_of_classes(&self) -> usize {
            panic!("not implemented")
        }

        fn to_vec(&self) -> Vec<f64> {
            panic!("not implemented")
        }

        fn header(&self) -> &InstanceHeader {
            panic!("not implemented")
        }
    }

    #[derive(Clone)]
    struct NumericAttrTest {
        name: String,
    }
    impl NumericAttrTest {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }
    impl Attribute for NumericAttrTest {
        fn name(&self) -> String {
            self.name.clone()
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn arff_representation(&self) -> String {
            format!("@attribute {} numeric", self.name)
        }
    }

    fn nominal_attr_ref(name: &str, values: &[&str]) -> AttributeRef {
        let vals: Vec<String> = values.iter().map(|s| s.to_string()).collect();
        let mut map = HashMap::new();
        for (i, v) in vals.iter().enumerate() {
            map.insert(v.clone(), i);
        }
        Arc::new(NominalAttribute::with_values(name.to_string(), vals, map)) as AttributeRef
    }
    fn numeric_attr_ref(name: &str) -> AttributeRef {
        Arc::new(NumericAttrTest::new(name)) as AttributeRef
    }
    fn approx(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() <= eps
    }
    const EPS: f64 = 1e-9;

    #[test]
    fn votes_with_single_nominal_attribute() {
        let mut nb = NaiveBayes::new();
        nb.observed_class_distribution = vec![4.0, 6.0];
        nb.attribute_observers = vec![None];

        let mut obs = NominalAttributeClassObserver::new();
        obs.observe_attribute_class(1.0, 0, 3.0);
        obs.observe_attribute_class(0.0, 0, 1.0);
        obs.observe_attribute_class(1.0, 1, 1.0);
        obs.observe_attribute_class(0.0, 1, 5.0);
        nb.attribute_observers[0] = Some(Box::new(obs));

        let inst = TestInstance::new(vec![1.0, f64::NAN], 1, None, 1.0);

        let votes = nb.get_votes_for_instance(Box::new(inst)).unwrap();
        assert_eq!(votes.len(), 2);
        assert!(approx(votes[0], 4.0 / 15.0 * 1.0, 1e-12));
        assert!(approx(votes[1], 0.15, EPS));
    }

    #[test]
    fn missing_attribute_is_ignored_in_votes() {
        let mut nb = NaiveBayes::new();
        nb.observed_class_distribution = vec![5.0, 5.0];
        nb.attribute_observers = vec![None];

        let inst = TestInstance::new(vec![f64::NAN, 0.0], 1, None, 1.0);

        let votes = nb.get_votes_for_instance(Box::new(inst)).unwrap();
        assert!(approx(votes[0], 0.5, EPS));
        assert!(approx(votes[1], 0.5, EPS));
    }

    #[test]
    fn gaussian_numeric_observer_affects_votes() {
        let mut nb = NaiveBayes::new();
        nb.observed_class_distribution = vec![3.0, 3.0];
        nb.attribute_observers = vec![None];

        let mut gobs = GaussianNumericAttributeClassObserver::new();
        for &x in &[-1.0, 0.0, 1.0] {
            gobs.observe_attribute_class(x, 0, 1.0);
        }
        for &x in &[4.0, 5.0, 6.0] {
            gobs.observe_attribute_class(x, 1, 1.0);
        }
        nb.attribute_observers[0] = Some(Box::new(gobs));

        let inst_near_c0 = TestInstance::new(vec![0.2, 0.0], 1, None, 1.0);
        let v0 = nb.get_votes_for_instance(Box::new(inst_near_c0)).unwrap();
        assert!(
            v0[0] > v0[1],
            "esperado voto classe 0 > classe 1; got: {:?}",
            v0
        );

        let inst_near_c1 = TestInstance::new(vec![5.2, 0.0], 1, None, 1.0);
        let v1 = nb.get_votes_for_instance(Box::new(inst_near_c1)).unwrap();
        assert!(
            v1[1] > v1[0],
            "esperado voto classe 1 > classe 0; got: {:?}",
            v1
        );
    }

    #[test]
    fn no_observers_means_votes_are_priors() {
        let mut nb = NaiveBayes::new();
        nb.observed_class_distribution = vec![2.0, 6.0];
        nb.attribute_observers = vec![None, None];

        let inst = TestInstance::new(vec![1.0, 2.0, 0.0], 2, None, 1.0);
        let votes = nb.get_votes_for_instance(Box::new(inst)).unwrap();
        let sum = nb.observed_class_distribution.iter().sum::<f64>();
        assert!(approx(votes[0], 2.0 / sum, EPS));
        assert!(approx(votes[1], 6.0 / sum, EPS));
    }

    #[test]
    fn nominal_and_gaussian_combined() {
        let mut nb = NaiveBayes::new();
        nb.observed_class_distribution = vec![5.0, 5.0];
        nb.attribute_observers = vec![None, None];

        let mut nobs = NominalAttributeClassObserver::new();

        nobs.observe_attribute_class(1.0, 0, 4.0);
        nobs.observe_attribute_class(0.0, 0, 1.0);
        nobs.observe_attribute_class(0.0, 1, 4.0);
        nobs.observe_attribute_class(1.0, 1, 1.0);
        nb.attribute_observers[0] = Some(Box::new(nobs));

        let mut gobs = GaussianNumericAttributeClassObserver::new();
        for &x in &[0.0, 0.2, -0.1, 0.1] {
            gobs.observe_attribute_class(x, 0, 1.0);
        }
        for &x in &[3.8, 4.9, 5.1, 6.0] {
            gobs.observe_attribute_class(x, 1, 1.0);
        }
        nb.attribute_observers[1] = Some(Box::new(gobs));

        let inst = TestInstance::new(vec![1.0, 0.05, f64::NAN], 2, None, 1.0);
        let votes = nb.get_votes_for_instance(Box::new(inst)).unwrap();
        assert!(votes[0] > votes[1], "esperado C0> C1. votes={:?}", votes);
    }

    #[test]
    fn train_updates_priors_and_nominal_observer() {
        let a0 = nominal_attr_ref("A0", &["0", "1"]);
        let class_attr = nominal_attr_ref("C", &["c0", "c1"]);
        let header = InstanceHeader::new("rel".into(), vec![a0, class_attr], 1);

        let mut nb = NaiveBayes::new();
        nb.set_model_context(Arc::new(header));

        let class_idx = 1_usize;

        let train = |nb: &mut NaiveBayes, x: f64, c: f64| {
            let inst = TestInstance::new(vec![x, f64::NAN], class_idx, Some(c), 1.0);
            nb.train_on_instance(Box::new(inst));
        };

        train(&mut nb, 1.0, 0.0);
        train(&mut nb, 1.0, 0.0);
        train(&mut nb, 0.0, 0.0);

        train(&mut nb, 0.0, 1.0);
        train(&mut nb, 0.0, 1.0);
        train(&mut nb, 1.0, 1.0);

        assert_eq!(nb.observed_class_distribution.len(), 2);
        assert!(approx(nb.observed_class_distribution[0], 3.0, EPS));
        assert!(approx(nb.observed_class_distribution[1], 3.0, EPS));

        assert_eq!(nb.attribute_observers.len(), 1);
        assert!(nb.attribute_observers[0].is_some());

        let test = TestInstance::new(vec![1.0, f64::NAN], class_idx, None, 1.0);
        let votes = nb.get_votes_for_instance(Box::new(test)).unwrap();
        assert_eq!(votes.len(), 2);
        assert!(approx(votes[0], 0.3, 1e-6), "votes={:?}", votes);
        assert!(approx(votes[1], 0.2, 1e-6), "votes={:?}", votes);
    }

    #[test]
    fn train_ignores_missing_value_but_updates_prior() {
        let a0 = nominal_attr_ref("A0", &["0", "1", "2"]);
        let class_attr = nominal_attr_ref("C", &["c0", "c1"]);
        let header = InstanceHeader::new("rel".into(), vec![a0, class_attr], 1);

        let mut nb = NaiveBayes::new();
        nb.set_model_context(Arc::new(header));

        let inst = TestInstance::new(vec![f64::NAN, f64::NAN], 1, Some(0.0), 2.0);
        nb.train_on_instance(Box::new(inst));

        assert_eq!(nb.observed_class_distribution.len(), 2);
        assert!(approx(nb.observed_class_distribution[0], 2.0, EPS));
        assert!(approx(nb.observed_class_distribution[1], 0.0, EPS));

        assert_eq!(nb.attribute_observers.len(), 1);
        assert!(nb.attribute_observers[0].is_some());
    }

    #[test]
    fn train_numeric_gaussian_observer_affects_votes() {
        let x = numeric_attr_ref("X");
        let class_attr = nominal_attr_ref("C", &["c0", "c1"]);
        let header = InstanceHeader::new("rel".into(), vec![x, class_attr], 1);

        let mut nb = NaiveBayes::new();
        nb.set_model_context(Arc::new(header));

        let class_idx = 1_usize;

        let train = |nb: &mut NaiveBayes, x: f64, c: f64| {
            let inst = TestInstance::new(vec![x, f64::NAN], class_idx, Some(c), 1.0);
            nb.train_on_instance(Box::new(inst));
        };

        for &v in &[-0.5, 0.0, 0.1, 0.2, -0.2] {
            train(&mut nb, v, 0.0);
        }
        for &v in &[4.8, 5.0, 5.2, 6.0, 4.0] {
            train(&mut nb, v, 1.0);
        }

        assert!(approx(nb.observed_class_distribution[0], 5.0, EPS));
        assert!(approx(nb.observed_class_distribution[1], 5.0, EPS));
        assert_eq!(nb.attribute_observers.len(), 1);
        assert!(nb.attribute_observers[0].is_some());

        let near_c0 = TestInstance::new(vec![0.15, f64::NAN], class_idx, None, 1.0);
        let v0 = nb.get_votes_for_instance(Box::new(near_c0)).unwrap();
        assert!(v0[0] > v0[1], "esperado C0 > C1; votes={:?}", v0);

        let near_c1 = TestInstance::new(vec![5.1, f64::NAN], class_idx, None, 1.0);
        let v1 = nb.get_votes_for_instance(Box::new(near_c1)).unwrap();
        assert!(v1[1] > v1[0], "esperado C1 > C0; votes={:?}", v1);
    }
}
