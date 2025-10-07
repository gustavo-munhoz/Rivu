use crate::ui::types::choices::UIChoice;
use schemars::{JsonSchema, Schema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum_macros::{Display, EnumDiscriminants, EnumIter, EnumMessage, EnumString, IntoStaticStr};

fn default_false() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, EnumDiscriminants)]
#[serde(tag = "type", content = "params", rename_all = "kebab-case")]
#[strum_discriminants(name(EvaluatorKind))]
#[strum_discriminants(derive(EnumIter, EnumString, Display, IntoStaticStr, EnumMessage))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum EvaluatorChoice {
    #[strum_discriminants(strum(
        message = "Basic Classification",
        detailed_message = "Online classification metrics (accuracy, precision/recall, kappa, etc.)."
    ))]
    BasicClassification(BasicClassificationParameters),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct BasicClassificationParameters {
    #[serde(default = "default_false")]
    #[schemars(
        title = "Precision/Recall summary",
        description = "Include a global precision/recall summary in the output?",
        default = "default_false"
    )]
    pub precision_recall_output: bool,

    #[serde(default = "default_false")]
    #[schemars(
        title = "Precision per class",
        description = "Track precision broken down by class?",
        default = "default_false"
    )]
    pub precision_per_class: bool,

    #[serde(default = "default_false")]
    #[schemars(
        title = "Recall per class",
        description = "Track recall broken down by class?",
        default = "default_false"
    )]
    pub recall_per_class: bool,

    #[serde(default = "default_false")]
    #[schemars(
        title = "F1 per class",
        description = "Track F1 score broken down by class?",
        default = "default_false"
    )]
    pub f1_per_class: bool,
}

impl UIChoice for EvaluatorChoice {
    type Kind = EvaluatorKind;

    fn schema() -> Schema {
        schema_for!(EvaluatorChoice)
    }

    fn prompt_label() -> &'static str {
        "Choose an evaluator:"
    }
    fn default_params(kind: Self::Kind) -> Value {
        match kind {
            EvaluatorKind::BasicClassification => {
                serde_json::to_value(BasicClassificationParameters::default()).unwrap()
            }
        }
    }
}
