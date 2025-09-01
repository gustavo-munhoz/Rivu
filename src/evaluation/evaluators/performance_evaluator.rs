use crate::core::instances::Instance;
use crate::evaluation::Measurement;

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
