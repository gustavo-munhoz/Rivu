use crate::classifiers::HoeffdingTree;
use crate::classifiers::attribute_class_observers::GaussianNumericAttributeClassObserver;
use crate::classifiers::hoeffding_tree::LeafPredictionOption;
use crate::classifiers::hoeffding_tree::split_criteria::GiniSplitCriterion;
use crate::ui::types::choices::{
    HoeffdingTreeParams, LeafPredictionChoice, NumericEstimatorChoice, SplitCriterionChoice,
};

impl From<HoeffdingTreeParams> for HoeffdingTree {
    fn from(params: HoeffdingTreeParams) -> Self {
        let numeric_estimator = Box::new(match params.numeric_estimator {
            NumericEstimatorChoice::GaussianNumeric(_) => {
                GaussianNumericAttributeClassObserver::new()
            }
        });

        let split_criterion = Box::new(match params.split_criterion {
            SplitCriterionChoice::GiniSplit(_) => GiniSplitCriterion::new(),
        });

        let leaf_prediction = match params.leaf_prediction {
            LeafPredictionChoice::NBAdaptive(_) => LeafPredictionOption::AdaptiveNaiveBayes,
            LeafPredictionChoice::MajorityClass(_) => LeafPredictionOption::MajorityClass,
            LeafPredictionChoice::NaiveBayes(_) => LeafPredictionOption::NaiveBayes,
        };

        HoeffdingTree::new(
            params.max_byte_size,
            numeric_estimator,
            params.memory_estimate_period,
            params.grace_period,
            split_criterion,
            params.split_confidence,
            params.tie_threshold,
            params.binary_splits,
            params.stop_memory_management,
            params.remove_poor_attributes,
            params.no_pre_prune,
            leaf_prediction,
            params.nb_threshold,
        )
    }
}
