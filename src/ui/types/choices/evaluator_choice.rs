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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use schemars::schema_for;
    use serde_json::{Value, json};
    use strum::EnumMessage;

    fn root_props_of<T: JsonSchema>() -> Value {
        let root = schema_for!(T);
        let v = serde_json::to_value(root).expect("schema to JSON");
        v.get("schema")
            .cloned()
            .unwrap_or(v)
            .get("properties")
            .cloned()
            .unwrap_or_else(|| json!({}))
    }

    #[test]
    fn basic_params_default_all_false() {
        let p = BasicClassificationParameters::default();
        assert!(!p.precision_recall_output);
        assert!(!p.precision_per_class);
        assert!(!p.recall_per_class);
        assert!(!p.f1_per_class);
    }

    #[test]
    fn serde_roundtrip_params() {
        let p0 = BasicClassificationParameters {
            precision_recall_output: true,
            precision_per_class: true,
            recall_per_class: false,
            f1_per_class: true,
        };
        let j = serde_json::to_string(&p0).unwrap();
        let p1: BasicClassificationParameters = serde_json::from_str(&j).unwrap();
        assert_eq!(p0.precision_recall_output, p1.precision_recall_output);
        assert_eq!(p0.precision_per_class, p1.precision_per_class);
        assert_eq!(p0.recall_per_class, p1.recall_per_class);
        assert_eq!(p0.f1_per_class, p1.f1_per_class);
    }

    #[test]
    fn serde_missing_fields_apply_defaults() {
        let p: BasicClassificationParameters = serde_json::from_value(json!({})).unwrap();
        assert_eq!(p, BasicClassificationParameters::default());
    }

    #[test]
    fn tagged_enum_serialization_evaluator_choice() {
        let choice = EvaluatorChoice::BasicClassification(BasicClassificationParameters::default());
        let v = serde_json::to_value(choice).unwrap();
        assert_eq!(
            v.get("type").and_then(Value::as_str),
            Some("basic-classification")
        );
        let params = v
            .get("params")
            .and_then(Value::as_object)
            .expect("params object");
        for k in [
            "precision_recall_output",
            "precision_per_class",
            "recall_per_class",
            "f1_per_class",
        ] {
            assert!(params.contains_key(k), "missing key in params: {k}");
            assert_eq!(params[k].as_bool(), Some(false));
        }
    }

    #[test]
    fn default_params_matches_struct_default() {
        let v = <EvaluatorChoice as UIChoice>::default_params(EvaluatorKind::BasicClassification);
        let de: BasicClassificationParameters = serde_json::from_value(v.clone()).unwrap();
        assert_eq!(de, BasicClassificationParameters::default());

        let rebuilt =
            <EvaluatorChoice as UIChoice>::from_parts(EvaluatorKind::BasicClassification, v)
                .unwrap();
        match rebuilt {
            EvaluatorChoice::BasicClassification(p) => {
                assert_eq!(p, BasicClassificationParameters::default());
            }
        }
    }

    #[test]
    fn schema_has_titles_and_defaults_false() {
        let props = root_props_of::<BasicClassificationParameters>();
        let obj = props.as_object().unwrap();

        for k in [
            ("precision_recall_output", "Precision/Recall summary"),
            ("precision_per_class", "Precision per class"),
            ("recall_per_class", "Recall per class"),
            ("f1_per_class", "F1 per class"),
        ] {
            let field = obj.get(k.0).unwrap().as_object().unwrap();
            assert_eq!(field.get("title").and_then(Value::as_str), Some(k.1));
            assert_eq!(field.get("default").and_then(Value::as_bool), Some(false));
        }
    }

    #[test]
    fn discriminant_messages_available() {
        assert_eq!(
            EvaluatorKind::BasicClassification.get_message(),
            Some("Basic Classification")
        );
        assert!(
            EvaluatorKind::BasicClassification
                .get_detailed_message()
                .is_some()
        );
    }
}
