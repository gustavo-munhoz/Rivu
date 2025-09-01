/// Online scalar estimator (e.g., streaming mean).
///
/// Implementations accept values incrementally via [`add`] and expose the
/// current estimate via [`estimation`].
pub trait Estimator {
    /// Incorporates a new observation.
    fn add(&mut self, v: f64);

    /// Returns the current estimate.
    fn estimation(&self) -> f64;
}
