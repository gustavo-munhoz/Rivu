use crate::ui::types::choices::UIChoice;
use schemars::{JsonSchema, Schema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum_macros::{Display, EnumDiscriminants, EnumIter, EnumMessage, EnumString, IntoStaticStr};

/// Empty parameter object so the wizard can still look under "params"
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct NoLearnerParams {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, EnumDiscriminants)]
#[serde(tag = "type", content = "params", rename_all = "kebab-case")]
#[strum_discriminants(name(LearnerKind))]
#[strum_discriminants(derive(EnumIter, EnumString, Display, IntoStaticStr, EnumMessage))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum LearnerChoice {
    #[strum_discriminants(strum(
        message = "Naive Bayes Classifier",
        detailed_message = "Performs classic Bayesian prediction assuming feature independence."
    ))]
    NaiveBayes(NoLearnerParams),
    // #[strum_discriminants(strum(
    //     message = "Hoeffding Tree Classifier",
    //     detailed_message = "Hoeffding Tree / VFDT."
    // ))]
    // HoeffdingTree(NoLearnerParams),
}

impl UIChoice for LearnerChoice {
    type Kind = LearnerKind;

    fn schema() -> Schema {
        schema_for!(LearnerChoice)
    }

    fn prompt_label() -> &'static str {
        "Choose a learner:"
    }

    fn default_params(kind: Self::Kind) -> Value {
        match kind {
            LearnerKind::NaiveBayes /* | LearnerKind::HoeffdingTree */ => {
                serde_json::to_value(NoLearnerParams::default()).unwrap()
            }
        }
    }
}
