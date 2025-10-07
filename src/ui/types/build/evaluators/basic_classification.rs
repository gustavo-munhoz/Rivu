use crate::evaluation::{BasicClassificationEvaluator, BasicEstimator};
use crate::ui::types::build::BuildError;
use crate::ui::types::choices::BasicClassificationParameters;

impl TryFrom<BasicClassificationParameters> for BasicClassificationEvaluator<BasicEstimator> {
    type Error = BuildError;

    fn try_from(p: BasicClassificationParameters) -> Result<Self, Self::Error> {
        Ok(BasicClassificationEvaluator::new(
            0,
            p.precision_recall_output,
            p.precision_per_class,
            p.recall_per_class,
            p.f1_per_class,
        ))
    }
}
