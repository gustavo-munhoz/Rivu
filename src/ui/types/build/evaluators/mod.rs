use crate::evaluation::{BasicClassificationEvaluator, BasicEstimator, PerformanceEvaluator};
use crate::ui::types::build::BuildError;
use crate::ui::types::choices::EvaluatorChoice;

mod basic_classification;

pub fn build_evaluator(
    choice: EvaluatorChoice,
) -> Result<Box<dyn PerformanceEvaluator>, BuildError> {
    match choice {
        EvaluatorChoice::BasicClassification(p) => {
            let ev = BasicClassificationEvaluator::<BasicEstimator>::try_from(p)?;
            Ok(Box::new(ev))
        }
    }
}
