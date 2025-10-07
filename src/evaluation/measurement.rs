/// Summarized scalar metric produced by a performance evaluator.
///
/// Typical examples: `"accuracy"`, `"kappa"`, `"log_loss"`.
#[derive(Debug, Clone, PartialEq)]
pub struct Measurement {
    pub name: String,
    pub value: f64,
}

impl Measurement {
    /// Convenience constructor
    #[inline]
    pub fn new<N: Into<String>>(name: N, value: f64) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }
}
