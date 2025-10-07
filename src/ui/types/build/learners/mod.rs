use crate::classifiers::Classifier;
use crate::classifiers::bayes::naive_bayes::NaiveBayes;
use crate::ui::types::build::BuildError;
use crate::ui::types::choices::LearnerChoice;

mod naive_bayes;

pub fn build_learner(choice: LearnerChoice) -> Result<Box<dyn Classifier>, BuildError> {
    match choice {
        LearnerChoice::NaiveBayes(p) => Ok(Box::new(NaiveBayes::from(p))),
        _ => unimplemented!(),
    }
}
