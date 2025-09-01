use crate::evaluation::estimators::Estimator;

/// Streaming mean estimator: `mean = sum / len`.
///
/// Suitable for bounded inputs (e.g., 0/1 accuracy indicators).
#[derive(Debug, Default, Clone, Copy)]
pub struct BasicEstimator {
    len: f64,
    sum: f64,
}

impl Estimator for BasicEstimator {
    #[inline]
    fn add(&mut self, v: f64) {
        if v.is_nan() {
            return;
        }
        self.len += 1.0;
        self.sum += v;
    }

    #[inline]
    fn estimation(&self) -> f64 {
        if self.len > 0.0 {
            self.sum / self.len
        } else {
            f64::NAN
        }
    }
}
