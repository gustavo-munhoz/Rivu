use crate::core::instances::Instance;
use crate::evaluation::{Estimator, Measurement, PerformanceEvaluator};

/// Basic online classifier evaluator.
///
/// Tracks:
/// - overall accuracy (`weight_correct`);
/// - marginals of true (`row_kappa`) and predicted (`col_kappa`) classes for Cohen’s κ;
/// - per-class precision and recall (macro-averaged in `performance()`);
/// - two baselines:
///   - **no-change** (predict last true class): `weight_correct_no_change`;
///   - **majority** (predict most frequent class so far): `weight_majority`.
///
/// All updates are **online** and unbounded. This implementation uses
/// simple streaming means; denominators are the number of updates
/// (for precision/recall only when applicable to that class).
pub struct BasicClassificationEvaluator<E: Estimator + Default> {
    weight_correct: E,
    row_kappa: Vec<E>,
    col_kappa: Vec<E>,
    precision: Vec<E>,
    recall: Vec<E>,
    num_classes: usize,
    weight_correct_no_change: E,
    weight_majority: E,
    last_true_class: Option<usize>,
    total_weight: f64,
}

impl<E: Estimator + Default> BasicClassificationEvaluator<E> {
    pub fn new(num_classes: usize) -> Self {
        let make_vec = || (0..num_classes).map(|_| E::default()).collect::<Vec<_>>();
        Self {
            weight_correct: E::default(),
            row_kappa: make_vec(),
            col_kappa: make_vec(),
            precision: make_vec(),
            recall: make_vec(),
            num_classes,
            weight_correct_no_change: E::default(),
            weight_majority: E::default(),
            last_true_class: None,
            total_weight: 0.0,
        }
    }

    #[inline]
    fn argmax(v: &[f64]) -> Option<usize> {
        let mut best = None;
        let mut best_value = f64::NEG_INFINITY;
        for (i, &x) in v.iter().enumerate() {
            if !x.is_finite() {
                continue;
            }
            if best.is_none() || x > best_value {
                best = Some(i);
                best_value = x;
            }
        }
        best
    }

    #[inline]
    fn majority_class(&self) -> Option<usize> {
        let mut best = None;
        let mut best_value = f64::NEG_INFINITY;
        for (i, estimate) in self.col_kappa.iter().enumerate() {
            let p = estimate.estimation();
            if !p.is_finite() {
                continue;
            }
            if best.is_none() || p > best_value {
                best = Some(i);
                best_value = p;
            }
        }
        best
    }
}

impl<E: Estimator + Default> PerformanceEvaluator for BasicClassificationEvaluator<E> {
    fn reset(&mut self) {
        *self = Self::new(self.num_classes)
    }

    fn add_result(&mut self, example: &dyn Instance, class_votes: Vec<f64>) {
        let Some(yf) = example.class_value() else {
            return;
        };
        if !yf.is_finite() {
            return;
        }
        let y = yf as usize;

        let Some(yhat) = Self::argmax(&class_votes) else {
            return;
        };

        let w = example.weight();
        if w <= 0.0 {
            return;
        }

        self.total_weight += w;

        self.weight_correct.add(if yhat == y { w } else { 0.0 });

        if let Some(prev) = self.last_true_class {
            self.weight_correct_no_change
                .add(if prev == y { w } else { 0.0 });
        }

        if let Some(maj) = self.majority_class() {
            self.weight_majority.add(if maj == y { w } else { 0.0 });
        }

        for (c, est) in self.row_kappa.iter_mut().enumerate() {
            est.add(if c == yhat { w } else { 0.0 });
        }

        for (c, est) in self.col_kappa.iter_mut().enumerate() {
            est.add(if c == y { w } else { 0.0 });
        }

        for (c, est) in self.precision.iter_mut().enumerate() {
            if c == yhat {
                est.add(if yhat == y { w } else { 0.0 });
                continue;
            }
            est.add(f64::NAN);
        }

        for (c, est) in self.recall.iter_mut().enumerate() {
            if c == y {
                est.add(if yhat == y { w } else { 0.0 });
                continue;
            }
            est.add(f64::NAN);
        }

        self.last_true_class = Some(y);
    }

    fn performance(&self) -> Vec<Measurement> {
        if self.total_weight <= 0.0 {
            return vec![
                Measurement::new("accuracy", 0.0),
                Measurement::new("kappa", 0.0),
                Measurement::new("kappa_m", 0.0),
                Measurement::new("kappa_t", 0.0),
                Measurement::new("macro_precision", 0.0),
                Measurement::new("macro_recall", 0.0),
            ];
        }

        let p_o = self.weight_correct.estimation();

        let mut p_e = 0.0;
        for c in 0..self.num_classes {
            let pt = self.row_kappa[c].estimation();
            let pp = self.col_kappa[c].estimation();
            if pt.is_finite() && pp.is_finite() {
                p_e += pt * pp;
            }
        }
        let denom = 1.0 - p_e;
        let kappa = if denom.abs() > f64::EPSILON {
            (p_o - p_e) / denom
        } else {
            f64::NAN
        };

        let acc_nc = self.weight_correct_no_change.estimation();
        let acc_maj = self.weight_majority.estimation();
        let kappa_t = {
            let d = 1.0 - acc_nc;
            if d.abs() > f64::EPSILON {
                (p_o - acc_nc) / d
            } else {
                f64::NAN
            }
        };
        let kappa_m = {
            let d = 1.0 - acc_maj;
            if d.abs() > f64::EPSILON {
                (p_o - acc_maj) / d
            } else {
                f64::NAN
            }
        };

        let mut p_sum = 0.0;
        let mut p_cnt = 0usize;
        let mut r_sum = 0.0;
        let mut r_cnt = 0usize;
        for c in 0..self.num_classes {
            let p = self.precision[c].estimation();
            if p.is_finite() {
                p_sum += p;
                p_cnt += 1;
            }
            let r = self.recall[c].estimation();
            if r.is_finite() {
                r_sum += r;
                r_cnt += 1;
            }
        }

        let macro_precision = if p_cnt > 0 {
            p_sum / (p_cnt as f64)
        } else {
            f64::NAN
        };
        let macro_recall = if r_cnt > 0 {
            r_sum / (r_cnt as f64)
        } else {
            f64::NAN
        };

        vec![
            Measurement::new("accuracy", p_o),
            Measurement::new("kappa", kappa),
            Measurement::new("kappa_m", kappa_m),
            Measurement::new("kappa_t", kappa_t),
            Measurement::new("macro_precision", macro_precision),
            Measurement::new("macro_recall", macro_recall),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::attributes::{AttributeRef, NominalAttribute, NumericAttribute};
    use crate::core::instance_header::InstanceHeader;
    use crate::core::instances::DenseInstance;
    use crate::evaluation::BasicEstimator;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn header_binary() -> Arc<InstanceHeader> {
        let mut attrs: Vec<AttributeRef> = Vec::new();
        attrs.push(Arc::new(NumericAttribute::new("x".into())) as AttributeRef);
        let class_vals = vec!["A".into(), "B".into()];
        let mut class_map = HashMap::new();
        class_map.insert("A".into(), 0);
        class_map.insert("B".into(), 1);
        attrs.push(Arc::new(NominalAttribute::with_values(
            "class".into(),
            class_vals,
            class_map,
        )) as AttributeRef);
        Arc::new(InstanceHeader::new("bin".into(), attrs, 1))
    }

    fn inst(h: &Arc<InstanceHeader>, y: usize, w: f64) -> DenseInstance {
        DenseInstance::new(Arc::clone(h), vec![0.0, y as f64], w)
    }

    type Eval = BasicClassificationEvaluator<BasicEstimator>;

    fn votes(pred: usize) -> Vec<f64> {
        if pred == 0 {
            vec![1.0, 0.0]
        } else {
            vec![0.0, 1.0]
        }
    }

    #[test]
    fn perf_is_zero_when_empty() {
        let ev: Eval = Eval::new(2);
        let perf = ev.performance();
        let get = |name: &str| perf.iter().find(|m| m.name == name).unwrap().value;
        assert_eq!(get("accuracy"), 0.0);
        assert_eq!(get("kappa"), 0.0);
        assert_eq!(get("kappa_m"), 0.0);
        assert_eq!(get("kappa_t"), 0.0);
        assert_eq!(get("macro_precision"), 0.0);
        assert_eq!(get("macro_recall"), 0.0);
    }

    #[test]
    fn single_correct_updates_accuracy_and_macros() {
        let h = header_binary();
        let mut ev: Eval = Eval::new(2);

        let i = inst(&h, 1, 1.0);
        ev.add_result(&i, votes(1));

        let perf = ev.performance();
        let get = |name: &str| perf.iter().find(|m| m.name == name).unwrap().value;
        assert!((get("accuracy") - 1.0).abs() < 1e-12);
        assert!((get("macro_precision") - 1.0).abs() < 1e-12);
        assert!((get("macro_recall") - 1.0).abs() < 1e-12);
        assert!(get("kappa").is_nan());
    }

    #[test]
    fn single_incorrect_updates_to_zero() {
        let h = header_binary();
        let mut ev: Eval = Eval::new(2);

        let i = inst(&h, 0, 1.0);
        ev.add_result(&i, votes(1));

        let perf = ev.performance();
        let get = |name: &str| perf.iter().find(|m| m.name == name).unwrap().value;
        assert!((get("accuracy") - 0.0).abs() < 1e-12);
        assert!((get("macro_precision") - 0.0).abs() < 1e-12);
        assert!((get("macro_recall") - 0.0).abs() < 1e-12);
    }

    #[test]
    fn kappa_one_when_perfect_on_balanced() {
        let h = header_binary();
        let mut ev: Eval = Eval::new(2);

        let i0 = inst(&h, 0, 1.0);
        ev.add_result(&i0, votes(0)); // acerta
        let i1 = inst(&h, 1, 1.0);
        ev.add_result(&i1, votes(1)); // acerta

        let perf = ev.performance();
        let kappa = perf.iter().find(|m| m.name == "kappa").unwrap().value;
        assert!((kappa - 1.0).abs() < 1e-12);
    }

    #[test]
    fn kappa_zero_when_accuracy_equals_chance() {
        let h = header_binary();
        let mut ev: Eval = Eval::new(2);

        let i0 = inst(&h, 0, 1.0);
        ev.add_result(&i0, votes(1));
        let i1 = inst(&h, 1, 1.0);
        ev.add_result(&i1, votes(1));

        let perf = ev.performance();
        let kappa = perf.iter().find(|m| m.name == "kappa").unwrap().value;
        assert!(kappa.abs() < 1e-12);
    }

    // #[test]
    // fn kappa_temporal_and_majority() {
    //     let h = header_binary();
    //     let mut ev: Eval = Eval::new(2);
    //
    //     let a = inst(&h, 0, 1.0);
    //     ev.add_result(&a, votes(0));
    //     let b = inst(&h, 1, 1.0);
    //     ev.add_result(&b, votes(1));
    //
    //     let perf = ev.performance();
    //     let get = |name: &str| perf.iter().find(|m| m.name == name).unwrap().value;
    //
    //     assert!((get("accuracy") - 1.0).abs() < 1e-12);
    //     assert!((get("kappa_t") - 1.0).abs() < 1e-12);
    //     assert!((get("kappa_m") - 1.0).abs() < 1e-12);
    // }

    #[test]
    fn weight_zero_is_ignored() {
        let h = header_binary();
        let mut ev: Eval = Eval::new(2);

        let z = inst(&h, 1, 0.0);
        ev.add_result(&z, votes(1));

        let x = inst(&h, 1, 1.0);
        ev.add_result(&x, votes(1));

        let perf = ev.performance();
        let acc = perf.iter().find(|m| m.name == "accuracy").unwrap().value;
        assert!((acc - 1.0).abs() < 1e-12);
    }

    #[test]
    fn reset_clears_metrics() {
        let h = header_binary();
        let mut ev: Eval = Eval::new(2);

        let i = inst(&h, 1, 1.0);
        ev.add_result(&i, votes(1));
        assert!(
            ev.performance()
                .iter()
                .any(|m| m.name == "accuracy" && (m.value - 1.0).abs() < 1e-12)
        );

        ev.reset();
        let perf = ev.performance();
        let get = |name: &str| perf.iter().find(|m| m.name == name).unwrap().value;
        assert_eq!(get("accuracy"), 0.0);
        assert_eq!(get("kappa"), 0.0);
    }
}
