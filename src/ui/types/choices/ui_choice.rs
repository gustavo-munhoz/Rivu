use anyhow::Result;
use schemars::{JsonSchema, Schema};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};
use strum::{EnumMessage, IntoEnumIterator};

/// Contract for any “choice enum”
pub trait UIChoice: Sized + Serialize + DeserializeOwned + JsonSchema {
    type Kind: Copy + Into<&'static str> + EnumMessage + IntoEnumIterator;

    /// JSON Schema for the whole tagged enum.
    fn schema() -> Schema;

    fn prompt_label() -> &'static str {
        "Choose a type:"
    }
    fn prompt_help() -> Option<&'static str> {
        Some("↑/↓ to navigate, ↵ to select")
    }

    /// Default `params` JSON for a given kind (usually from `*Parameters::default()`).
    fn default_params(kind: Self::Kind) -> Value;

    /// Optional hook to collect *extra* params via nested wizards
    /// (e.g., learner/stream/evaluator). Default: none.
    fn subprompts<D: crate::ui::cli::drivers::PromptDriver>(
        _driver: &D,
        _kind: Self::Kind,
    ) -> Result<Option<Map<String, Value>>> {
        Ok(None)
    }

    /// Build the typed enum from kind + params.
    fn from_parts(kind: Self::Kind, params: Value) -> Result<Self> {
        let key: &'static str = kind.into();
        let v = json!({ "type": key, "params": params });
        Ok(serde_json::from_value(v)?)
    }
}
