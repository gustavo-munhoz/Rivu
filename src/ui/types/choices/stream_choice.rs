use crate::ui::types::choices::UIChoice;
use schemars::{JsonSchema, Schema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use strum_macros::{Display, EnumDiscriminants, EnumIter, EnumMessage, EnumString, IntoStaticStr};

const DEFAULT_SEED: u64 = 42;
fn default_seed() -> u64 {
    DEFAULT_SEED
}

fn default_sea_function() -> u8 {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct ArffParameters {
    #[schemars(
        with = "String",
        title = "ARFF Path",
        description = "Path to .arff file",
        extend(
            "format" = "path",
            "x-file" = true,
            "x-must-exist" = true,
            "x-extensions" = ["arff"]
        )
    )]
    pub path: PathBuf,

    #[schemars(
        title = "Class Index",
        description = "Zero-based index of the class column",
        range(min = 0)
    )]
    pub class_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct SeaParameters {
    #[serde(default = "default_sea_function")]
    #[schemars(
        title = "Function",
        description = "Classification SEA Function used (1-4)",
        range(min = 1, max = 4),
        default = "default_sea_function"
    )]
    pub function_id: u8,

    #[schemars(title = "Balance", description = "Balance classes during generation?")]
    pub balance: bool,

    #[schemars(
        title = "Noise",
        description = "Noise percentage (0.0–1.0)",
        range(min = 0.0, max = 1.0)
    )]
    pub noise_pct: f32,

    #[serde(default)]
    #[schemars(
        title = "Concept Instances Number",
        description = "The number of instances for each concept"
    )]
    pub max_instances: Option<u64>,

    #[serde(default = "default_seed")]
    #[schemars(title = "Seed", description = "PRNG seed", default = "default_seed")]
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct AgrawalParameters {
    #[schemars(
        title = "Function",
        description = "Agrawal function (1–10)",
        range(min = 1, max = 10)
    )]
    pub function_id: u8,

    #[schemars(title = "Balance", description = "Balance classes during generation?")]
    pub balance: bool,

    #[schemars(
        title = "Perturbation Fraction",
        description = "Drift/perturbation fraction (0.0–1.0)",
        range(min = 0.0, max = 1.0)
    )]
    pub perturb_fraction: f64,

    #[serde(default)]
    #[schemars(
        title = "Max Instances",
        description = "Upper bound on instances; empty = infinite"
    )]
    pub max_instances: Option<u64>,

    #[serde(default = "default_seed")]
    #[schemars(title = "Seed", description = "PRNG seed", default = "default_seed")]
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct AssetNegotiationParameters {
    #[schemars(
        title = "Rule",
        description = "Concept rule (1-5)",
        range(min = 1, max = 5)
    )]
    pub rule_id: u8,

    #[schemars(title = "Balance", description = "Balance classes during generation?")]
    pub balance: bool,

    #[schemars(
        title = "Noise (%)",
        description = "Noise fraction (0.0–1.0)",
        range(min = 0.0, max = 1.0)
    )]
    pub noise_pct: f32,

    #[serde(default = "default_seed")]
    #[schemars(title = "Seed", description = "PRNG seed", default = "default_seed")]
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, EnumDiscriminants)]
#[serde(tag = "type", content = "params", rename_all = "kebab-case")]
#[strum_discriminants(name(StreamKind))]
#[strum_discriminants(derive(EnumIter, EnumString, Display, IntoStaticStr, EnumMessage))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum StreamChoice {
    #[strum_discriminants(strum(
        message = "Arff File Stream",
        detailed_message = "A stream read from an ARFF file."
    ))]
    ArffFile(ArffParameters),

    #[strum_discriminants(strum(
        message = "SEA Generator",
        detailed_message = "Generates SEA concept functions."
    ))]
    SeaGenerator(SeaParameters),

    #[strum_discriminants(strum(
        message = "Agrawal Generator",
        detailed_message = "Generates one of ten different pre-defined loan functions."
    ))]
    AgrawalGenerator(AgrawalParameters),

    #[strum_discriminants(strum(
        message = "Asset Negotiation Generator",
        detailed_message = "Generates instances using 5 concept functions to model agent interest."
    ))]
    AssetNegotiationGenerator(AssetNegotiationParameters),
}

impl UIChoice for StreamChoice {
    type Kind = StreamKind;

    fn schema() -> Schema {
        schema_for!(StreamChoice)
    }

    fn prompt_label() -> &'static str {
        "Choose a stream:"
    }

    fn default_params(kind: Self::Kind) -> Value {
        match kind {
            StreamKind::ArffFile => serde_json::to_value(ArffParameters::default()).unwrap(),
            StreamKind::SeaGenerator => serde_json::to_value(SeaParameters::default()).unwrap(),
            StreamKind::AgrawalGenerator => {
                serde_json::to_value(AgrawalParameters::default()).unwrap()
            }
            StreamKind::AssetNegotiationGenerator => {
                serde_json::to_value(AssetNegotiationParameters::default()).unwrap()
            }
        }
    }
}
