use crate::ui::cli::wizard::prompt_choice;
use crate::ui::types::choices::{EvaluatorChoice, LearnerChoice, StreamChoice, UIChoice};
use schemars::{JsonSchema, Schema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use strum_macros::{Display, EnumDiscriminants, EnumIter, EnumMessage, EnumString, IntoStaticStr};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrequentialParams {
    #[schemars(skip)]
    pub learner: LearnerChoice,
    #[schemars(skip)]
    pub stream: StreamChoice,
    #[schemars(skip)]
    pub evaluator: EvaluatorChoice,

    #[serde(default)]
    #[schemars(
        title = "Max Instances",
        description = "Stop after this many instances (None = unlimited)"
    )]
    pub max_instances: Option<u64>,

    #[serde(default)]
    #[schemars(
        title = "Max Seconds",
        description = "Stop after this many seconds (None = unlimited)"
    )]
    pub max_seconds: Option<u64>,

    #[schemars(
        title = "Sample Frequency",
        description = "Emit metrics every N instances",
        range(min = 1)
    )]
    pub sample_frequency: u64,

    #[schemars(
        title = "Memory Check Frequency",
        description = "Check memory every N instances",
        range(min = 1)
    )]
    pub mem_check_frequency: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, EnumDiscriminants)]
#[serde(tag = "type", content = "params", rename_all = "kebab-case")]
#[strum_discriminants(name(TaskKind))]
#[strum_discriminants(derive(EnumIter, EnumString, Display, IntoStaticStr, EnumMessage))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum TaskChoice {
    #[strum_discriminants(strum(
        message = "Evaluate Prequential",
        detailed_message = "Interleave test-then-train with periodic reporting."
    ))]
    EvaluatePrequential(PrequentialParams),
}

impl UIChoice for TaskChoice {
    type Kind = TaskKind;

    fn schema() -> Schema {
        schema_for!(TaskChoice)
    }

    fn prompt_label() -> &'static str {
        "Choose a task:"
    }
    fn default_params(kind: Self::Kind) -> Value {
        match kind {
            TaskKind::EvaluatePrequential => json!({
                "max_instances": null,
                "max_seconds": null,
                "sample_frequency": 100_000,
                "mem_check_frequency": 100_000,
            }),
        }
    }

    fn subprompts<D: crate::ui::cli::drivers::PromptDriver>(
        driver: &D,
        kind: Self::Kind,
    ) -> anyhow::Result<Option<Map<String, Value>>> {
        match kind {
            TaskKind::EvaluatePrequential => {
                let learner = prompt_choice::<LearnerChoice, _>(driver)?;
                let stream = prompt_choice::<StreamChoice, _>(driver)?;
                let eval = prompt_choice::<EvaluatorChoice, _>(driver)?;

                let mut m = Map::new();
                m.insert("learner".into(), serde_json::to_value(learner)?);
                m.insert("stream".into(), serde_json::to_value(stream)?);
                m.insert("evaluator".into(), serde_json::to_value(eval)?);
                Ok(Some(m))
            }
        }
    }

    fn from_parts(kind: Self::Kind, params: Value) -> anyhow::Result<Self> {
        match kind {
            TaskKind::EvaluatePrequential => {
                let p: PrequentialParams = serde_json::from_value(params)?;
                Ok(TaskChoice::EvaluatePrequential(p))
            }
        }
    }
}
