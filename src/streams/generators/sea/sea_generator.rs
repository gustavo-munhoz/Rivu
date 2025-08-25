use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::sync::Arc;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::core::attributes::{AttributeRef, NominalAttribute, NumericAttribute};
use crate::core::instance_header::InstanceHeader;
use crate::core::instances::{DenseInstance, Instance};
use crate::streams::generators::sea::SeaFunction;
use crate::streams::stream::Stream;

#[derive(Debug)]
pub struct SeaGenerator {
    seed: u64,
    rng: StdRng,
    function: SeaFunction,
    threshold: f64,
    balance_classes: bool,
    next_class_should_be_zero: bool,
    noise_percentage: u32,
    header: Arc<InstanceHeader>,
    max_instances: Option<usize>,
    produced: usize,
}

impl SeaGenerator {
    pub fn new(
        function: SeaFunction,
        balance: bool,
        noise_percentage: u32,
        max_instances: Option<usize>,
        seed: u64,
    ) -> Result<Self, Error> {
        if noise_percentage > 100 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Noise percentage must be in [0, 100]",
            ));
        }

        let mut map = HashMap::new();
        map.insert("groupA".to_string(), 0usize);
        map.insert("groupB".to_string(), 1usize);
        let attributes: Vec<AttributeRef> = vec![
            Arc::new(NumericAttribute::new("attrib1".into())) as AttributeRef,
            Arc::new(NumericAttribute::new("attrib2".into())) as AttributeRef,
            Arc::new(NumericAttribute::new("attrib3".into())) as AttributeRef,
            Arc::new(NominalAttribute::with_values(
                "class".into(),
                vec!["groupA".into(), "groupB".into()],
                map,
            )) as AttributeRef,
        ];
        let header = Arc::new(InstanceHeader::new("SEA".into(), attributes, 3));

        Ok(Self {
            seed,
            rng: StdRng::seed_from_u64(seed),
            function,
            threshold: function.threshold(),
            balance_classes: balance,
            next_class_should_be_zero: false,
            noise_percentage,
            header,
            max_instances,
            produced: 0,
        })
    }

    pub fn new_with_threshold(
        threshold: f64,
        balance: bool,
        noise_percentage: u32,
        max_instances: Option<usize>,
        seed: u64,
    ) -> Result<Self, Error> {
        if !(0.0..=20.0).contains(&threshold) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Threshold must be in [0.0, 20.0] for attributes [0,10]",
            ));
        }
        if noise_percentage > 100 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Noise percentage must be in [0, 100]",
            ));
        }
        Self::new(
            SeaFunction::F2,
            balance,
            noise_percentage,
            max_instances,
            seed,
        )
        .map(|mut g| {
            g.threshold = threshold;
            g
        })
    }

    #[inline]
    fn gen_attr(&mut self) -> f64 {
        self.rng.random_range(0.0..10.0)
    }

    #[inline]
    fn determine_class(&self, a1: f64, a2: f64, _a3: f64) -> u8 {
        if a1 + a2 <= self.threshold { 0 } else { 1 }
    }

    #[inline]
    fn maybe_flip_with_noise(&mut self, cls: u8) -> u8 {
        let roll: u32 = self.rng.random_range(1..=100);
        if roll <= self.noise_percentage {
            1 - cls
        } else {
            cls
        }
    }
}

impl Stream for SeaGenerator {
    fn header(&self) -> &InstanceHeader {
        &self.header
    }

    fn has_more_instances(&self) -> bool {
        self.max_instances.map_or(true, |max| self.produced < max)
    }

    fn next_instance(&mut self) -> Option<Box<dyn Instance>> {
        if !self.has_more_instances() {
            return None;
        }

        let (a1, a2, a3, mut cls) = loop {
            let a1 = self.gen_attr();
            let a2 = self.gen_attr();
            let a3 = self.gen_attr();

            let g = self.determine_class(a1, a2, a3);

            if !self.balance_classes {
                break (a1, a2, a3, g);
            } else {
                let want_zero = self.next_class_should_be_zero;
                if (want_zero && g == 0) || (!want_zero && g == 1) {
                    self.next_class_should_be_zero = !self.next_class_should_be_zero;
                    break (a1, a2, a3, g);
                }
            }
        };

        cls = self.maybe_flip_with_noise(cls);

        let inst = DenseInstance::new(Arc::clone(&self.header), vec![a1, a2, a3, cls as f64], 1.0);
        self.produced += 1;
        Some(Box::new(inst))
    }

    fn restart(&mut self) -> Result<(), Error> {
        self.rng = StdRng::seed_from_u64(self.seed);
        self.produced = 0;
        self.next_class_should_be_zero = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::attributes::NominalAttribute;
    use crate::streams::stream::Stream;

    fn classes_from(generator: &mut SeaGenerator, n: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            let inst = generator.next_instance().expect("instance");
            let v = inst.to_vec();
            out.push(v[3] as u8);
        }
        out
    }

    #[test]
    fn header_shape_and_labels_match_moa() {
        let generator = SeaGenerator::new(SeaFunction::F1, false, 0, Some(1), 42).unwrap();
        let h = generator.header();
        assert_eq!(h.number_of_attributes(), 4);
        assert_eq!(h.class_index(), 3);
        assert_eq!(h.attribute_at_index(0).unwrap().name(), "attrib1");
        assert_eq!(h.attribute_at_index(1).unwrap().name(), "attrib2");
        assert_eq!(h.attribute_at_index(2).unwrap().name(), "attrib3");
        assert_eq!(h.attribute_at_index(3).unwrap().name(), "class");

        let class_attr = h
            .attribute_at_index(3)
            .unwrap()
            .as_any()
            .downcast_ref::<NominalAttribute>()
            .unwrap();
        assert_eq!(
            class_attr.values,
            vec!["groupA".to_string(), "groupB".to_string()]
        );
        assert_eq!(class_attr.label_to_index.get("groupA").copied(), Some(0));
        assert_eq!(class_attr.label_to_index.get("groupB").copied(), Some(1));
    }

    #[test]
    fn class_rule_matches_threshold_f1_no_noise_no_balance() {
        let threshold = SeaFunction::F1.threshold();
        let mut generator = SeaGenerator::new(SeaFunction::F1, false, 0, Some(500), 123).unwrap();
        for _ in 0..200 {
            let inst = generator.next_instance().unwrap();
            let v = inst.to_vec();
            let (a1, a2, cls) = (v[0], v[1], v[3]);
            assert!(a1 >= 0.0 && a1 < 10.0);
            assert!(a2 >= 0.0 && a2 < 10.0);
            let rule_is_zero = a1 + a2 <= threshold + 1e-12;
            assert_eq!(
                rule_is_zero,
                cls == 0.0,
                "a1={a1}, a2={a2}, sum={}, cls={cls}",
                a1 + a2
            );
        }
    }

    #[test]
    fn balance_true_alternates_classes_starting_with_one() {
        let mut generator = SeaGenerator::new(SeaFunction::F2, true, 0, Some(20), 7).unwrap();
        let got = classes_from(&mut generator, 10);
        let expected: Vec<u8> = (0..10).map(|i| if i % 2 == 0 { 1 } else { 0 }).collect();
        assert_eq!(got, expected);
    }

    #[test]
    fn noise_100_percent_flips_all_classes_when_base_is_all_zero() {
        let mut generator =
            SeaGenerator::new_with_threshold(20.0, false, 100, Some(50), 99).unwrap();
        let got = classes_from(&mut generator, 50);
        assert!(
            got.iter().all(|&c| c == 1),
            "esperava todas classes == 1, got={:?}",
            got
        );
    }

    #[test]
    fn restart_resets_sequence_with_same_seed() {
        let mut generator = SeaGenerator::new(SeaFunction::F3, true, 10, Some(100), 12345).unwrap();
        let first: Vec<Vec<f64>> = (0..30)
            .map(|_| generator.next_instance().unwrap().to_vec())
            .collect();
        generator.restart().unwrap();
        let second: Vec<Vec<f64>> = (0..30)
            .map(|_| generator.next_instance().unwrap().to_vec())
            .collect();
        assert_eq!(first, second);
    }

    #[test]
    fn invalid_parameters_are_rejected() {
        // noise > 100
        let err = SeaGenerator::new(SeaFunction::F1, false, 101, None, 1).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);

        let err = SeaGenerator::new_with_threshold(-0.1, false, 0, None, 1).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);

        let err = SeaGenerator::new_with_threshold(20.1, false, 0, None, 1).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn all_four_functions_respect_their_thresholds() {
        let cases = [
            (SeaFunction::F1, 8.0),
            (SeaFunction::F2, 9.0),
            (SeaFunction::F3, 7.0),
            (SeaFunction::F4, 9.5),
        ];
        for (f, thr) in cases {
            let mut generator = SeaGenerator::new(f, false, 0, Some(200), 2025).unwrap();
            for _ in 0..50 {
                let inst = generator.next_instance().unwrap();
                let v = inst.to_vec();
                let (a1, a2, cls) = (v[0], v[1], v[3]);
                let rule_is_zero = a1 + a2 <= thr + 1e-12;
                assert_eq!(
                    rule_is_zero,
                    cls == 0.0,
                    "func={f:?} a1+a2={} thr={thr} cls={cls}",
                    a1 + a2
                );
            }
        }
    }
}
