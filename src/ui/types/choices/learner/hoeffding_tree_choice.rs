use crate::ui::types::choices::NoParams;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumDiscriminants, EnumIter, EnumMessage, EnumString, IntoStaticStr};

fn default_max_byte_size() -> usize {
    33_554_432
}
fn default_memory_estimate_period() -> usize {
    1_000_000
}
fn default_grace_period() -> usize {
    200
}
fn default_split_confidence() -> f64 {
    0.0
}
fn default_tie_threshold() -> f64 {
    0.05
}
fn default_nb_threshold() -> Option<usize> {
    Some(0)
}
fn default_num_bins() -> usize {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct GaussianNumericClassObserverParams {
    #[serde(default = "default_num_bins")]
    #[schemars(
        title = "Number of bins",
        description = "Histogram bins for numeric observer.",
        default = "default_num_bins"
    )]
    pub num_bins: usize,
}
impl Default for GaussianNumericClassObserverParams {
    fn default() -> Self {
        Self {
            num_bins: default_num_bins(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, EnumDiscriminants, PartialEq)]
#[serde(tag = "type", content = "params", rename_all = "kebab-case")]
#[strum_discriminants(name(NumericEstimatorKind))]
#[strum_discriminants(derive(EnumIter, EnumString, Display, IntoStaticStr, EnumMessage))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum NumericEstimatorChoice {
    #[strum_discriminants(strum(
        message = "Gaussian Numeric Attribute Class Observer",
        detailed_message = "Histogram+Gaussian observer for numeric attributes."
    ))]
    GaussianNumeric(GaussianNumericClassObserverParams),
}
impl Default for NumericEstimatorChoice {
    fn default() -> Self {
        Self::GaussianNumeric(GaussianNumericClassObserverParams::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, EnumDiscriminants, PartialEq)]
#[serde(tag = "type", content = "params", rename_all = "kebab-case")]
#[strum_discriminants(name(SplitCriterionKind))]
#[strum_discriminants(derive(EnumIter, EnumString, Display, IntoStaticStr, EnumMessage))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum SplitCriterionChoice {
    #[strum_discriminants(strum(
        message = "Gini Split Criterion",
        detailed_message = "Use Gini impurity to choose splits."
    ))]
    GiniSplit(NoParams),
}
impl Default for SplitCriterionChoice {
    fn default() -> Self {
        Self::GiniSplit(NoParams::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, EnumDiscriminants, PartialEq)]
#[serde(tag = "type", content = "params", rename_all = "kebab-case")]
#[strum_discriminants(name(LeafPredictionKind))]
#[strum_discriminants(derive(EnumIter, EnumString, Display, IntoStaticStr, EnumMessage))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum LeafPredictionChoice {
    #[serde(rename = "nb-adaptive")]
    #[strum_discriminants(strum(
        serialize = "nb-adaptive",
        message = "Naive Bayes Adaptive",
        detailed_message = "NB vs MC adaptively."
    ))]
    NBAdaptive(NoParams),
    #[strum_discriminants(strum(
        message = "Naive Bayes",
        detailed_message = "Always NB at leaves."
    ))]
    NaiveBayes(NoParams),
    #[strum_discriminants(strum(
        message = "Majority Class",
        detailed_message = "Predict majority class."
    ))]
    MajorityClass(NoParams),
}
impl Default for LeafPredictionChoice {
    fn default() -> Self {
        Self::NBAdaptive(NoParams::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct HoeffdingTreeParams {
    #[serde(default = "default_max_byte_size")]
    #[schemars(
        title = "Maximum byte size",
        description = "Maximum memory consumed by the tree (bytes).",
        default = "default_max_byte_size"
    )]
    pub max_byte_size: usize,

    #[serde(default)]
    #[schemars(skip)]
    pub numeric_estimator: NumericEstimatorChoice,

    #[serde(default = "default_memory_estimate_period")]
    #[schemars(
        title = "Memory estimate period",
        description = "Instances between memory usage checks.",
        default = "default_memory_estimate_period"
    )]
    pub memory_estimate_period: usize,

    #[serde(default = "default_grace_period")]
    #[schemars(
        title = "Grace period",
        description = "Instances a leaf should observe between split attempts.",
        default = "default_grace_period"
    )]
    pub grace_period: usize,

    #[serde(default)]
    #[schemars(skip)]
    pub split_criterion: SplitCriterionChoice,

    #[serde(default = "default_split_confidence")]
    #[schemars(
        title = "Split confidence",
        description = "Allowed error in split decision (0–1).",
        range(min = 0.0, max = 1.0),
        default = "default_split_confidence"
    )]
    pub split_confidence: f64,

    #[serde(default = "default_tie_threshold")]
    #[schemars(
        title = "Tie threshold",
        description = "Force split when merit difference < threshold (0–1).",
        range(min = 0.0, max = 1.0),
        default = "default_tie_threshold"
    )]
    pub tie_threshold: f64,

    #[serde(default)]
    #[schemars(
        title = "Enforce binary splits?",
        description = "Only allow binary splits."
    )]
    pub binary_splits: bool,

    #[serde(default)]
    #[schemars(
        title = "Stop memory management?",
        description = "Stop growing as soon as memory limit is hit."
    )]
    pub stop_memory_management: bool,

    #[serde(default)]
    #[schemars(
        title = "Disable poor attributes?",
        description = "Remove attributes with poor merit."
    )]
    pub remove_poor_attributes: bool,

    #[serde(default)]
    #[schemars(
        title = "Disable pre-pruning?",
        description = "Skip pre-pruning checks."
    )]
    pub no_pre_prune: bool,

    #[serde(default)]
    #[schemars(skip)]
    pub leaf_prediction: LeafPredictionChoice,

    #[serde(default = "default_nb_threshold")]
    #[schemars(
        title = "Naive Bayes threshold",
        description = "Instances before allowing NB at leaves.",
        default = "default_nb_threshold"
    )]
    pub nb_threshold: Option<usize>,
}
impl Default for HoeffdingTreeParams {
    fn default() -> Self {
        Self {
            max_byte_size: default_max_byte_size(),
            numeric_estimator: NumericEstimatorChoice::default(),
            memory_estimate_period: default_memory_estimate_period(),
            grace_period: default_grace_period(),
            split_criterion: SplitCriterionChoice::default(),
            split_confidence: default_split_confidence(),
            tie_threshold: default_tie_threshold(),
            binary_splits: false,
            stop_memory_management: false,
            remove_poor_attributes: false,
            no_pre_prune: false,
            leaf_prediction: LeafPredictionChoice::default(),
            nb_threshold: default_nb_threshold(),
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
    fn default_functions_are_expected() {
        assert_eq!(default_max_byte_size(), 33_554_432);
        assert_eq!(default_memory_estimate_period(), 1_000_000);
        assert_eq!(default_grace_period(), 200);
        assert!((default_split_confidence() - 0.0).abs() < f64::EPSILON);
        assert!((default_tie_threshold() - 0.05).abs() < f64::EPSILON);
        assert_eq!(default_nb_threshold(), Some(0));
        assert_eq!(default_num_bins(), 10);
    }

    #[test]
    fn enum_defaults_are_stable() {
        let NumericEstimatorChoice::GaussianNumeric(p) = NumericEstimatorChoice::default();
        assert_eq!(p.num_bins, 10);
        matches!(
            SplitCriterionChoice::default(),
            SplitCriterionChoice::GiniSplit(_)
        );
        matches!(
            LeafPredictionChoice::default(),
            LeafPredictionChoice::NBAdaptive(_)
        );
    }

    #[test]
    fn params_default_is_populated() {
        let p = HoeffdingTreeParams::default();
        assert_eq!(p.max_byte_size, 33_554_432);
        matches!(
            p.numeric_estimator,
            NumericEstimatorChoice::GaussianNumeric(_)
        );
        assert_eq!(p.memory_estimate_period, 1_000_000);
        assert_eq!(p.grace_period, 200);
        matches!(p.split_criterion, SplitCriterionChoice::GiniSplit(_));
        assert!((p.split_confidence - 0.0).abs() < f64::EPSILON);
        assert!((p.tie_threshold - 0.05).abs() < f64::EPSILON);
        assert!(!p.binary_splits);
        assert!(!p.stop_memory_management);
        assert!(!p.remove_poor_attributes);
        assert!(!p.no_pre_prune);
        matches!(p.leaf_prediction, LeafPredictionChoice::NBAdaptive(_));
        assert_eq!(p.nb_threshold, Some(0));
    }

    #[test]
    fn serde_roundtrip_params() {
        let p0 = HoeffdingTreeParams::default();
        let j = serde_json::to_string(&p0).unwrap();
        let p1: HoeffdingTreeParams = serde_json::from_str(&j).unwrap();
        // spot-check a few fields
        assert_eq!(p0.max_byte_size, p1.max_byte_size);
        assert_eq!(p0.grace_period, p1.grace_period);
        assert_eq!(p0.nb_threshold, p1.nb_threshold);
    }

    #[test]
    fn serde_missing_fields_apply_defaults() {
        let p: HoeffdingTreeParams = serde_json::from_value(json!({})).unwrap();
        assert_eq!(p, HoeffdingTreeParams::default());
    }

    #[test]
    fn tagged_enum_serialization_numeric_estimator() {
        let ne = NumericEstimatorChoice::default();
        let v = serde_json::to_value(ne).unwrap();
        assert_eq!(
            v.get("type").and_then(Value::as_str),
            Some("gaussian-numeric")
        );
        assert_eq!(
            v.get("params")
                .and_then(|x| x.get("num_bins"))
                .and_then(Value::as_u64),
            Some(10)
        );
    }

    #[test]
    fn tagged_enum_serialization_split_criterion() {
        let sc = SplitCriterionChoice::default();
        let v = serde_json::to_value(sc).unwrap();
        assert_eq!(v.get("type").and_then(Value::as_str), Some("gini-split"));
        assert!(
            v.get("params")
                .map(|p| p.as_object().unwrap().is_empty())
                .unwrap_or(false)
        );
    }

    #[test]
    fn tagged_enum_serialization_leaf_prediction() {
        let lp = LeafPredictionChoice::default(); // NBAdaptive
        let v = serde_json::to_value(lp).unwrap();
        assert_eq!(v.get("type").and_then(Value::as_str), Some("nb-adaptive"));
        assert!(
            v.get("params")
                .map(|p| p.as_object().unwrap().is_empty())
                .unwrap_or(false)
        );
    }

    #[test]
    fn schema_skips_nested_choice_fields() {
        let props = root_props_of::<HoeffdingTreeParams>();
        let obj = props.as_object().expect("props object");
        assert!(!obj.contains_key("numeric_estimator"));
        assert!(!obj.contains_key("split_criterion"));
        assert!(!obj.contains_key("leaf_prediction"));
        for key in [
            "max_byte_size",
            "memory_estimate_period",
            "grace_period",
            "split_confidence",
            "tie_threshold",
            "binary_splits",
            "stop_memory_management",
            "remove_poor_attributes",
            "no_pre_prune",
            "nb_threshold",
        ] {
            assert!(obj.contains_key(key), "missing key in schema: {key}");
        }
    }

    #[test]
    fn schema_contains_titles_ranges_and_defaults() {
        let props = root_props_of::<HoeffdingTreeParams>();
        let obj = props.as_object().unwrap();

        let sc = obj.get("split_confidence").unwrap().as_object().unwrap();
        assert!(sc.get("title").is_some());
        assert_eq!(sc.get("minimum").and_then(Value::as_f64), Some(0.0));
        assert_eq!(sc.get("maximum").and_then(Value::as_f64), Some(1.0));
        // Default present
        assert_eq!(
            sc.get("default").and_then(Value::as_f64).unwrap_or(-1.0),
            0.0
        );

        let tt = obj.get("tie_threshold").unwrap().as_object().unwrap();
        assert_eq!(tt.get("minimum").and_then(Value::as_f64), Some(0.0));
        assert_eq!(tt.get("maximum").and_then(Value::as_f64), Some(1.0));

        let bs = obj.get("binary_splits").unwrap().as_object().unwrap();
        assert!(bs.get("title").is_some());
    }

    #[test]
    fn discriminant_messages_are_available() {
        use super::{LeafPredictionKind, NumericEstimatorKind, SplitCriterionKind};

        assert_eq!(
            NumericEstimatorKind::GaussianNumeric.get_message(),
            Some("Gaussian Numeric Attribute Class Observer")
        );

        assert_eq!(
            SplitCriterionKind::GiniSplit.get_message(),
            Some("Gini Split Criterion")
        );

        assert_eq!(
            LeafPredictionKind::NBAdaptive.get_message(),
            Some("Naive Bayes Adaptive")
        );

        assert!(
            NumericEstimatorKind::GaussianNumeric
                .get_detailed_message()
                .is_some()
        );
    }
}
