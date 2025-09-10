use crate::core::instances::Instance;
use crate::evaluation::Measurement;
use std::collections::HashMap;

/// Online evaluator of predictive performance.
///
/// A `PerformanceEvaluator` consumes ground-truth examples and their
/// associated prediction scores (class votes) and exposes aggregated
/// metrics via [`performance`].
pub trait PerformanceEvaluator {
    /// Clears internal state/metrics (schema does not change).
    fn reset(&mut self);

    /// Feeds one labeled example and its class votes (one score per class).
    ///
    /// The evaluator is free to interpret/normalize the scores as needed.
    /// If the example has a missing class or the votes are unusable,
    /// the implementation may choose to skip the update.
    fn add_result(&mut self, example: &dyn Instance, class_votes: Vec<f64>);

    /// Returns a snapshot of current metrics.
    fn performance(&self) -> Vec<Measurement>;
}

pub trait PerformanceEvaluatorExt {
    /// Returns (name, Some(value)|None) for each requested metric, preserving order.
    fn metrics<'a, I>(&self, names: I) -> Vec<(String, Option<f64>)>
    where
        I: IntoIterator<Item = &'a str>;

    fn metric(&self, name: &str) -> Option<f64> {
        self.metrics([name]).into_iter().next().unwrap().1
    }
}

impl<T: PerformanceEvaluator + ?Sized> PerformanceEvaluatorExt for T {
    fn metrics<'a, I>(&self, names: I) -> Vec<(String, Option<f64>)>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let ms = self.performance();
        let map: HashMap<_, _> = ms.into_iter().map(|m| (m.name, m.value)).collect();
        names
            .into_iter()
            .map(|n| (n.to_string(), map.get(n).copied()))
            .collect()
    }
}
