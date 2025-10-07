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
    show_pr_summary: bool,
    show_precision_per_class: bool,
    show_recall_per_class: bool,
    show_f1_per_class: bool,
}

impl<E: Estimator + Default> BasicClassificationEvaluator<E> {
    pub fn new(
        num_classes: usize,
        show_pr_summary: bool,
        show_precision_per_class: bool,
        show_recall_per_class: bool,
        show_f1_per_class: bool,
    ) -> Self {
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
            show_pr_summary,
            show_precision_per_class,
            show_recall_per_class,
            show_f1_per_class,
        }
    }

    pub fn new_with_default_flags(num_classes: usize) -> Self {
        Self::new(num_classes, false, false, false, false)
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

    #[inline]
    fn init_vectors(&mut self, k: usize) {
        let make_vec = || (0..k).map(|_| E::default()).collect::<Vec<_>>();
        self.row_kappa = make_vec();
        self.col_kappa = make_vec();
        self.precision = make_vec();
        self.recall = make_vec();
        self.num_classes = k;
    }

    #[inline]
    fn ensure_initialized(&mut self, k_hint: usize) {
        if k_hint == 0 {
            return;
        }
        if self.num_classes == 0 {
            self.init_vectors(k_hint);
            return;
        }
        if k_hint > self.num_classes {
            let add = k_hint - self.num_classes;
            self.row_kappa.extend((0..add).map(|_| E::default()));
            self.col_kappa.extend((0..add).map(|_| E::default()));
            self.precision.extend((0..add).map(|_| E::default()));
            self.recall.extend((0..add).map(|_| E::default()));
            self.num_classes = k_hint;
        }
    }
}

impl<E: Estimator + Default> PerformanceEvaluator for BasicClassificationEvaluator<E> {
    fn reset(&mut self) {
        *self = Self::new(
            self.num_classes,
            self.show_pr_summary,
            self.show_precision_per_class,
            self.show_recall_per_class,
            self.show_f1_per_class,
        )
    }

    fn add_result(&mut self, example: &dyn Instance, class_votes: Vec<f64>) {
        let Some(yf) = example.class_value() else {
            return;
        };
        if !yf.is_finite() {
            return;
        }
        let y = yf as usize;

        let k_hint = class_votes.len().max(y + 1);
        self.ensure_initialized(k_hint);

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
            } else {
                est.add(f64::NAN);
            }
        }
        for (c, est) in self.recall.iter_mut().enumerate() {
            if c == y {
                est.add(if yhat == y { w } else { 0.0 });
            } else {
                est.add(f64::NAN);
            }
        }

        self.last_true_class = Some(y);
    }

    fn performance(&self) -> Vec<Measurement> {
        let mut m = vec![Measurement::new(
            "accuracy",
            self.weight_correct.estimation(),
        )];

        if self.total_weight <= 0.0 {
            m.extend([
                Measurement::new("kappa", 0.0),
                Measurement::new("kappa_t", 0.0),
                Measurement::new("kappa_m", 0.0),
            ]);
            return m;
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

        m.push(Measurement::new("kappa", kappa));
        m.push(Measurement::new("kappa_t", kappa_t));
        m.push(Measurement::new("kappa_m", kappa_m));

        if self.show_pr_summary {
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

            let macro_f1 = {
                let s = macro_precision + macro_recall;
                if macro_precision.is_finite() && macro_recall.is_finite() && s > f64::EPSILON {
                    2.0 * (macro_precision * macro_recall) / s
                } else {
                    f64::NAN
                }
            };

            m.push(Measurement::new("precision", macro_precision));
            m.push(Measurement::new("recall", macro_recall));
            m.push(Measurement::new("f1", macro_f1));
        }

        if self.show_precision_per_class {
            for c in 0..self.num_classes {
                m.push(Measurement::new(
                    &format!("precision_class_{c}"),
                    self.precision[c].estimation(),
                ));
            }
        }
        if self.show_recall_per_class {
            for c in 0..self.num_classes {
                m.push(Measurement::new(
                    &format!("recall_class_{c}"),
                    self.recall[c].estimation(),
                ));
            }
        }
        if self.show_f1_per_class {
            for c in 0..self.num_classes {
                let p = self.precision[c].estimation();
                let r = self.recall[c].estimation();
                let s = p + r;
                let f1 = if p.is_finite() && r.is_finite() && s > f64::EPSILON {
                    2.0 * (p * r) / s
                } else {
                    f64::NAN
                };
                m.push(Measurement::new(&format!("f1_class_{c}"), f1));
            }
        }
        m
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
        let ev: Eval = Eval::new_with_default_flags(2);
        let perf = ev.performance();
        let get = |name: &str| perf.iter().find(|m| m.name == name).unwrap().value;
        assert!(get("accuracy").is_nan());
        assert_eq!(get("kappa"), 0.0);
        assert_eq!(get("kappa_m"), 0.0);
        assert_eq!(get("kappa_t"), 0.0);
    }

    #[test]
    fn single_correct_updates_accuracy_and_flags_summary_when_enabled() {
        let h = header_binary();
        type Eval = BasicClassificationEvaluator<BasicEstimator>;
        let mut ev: Eval = Eval::new(2, true, false, false, false);

        let i = inst(&h, 1, 1.0);
        ev.add_result(&i, votes(1));

        let perf = ev.performance();
        let get = |name: &str| perf.iter().find(|m| m.name == name).unwrap().value;

        assert!((get("accuracy") - 1.0).abs() < 1e-12);
        assert!(get("kappa").is_nan()); // denom==0 with one perfect instance

        // Since we enabled summary, these exist and equal 1
        assert!((get("precision") - 1.0).abs() < 1e-12);
        assert!((get("recall") - 1.0).abs() < 1e-12);
        assert!((get("f1") - 1.0).abs() < 1e-12);
    }

    #[test]
    fn summary_metrics_present_only_when_enabled() {
        let h = header_binary();
        type Eval = BasicClassificationEvaluator<BasicEstimator>;

        // OFF
        let mut ev = Eval::new(2, false, false, false, false);
        ev.add_result(&inst(&h, 1, 1.0), votes(1));
        let perf = ev.performance();
        assert!(perf.iter().find(|m| m.name == "precision").is_none());
        assert!(perf.iter().find(|m| m.name == "recall").is_none());
        assert!(perf.iter().find(|m| m.name == "f1").is_none());

        // ON
        let mut ev = Eval::new(2, true, false, false, false);
        ev.add_result(&inst(&h, 1, 1.0), votes(1));
        let perf = ev.performance();
        let get = |name: &str| perf.iter().find(|m| m.name == name).unwrap().value;
        assert!((get("precision") - 1.0).abs() < 1e-12);
        assert!((get("recall") - 1.0).abs() < 1e-12);
        assert!((get("f1") - 1.0).abs() < 1e-12);
    }

    #[test]
    fn per_class_metrics_present_only_when_enabled() {
        let h = header_binary();
        type Eval = BasicClassificationEvaluator<BasicEstimator>;
        let mut ev = Eval::new(2, false, true, true, true);

        ev.add_result(&inst(&h, 0, 1.0), votes(0));
        ev.add_result(&inst(&h, 1, 1.0), votes(0));

        let perf = ev.performance();
        let has = |name: &str| perf.iter().any(|m| m.name == name);
        for name in [
            "precision_class_0",
            "precision_class_1",
            "recall_class_0",
            "recall_class_1",
            "f1_class_0",
            "f1_class_1",
        ] {
            assert!(has(name));
        }
    }

    #[test]
    fn single_incorrect_updates_to_zero() {
        let h = header_binary();
        type Eval = BasicClassificationEvaluator<BasicEstimator>;
        let mut ev: Eval = Eval::new_with_default_flags(2);

        let i = inst(&h, 0, 1.0);
        ev.add_result(&i, votes(1));

        let perf = ev.performance();
        let get = |name: &str| perf.iter().find(|m| m.name == name).unwrap().value;

        assert!((get("accuracy") - 0.0).abs() < 1e-12);
        assert!(perf.iter().find(|m| m.name == "precision").is_none());
        assert!(perf.iter().find(|m| m.name == "recall").is_none());
        assert!(perf.iter().find(|m| m.name == "f1").is_none());
    }

    #[test]
    fn kappa_one_when_perfect_on_balanced() {
        let h = header_binary();
        let mut ev: Eval = Eval::new_with_default_flags(2);

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
        let mut ev: Eval = Eval::new_with_default_flags(2);

        let i0 = inst(&h, 0, 1.0);
        ev.add_result(&i0, votes(1));
        let i1 = inst(&h, 1, 1.0);
        ev.add_result(&i1, votes(1));

        let perf = ev.performance();
        let kappa = perf.iter().find(|m| m.name == "kappa").unwrap().value;
        assert!(kappa.abs() < 1e-12);
    }

    #[test]
    fn weight_zero_is_ignored() {
        let h = header_binary();
        let mut ev: Eval = Eval::new_with_default_flags(2);

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
        type Eval = BasicClassificationEvaluator<BasicEstimator>;
        let mut ev: Eval = Eval::new_with_default_flags(2);

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

        assert!(get("accuracy").is_nan());
        assert_eq!(get("kappa"), 0.0);
    }
}
