use crate::classifiers::bayes::naive_bayes::NaiveBayes;
use crate::ui::types::choices::NoLearnerParams;

impl From<NoLearnerParams> for NaiveBayes {
    fn from(_: NoLearnerParams) -> Self {
        NaiveBayes::new()
    }
}
