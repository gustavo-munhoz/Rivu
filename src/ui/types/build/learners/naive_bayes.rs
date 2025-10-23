use crate::classifiers::NaiveBayes;
use crate::ui::types::choices::NoParams;

impl From<NoParams> for NaiveBayes {
    fn from(_: NoParams) -> Self {
        NaiveBayes::new()
    }
}
