use crate::ui::types::choices::UIChoice;
use crate::ui::types::choices::learner::*;
use schemars::{JsonSchema, Schema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum_macros::{Display, EnumDiscriminants, EnumIter, EnumMessage, EnumString, IntoStaticStr};

/// Empty parameter object so the wizard can still look under "params"
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq)]
pub struct NoParams {}

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
    NaiveBayes(NoParams),
    #[strum_discriminants(strum(
        message = "Hoeffding Tree Classifier",
        detailed_message = "Hoeffding Tree / VFDT."
    ))]
    HoeffdingTree(HoeffdingTreeParams),
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
            LearnerKind::NaiveBayes => serde_json::to_value(NoParams::default()).unwrap(),
            LearnerKind::HoeffdingTree => {
                serde_json::to_value(HoeffdingTreeParams::default()).unwrap()
            }
        }
    }

    fn subprompts<D: crate::ui::cli::drivers::PromptDriver>(
        driver: &D,
        kind: Self::Kind,
    ) -> anyhow::Result<Option<serde_json::Map<String, Value>>> {
        use crate::ui::cli::wizard::prompt_choice;

        if let LearnerKind::HoeffdingTree = kind {
            let ne: NumericEstimatorChoice = prompt_choice::<NumericEstimatorChoice, _>(driver)?;
            let sc: SplitCriterionChoice = prompt_choice::<SplitCriterionChoice, _>(driver)?;
            let lp: LeafPredictionChoice = prompt_choice::<LeafPredictionChoice, _>(driver)?;

            let mut extra = serde_json::Map::new();
            extra.insert("numeric_estimator".into(), serde_json::to_value(ne)?);
            extra.insert("split_criterion".into(), serde_json::to_value(sc)?);
            extra.insert("leaf_prediction".into(), serde_json::to_value(lp)?);
            return Ok(Some(extra));
        }
        Ok(None)
    }
}

impl UIChoice for NumericEstimatorChoice {
    type Kind = NumericEstimatorKind;

    fn schema() -> Schema {
        schema_for!(NumericEstimatorChoice)
    }

    fn prompt_label() -> &'static str {
        "Choose a numeric estimator:"
    }

    fn default_params(kind: Self::Kind) -> Value {
        match kind {
            NumericEstimatorKind::GaussianNumeric => {
                serde_json::to_value(GaussianNumericClassObserverParams::default()).unwrap()
            }
        }
    }
}

impl UIChoice for SplitCriterionChoice {
    type Kind = SplitCriterionKind;

    fn schema() -> Schema {
        schema_for!(SplitCriterionChoice)
    }

    fn prompt_label() -> &'static str {
        "Choose a split criterion:"
    }

    fn default_params(kind: Self::Kind) -> Value {
        match kind {
            SplitCriterionKind::GiniSplit => serde_json::to_value(NoParams::default()).unwrap(),
        }
    }
}

impl UIChoice for LeafPredictionChoice {
    type Kind = LeafPredictionKind;

    fn schema() -> Schema {
        schema_for!(LeafPredictionChoice)
    }

    fn prompt_label() -> &'static str {
        "Choose which leaf prediction to use:"
    }

    fn default_params(_: Self::Kind) -> Value {
        serde_json::to_value(NoParams::default()).unwrap()
    }
}
