use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::io::{Error, ErrorKind};
use std::sync::Arc;

use crate::core::instance_header::InstanceHeader;
use crate::core::instances::{DenseInstance, Instance};
use crate::streams::stream::Stream;

use super::AssetRule;
use super::domain::{AMOUNT, COLOR, DELAY, PAYMENT, PRICE, build_header};
use super::rules::{evaluate_rule_idx, make_true_sample_idx};

/// Synthetic stream generator for the “Asset Negotiation” concept.
///
/// This generator produces an unbounded stream of instances with five
/// nominal attributes:
/// color, price, payment, amount, deliveryDelay, plus a nominal
/// class attribute (index = 5), where 0 = "interested" and 1 = "notInterested".
///
/// Key properties:
/// - Deterministic: fully reproducible given seed.
/// - Optional class balancing: alternates 0/1; when a sampled example
/// does not match the desired class, a positive pattern is injected to
/// enforce alternation.
/// - Label noise: with probability noise_percentage, the class label
/// is flipped (independently per example).
/// - Fixed schema: header is built once and shared (Arc<InstanceHeader>).
///
/// This type implements [Stream], returning DenseInstances with weight 1.0.
#[derive(Debug)]
pub struct AssetNegotiationGenerator {
    /// RNG seed used to (re)initialize the pseudo-random sequence.
    seed: u64,
    /// Pseudo-random generator; reseeded by [Stream::restart].
    rng: StdRng,
    /// Classification rule to use (R1...R5).
    rule: AssetRule,
    /// Probability ∈ [0, 1] of flipping the class label.
    noise_percentage: f32,
    /// If true, alternates target classes 0 and 1 across outputs.
    balance_classes: bool,
    /// Internal toggle used only when `balance_classes` = true.
    next_class_should_be_zero: bool,
    /// Stream schema: nominal domains for features and class (class index = 5).
    header: Arc<InstanceHeader>,
    /// Number of examples produced since last restart.
    produced: usize,
}

impl AssetNegotiationGenerator {
    /// Creates a new generator configured with:
    /// - rule: which of the five concept definitions to use (R1..R5)
    /// - balance: whether to enforce alternating classes 0/1
    /// - noise_percentage: probability to flip the label, in [0, 1]
    /// - seed: RNG seed, making the stream reproducible
    ///
    /// Returns an error if noise_percentage ∉ [0, 1].
    pub fn new(
        rule: AssetRule,
        balance: bool,
        noise_percentage: f32,
        seed: u64,
    ) -> Result<Self, Error> {
        if !(0.0..=1.0).contains(&noise_percentage) {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "noise_percentage must be in 0.0..=1.0",
            ));
        }

        Ok(Self {
            seed,
            rng: StdRng::seed_from_u64(seed),
            rule,
            noise_percentage,
            balance_classes: balance,
            next_class_should_be_zero: false,
            header: Arc::new(build_header()),
            produced: 0,
        })
    }

    /// Backwards-compatible constructor that accepts a numeric function_id (1..=5)
    /// and converts it to [AssetRule]. Prefer [new] in new code.
    pub fn new_with_id(
        function_id: u8,
        balance: bool,
        noise_percentage: f32,
        seed: u64,
    ) -> Result<Self, Error> {
        let rule = AssetRule::try_from(function_id)?;
        Self::new(rule, balance, noise_percentage, seed)
    }

    /// Bernoulli label noise: flips cls with probability noise_percentage.
    /// Returns the (possibly flipped) class index {0,1}.
    #[inline]
    fn add_noise(&mut self, cls: usize) -> usize {
        if self.rng.random::<f32>() <= self.noise_percentage {
            1 - cls
        } else {
            cls
        }
    }

    /// Uniformly samples one value index from each nominal domain:
    /// (`color`, `price`, `payment`, `amount`, `delay`).
    #[inline]
    fn sample_indices(&mut self) -> [usize; 5] {
        [
            self.rng.random_range(0..COLOR.len()),
            self.rng.random_range(0..PRICE.len()),
            self.rng.random_range(0..PAYMENT.len()),
            self.rng.random_range(0..AMOUNT.len()),
            self.rng.random_range(0..DELAY.len()),
        ]
    }

    /// Packs domain indices and the class into a Vec<f64> in header order.
    /// This is the storage layout used by DenseInstance.
    #[inline]
    fn build_instance_vec(vals: &[usize; 5], cls: usize) -> Vec<f64> {
        vec![
            vals[0] as f64,
            vals[1] as f64,
            vals[2] as f64,
            vals[3] as f64,
            vals[4] as f64,
            cls as f64,
        ]
    }
}

impl Stream for AssetNegotiationGenerator {
    fn header(&self) -> &InstanceHeader {
        &self.header
    }

    /// Always returns true (the stream is unbounded).
    fn has_more_instances(&self) -> bool {
        true
    }

    /// Generates the next example:
    /// 1) sample domain indices for the five attributes;
    /// 2) compute the true class via the selected rule;
    /// 3) if `balance_classes = true`, enforce alternating labels:
    /// - if we need class 1 but sampled 0, mutate the indices into a
    /// positive (class 0) pattern and then label it as 1;
    /// - if we need class 0 and sampled 0, accept as is;
    /// - if we need class 1 and sampled 1, accept as is;
    /// - otherwise, resample;
    /// 4) flip the label with probability `noise_percentage`;
    /// 5) return an `Instance` (weight = 1.0).
    fn next_instance(&mut self) -> Option<Box<dyn Instance>> {
        loop {
            let mut vals = self.sample_indices();
            let class0_or1 = evaluate_rule_idx(self.rule, &vals);

            let want_one = !self.next_class_should_be_zero;

            let out_cls = if !self.balance_classes {
                self.add_noise(class0_or1)
            } else if class0_or1 == 0 && want_one {
                make_true_sample_idx(self.rule, &mut self.rng, &mut vals);
                self.next_class_should_be_zero = !self.next_class_should_be_zero;
                self.add_noise(1)
            } else if class0_or1 == 0 && !want_one {
                self.next_class_should_be_zero = !self.next_class_should_be_zero;
                self.add_noise(0)
            } else if class0_or1 == 1 && want_one {
                self.next_class_should_be_zero = !self.next_class_should_be_zero;
                self.add_noise(1)
            } else {
                continue;
            };

            let inst = DenseInstance::new(
                Arc::clone(&self.header),
                Self::build_instance_vec(&vals, out_cls),
                1.0,
            );
            self.produced += 1;
            return Some(Box::new(inst));
        }
    }

    /// Resets generator state: `RNG` is reseeded with seed, class-alternation
    /// toggle is cleared, and `produced` is set to 0. After this call, the
    /// sequence of outputs matches a fresh generator constructed with the same
    /// parameters.
    fn restart(&mut self) -> Result<(), Error> {
        self.rng = StdRng::seed_from_u64(self.seed);
        self.next_class_should_be_zero = false;
        self.produced = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::attributes::NominalAttribute;
    use crate::streams::generators::asset_negotiation::rules::evaluate_rule_idx;
    use crate::streams::stream::Stream;

    fn label(h: &InstanceHeader, col: usize, idx: usize) -> String {
        let na = h
            .attribute_at_index(col)
            .unwrap()
            .as_any()
            .downcast_ref::<NominalAttribute>()
            .unwrap();
        na.values[idx].clone()
    }
    fn decode(
        g: &AssetNegotiationGenerator,
        v: &[f64],
    ) -> (String, String, String, String, String, usize) {
        let h = g.header();
        let color = label(h, 0, v[0] as usize);
        let price = label(h, 1, v[1] as usize);
        let payment = label(h, 2, v[2] as usize);
        let amount = label(h, 3, v[3] as usize);
        let delay = label(h, 4, v[4] as usize);
        let class = v[5] as usize;
        (color, price, payment, amount, delay, class)
    }

    #[test]
    fn new_validates_inputs() {
        assert!(AssetNegotiationGenerator::new_with_id(0, false, 0.0, 1).is_err());
        assert!(AssetNegotiationGenerator::new_with_id(6, false, 0.0, 1).is_err());
        assert!(AssetNegotiationGenerator::new_with_id(1, false, -0.001, 1).is_err());
        assert!(AssetNegotiationGenerator::new_with_id(1, false, 1.001, 1).is_err());
        assert!(AssetNegotiationGenerator::new_with_id(1, true, 0.0, 1).is_ok());
    }

    #[test]
    fn header_shape_and_domains() {
        let g = AssetNegotiationGenerator::new_with_id(1, false, 0.0, 42).unwrap();
        let h = g.header();
        assert_eq!(h.relation_name(), "asset_negotiation");
        assert_eq!(h.number_of_attributes(), 6);
        assert_eq!(h.class_index(), 5);

        let color = h
            .attribute_at_index(0)
            .unwrap()
            .as_any()
            .downcast_ref::<NominalAttribute>()
            .unwrap();
        assert_eq!(
            color.values,
            vec![
                "black", "blue", "cyan", "brown", "red", "green", "yellow", "magenta"
            ]
        );

        let price = h
            .attribute_at_index(1)
            .unwrap()
            .as_any()
            .downcast_ref::<NominalAttribute>()
            .unwrap();
        assert_eq!(
            price.values,
            vec![
                "veryLow",
                "low",
                "normal",
                "high",
                "veryHigh",
                "quiteHigh",
                "enormous",
                "non_salable"
            ]
        );

        let payment = h
            .attribute_at_index(2)
            .unwrap()
            .as_any()
            .downcast_ref::<NominalAttribute>()
            .unwrap();
        assert_eq!(
            payment.values,
            vec!["0", "30", "60", "90", "120", "150", "180", "210", "240"]
        );

        let amount = h
            .attribute_at_index(3)
            .unwrap()
            .as_any()
            .downcast_ref::<NominalAttribute>()
            .unwrap();
        assert_eq!(
            amount.values,
            vec![
                "veryLow",
                "low",
                "normal",
                "high",
                "veryHigh",
                "quiteHigh",
                "enormous",
                "non_ensured"
            ]
        );

        let delay = h
            .attribute_at_index(4)
            .unwrap()
            .as_any()
            .downcast_ref::<NominalAttribute>()
            .unwrap();
        assert_eq!(
            delay.values,
            vec!["veryLow", "low", "normal", "high", "veryHigh"]
        );

        let class = h
            .attribute_at_index(5)
            .unwrap()
            .as_any()
            .downcast_ref::<NominalAttribute>()
            .unwrap();
        assert_eq!(class.values, vec!["interested", "notInterested"]);
    }

    #[test]
    fn restart_is_reproducible() {
        let mut g = AssetNegotiationGenerator::new_with_id(3, true, 0.25, 2025).unwrap();
        let a1 = g.next_instance().unwrap().to_vec();
        let a2 = g.next_instance().unwrap().to_vec();
        g.restart().unwrap();
        let b1 = g.next_instance().unwrap().to_vec();
        let b2 = g.next_instance().unwrap().to_vec();
        assert_eq!(a1, b1);
        assert_eq!(a2, b2);
    }

    #[test]
    fn noise_zero_vs_one_changes_class() {
        let mut g0 = AssetNegotiationGenerator::new_with_id(4, true, 0.0, 777).unwrap();
        let mut g1 = AssetNegotiationGenerator::new_with_id(4, true, 1.0, 777).unwrap();
        let c0 = g0.next_instance().unwrap().class_value().unwrap();
        let c1 = g1.next_instance().unwrap().class_value().unwrap();
        assert_ne!(c0, c1);
    }

    #[test]
    fn has_more_instances_is_infinite() {
        let g = AssetNegotiationGenerator::new_with_id(2, false, 0.0, 1).unwrap();
        assert!(g.has_more_instances());
    }

    fn check_rule(fid: u8, seed: u64) {
        let mut g = AssetNegotiationGenerator::new_with_id(fid, false, 0.0, seed).unwrap();
        for _ in 0..400 {
            let inst = g.next_instance().unwrap();
            let v = inst.to_vec();

            let vals = [
                v[0] as usize,
                v[1] as usize,
                v[2] as usize,
                v[3] as usize,
                v[4] as usize,
            ];
            let want = evaluate_rule_idx(AssetRule::try_from(fid).unwrap(), &vals);

            let (c, p, pm, a, d, cls) = decode(&g, &v);
            assert_eq!(cls, want, "fid={fid} with ({c},{p},{pm},{a},{d})");
        }
    }

    #[test]
    fn rules_f1_match() {
        check_rule(1, 11);
    }
    #[test]
    fn rules_f2_match() {
        check_rule(2, 22);
    }
    #[test]
    fn rules_f3_match() {
        check_rule(3, 33);
    }
    #[test]
    fn rules_f4_match() {
        check_rule(4, 44);
    }
    #[test]
    fn rules_f5_match() {
        check_rule(5, 55);
    }

    fn collect_patterns(
        fid: u8,
        seed: u64,
        need: &[fn(&(String, String, String, String, String)) -> bool],
    ) {
        let mut g = AssetNegotiationGenerator::new_with_id(fid, true, 0.0, seed).unwrap();
        let mut seen = vec![false; need.len()];
        for _ in 0..2000 {
            let inst = g.next_instance().unwrap();
            let v = inst.to_vec();
            let (c, p, pm, a, d, cls) = decode(&g, &v);
            if cls == 1 {
                let tup = (c, p, pm, a, d);
                for (i, pred) in need.iter().enumerate() {
                    if !seen[i] && pred(&tup) {
                        seen[i] = true;
                    }
                }
            }
            if seen.iter().all(|&b| b) {
                return;
            }
        }
        panic!(
            "não observei todas as variantes de make_true_sample em tempo hábil (fid={fid}, seen={seen:?})"
        );
    }

    #[test]
    fn make_true_variants_f1() {
        let p1 = |t: &(String, String, String, String, String)| t.1 == "normal" && t.3 == "high";
        let p2 = |t: &(String, String, String, String, String)| {
            t.0 == "brown" && t.1 == "veryLow" && t.4 == "high"
        };
        collect_patterns(1, 101, &[p1, p2]);
    }

    #[test]
    fn make_true_variant_f2() {
        let p = |t: &(String, String, String, String, String)| {
            t.1 == "high" && t.3 == "veryHigh" && t.4 == "high"
        };
        collect_patterns(2, 202, &[p]);
    }

    #[test]
    fn make_true_variants_f3() {
        let p1 = |t: &(String, String, String, String, String)| {
            t.1 == "veryLow" && t.2 == "0" && t.3 == "high"
        };
        let p2 = |t: &(String, String, String, String, String)| {
            t.0 == "red" && t.1 == "low" && t.2 == "30"
        };
        collect_patterns(3, 303, &[p1, p2]);
    }

    #[test]
    fn make_true_variants_f4() {
        let p1 = |t: &(String, String, String, String, String)| {
            t.0 == "black" && t.2 == "90" && t.4 == "veryLow"
        };
        let p2 = |t: &(String, String, String, String, String)| {
            t.0 == "magenta" && t.1 == "high" && t.4 == "veryLow"
        };
        collect_patterns(4, 404, &[p1, p2]);
    }

    #[test]
    fn make_true_variants_f5() {
        let p1 = |t: &(String, String, String, String, String)| {
            t.0 == "blue" && t.2 == "60" && t.3 == "low" && t.4 == "normal"
        };
        let p2 = |t: &(String, String, String, String, String)| {
            t.0 == "cyan" && t.3 == "low" && t.4 == "normal"
        };
        collect_patterns(5, 505, &[p1, p2]);
    }

    #[test]
    fn balance_accepts_both_classes() {
        let mut g = AssetNegotiationGenerator::new_with_id(2, true, 0.0, 909).unwrap();
        let mut saw0 = false;
        let mut saw1 = false;
        for _ in 0..200 {
            let inst = g.next_instance().unwrap();
            let c = inst.class_value().unwrap() as u8;
            if c == 0 {
                saw0 = true;
            } else {
                saw1 = true;
            }
            if saw0 && saw1 {
                break;
            }
        }
        assert!(saw0 && saw1);
    }
}
