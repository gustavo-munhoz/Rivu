use anyhow::{Context, Result, anyhow, bail};
use schemars::{Schema, schema_for};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    String,
    Integer,
    Number,
    Boolean,
}

#[derive(Debug, Clone)]
pub struct FieldSpec {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub required: bool,
    pub kind: FieldKind,
    pub default: Option<Value>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

// Return the whole tagged-enum schema for T
pub fn schema_for<T: schemars::JsonSchema>() -> Schema {
    schema_for!(T)
}

pub fn specs_for_kind(root: &Schema, kind_key: &str) -> Result<Vec<FieldSpec>> {
    let root_obj = root.as_object().context("root schema is not an object")?;

    let alts = root_obj
        .get("oneOf")
        .or_else(|| root_obj.get("anyOf"))
        .and_then(|v| v.as_array())
        .context("missing oneOf/anyOf")?;

    for branch in alts {
        let bobj = branch.as_object().context("branch is not object")?;
        let props = match bobj.get("properties").and_then(|v| v.as_object()) {
            Some(p) => p,
            None => continue,
        };

        if !discriminant_matches(props, kind_key) {
            continue;
        }

        let params_val = match props.get("params") {
            None => return Ok(vec![]),
            Some(v) => v,
        };

        let mut params_obj = match params_val.as_object() {
            Some(o) => o,
            None => return Ok(vec![]),
        };

        params_obj = match resolve_ref_obj(root_obj, params_obj) {
            Some(o) => o,
            None => return Ok(vec![]),
        };

        let Some(params_props) = params_obj.get("properties").and_then(|v| v.as_object()) else {
            return Ok(vec![]);
        };

        let required: Vec<String> = params_obj
            .get("required")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();

        let mut out = Vec::new();
        for (name, field_schema) in params_props {
            let mut fs_obj = field_schema
                .as_object()
                .context("field schema not object")?;

            if fs_obj.get("$ref").is_some() {
                fs_obj = resolve_ref_obj(root_obj, fs_obj)
                    .ok_or_else(|| anyhow!("failed to resolve field $ref for '{name}'"))?;
            }

            let title = fs_obj
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or(name)
                .to_string();

            let description = fs_obj
                .get("description")
                .and_then(|v| v.as_str())
                .map(str::to_string);

            let default = fs_obj.get("default").cloned();

            let Some(kind) = detect_field_kind(fs_obj.get("type")) else {
                continue;
            };

            let min = fs_obj
                .get("minimum")
                .or_else(|| fs_obj.get("exclusiveMinimum"))
                .and_then(|v| v.as_f64());

            let max = fs_obj
                .get("maximum")
                .or_else(|| fs_obj.get("exclusiveMaximum"))
                .and_then(|v| v.as_f64());

            out.push(FieldSpec {
                name: name.clone(),
                title,
                description,
                required: required.iter().any(|r| r == name),
                kind,
                default,
                min,
                max,
            });
        }

        return Ok(out);
    }

    bail!("no branch found for type={kind_key}");
}

fn discriminant_matches(props: &Map<String, Value>, kind_key: &str) -> bool {
    let Some(tval) = props.get("type") else {
        return false;
    };
    let Some(tobj) = tval.as_object() else {
        return false;
    };

    if tobj.get("const").and_then(|v| v.as_str()) == Some(kind_key) {
        return true;
    }
    if let Some(arr) = tobj.get("enum").and_then(|v| v.as_array()) {
        if arr.len() == 1 && arr[0].as_str() == Some(kind_key) {
            return true;
        }
    }
    false
}

/// Resolve a local $ref like "#/$defs/SeaParameters" against the root object.
/// Returns the referenced object map, or None if it can't be resolved.
fn resolve_ref_obj<'a>(
    root_obj: &'a Map<String, Value>,
    obj: &'a Map<String, Value>,
) -> Option<&'a Map<String, Value>> {
    match obj.get("$ref") {
        Some(Value::String(r)) => {
            let path = r.strip_prefix("#/")?;
            let mut cur: &Map<String, Value> = root_obj;
            for raw_seg in path.split('/') {
                // JSON Pointer unescape (~1 => /, ~0 => ~)
                let seg = raw_seg.replace("~1", "/").replace("~0", "~");
                cur = cur.get(&seg)?.as_object()?;
            }
            Some(cur)
        }
        _ => Some(obj),
    }
}

fn detect_field_kind(ty: Option<&Value>) -> Option<FieldKind> {
    match ty {
        Some(Value::String(s)) => match s.as_str() {
            "string" => Some(FieldKind::String),
            "integer" => Some(FieldKind::Integer),
            "number" => Some(FieldKind::Number),
            "boolean" => Some(FieldKind::Boolean),
            _ => None,
        },
        Some(Value::Array(arr)) => {
            // handle unions like ["null","integer"] for Option<T>
            arr.iter().filter_map(|v| v.as_str()).find_map(|s| match s {
                "string" => Some(FieldKind::String),
                "integer" => Some(FieldKind::Integer),
                "number" => Some(FieldKind::Number),
                "boolean" => Some(FieldKind::Boolean),
                "null" => None,
                _ => None,
            })
        }
        _ => None,
    }
}
