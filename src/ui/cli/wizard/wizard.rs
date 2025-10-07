use anyhow::{Context, Result};
use serde_json::{Map, Value};
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use strum::{EnumMessage, IntoEnumIterator};

use crate::ui::cli::drivers::PromptDriver;
use crate::ui::types::choices::{FieldKind, UIChoice, schema_for, specs_for_kind};

const DIM_ITALIC: &str = "\x1b[2m\x1b[3m";
const RESET: &str = "\x1b[0m";

struct KindItem<K> {
    kind: K,
    text: String,
}

impl<K> Display for KindItem<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

fn kind_items<K>() -> Vec<KindItem<K>>
where
    K: Copy + Into<&'static str> + EnumMessage + IntoEnumIterator,
{
    K::iter()
        .map(|k| {
            let label = k.get_message().unwrap_or_else(|| k.into());
            let desc = k.get_detailed_message().unwrap_or("");
            let text = if desc.is_empty() {
                label.to_string()
            } else {
                format!("{label}  {DIM_ITALIC}{desc}{RESET}")
            };
            KindItem { kind: k, text }
        })
        .collect()
}

pub fn prompt_choice<C: UIChoice, D: PromptDriver>(driver: &D) -> Result<C> {
    let items = kind_items::<C::Kind>();

    let mut select = inquire::Select::new(C::prompt_label(), items);

    if let Some(help) = C::prompt_help() {
        select = select.with_help_message(help);
    }

    let selected = select.prompt()?;
    let choice_kind: C::Kind = selected.kind;

    let key: &'static str = choice_kind.into();
    let schema = schema_for::<C>();
    let specs = specs_for_kind(&schema, key)?;

    let defaults = C::default_params(choice_kind);

    let mut params = Map::new();
    for s in specs {
        let init = s.default.clone().or_else(|| defaults.get(&s.name).cloned());
        let help = s.description.as_deref().unwrap_or("");

        let is_optional_numeric = !s.required
            && matches!(s.kind, FieldKind::Integer | FieldKind::Number)
            && matches!(init, None | Some(Value::Null));

        let val_opt: Option<Value> = if is_optional_numeric {
            let def_txt = match s.kind {
                FieldKind::Integer => init
                    .as_ref()
                    .and_then(|v| v.as_u64())
                    .map(|n| n.to_string()),
                FieldKind::Number => init
                    .as_ref()
                    .and_then(|v| v.as_f64())
                    .map(|x| x.to_string()),
                _ => None,
            }
            .unwrap_or_default();

            let answer = driver.ask_string(
                &s.title,
                &format!("{help}\n(leave blank for none)"),
                &def_txt,
            )?;

            let answer = answer.trim();
            if answer.is_empty() {
                None
            } else {
                Some(match s.kind {
                    FieldKind::Integer => {
                        let n: u64 = answer
                            .parse()
                            .with_context(|| format!("invalid integer for {}", s.title))?;
                        Value::from(n)
                    }
                    FieldKind::Number => {
                        let x: f64 = answer
                            .parse()
                            .with_context(|| format!("invalid number for {}", s.title))?;
                        Value::from(x)
                    }
                    _ => unreachable!(),
                })
            }
        } else {
            Some(match s.kind {
                FieldKind::Boolean => {
                    let def = init.and_then(|v| v.as_bool()).unwrap_or(false);
                    Value::Bool(driver.ask_bool(&s.title, help, def)?)
                }
                FieldKind::String => {
                    let def = init
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                        .unwrap_or_default();
                    let is_arff_path = s.name == "path";

                    let answered = if is_arff_path {
                        let more_help = if help.is_empty() {
                            "Please type a valid .arff file path"
                        } else {
                            help
                        };
                        let pb = prompt_path_until_ok(
                            driver,
                            &s.title,
                            more_help,
                            &def,
                            true,
                            true,
                            &["arff"],
                        )?;
                        pb.to_string_lossy().into_owned()
                    } else {
                        driver.ask_string(&s.title, help, &def)?
                    };

                    Value::String(answered)
                }
                FieldKind::Integer => {
                    let def = init.and_then(|v| v.as_u64()).unwrap_or(0);
                    Value::from(driver.ask_u64(
                        &s.title,
                        help,
                        def,
                        s.min.map(|x| x as u64),
                        s.max.map(|x| x as u64),
                    )?)
                }
                FieldKind::Number => {
                    let def = init.and_then(|v| v.as_f64()).unwrap_or(0.0);
                    Value::from(driver.ask_f64(&s.title, help, def, s.min, s.max)?)
                }
            })
        };

        if let Some(val) = val_opt {
            params.insert(s.name.clone(), val);
        }
    }

    if let Some(extra) = C::subprompts(driver, choice_kind)? {
        params.extend(extra);
    }
    C::from_parts(choice_kind, Value::Object(params))
}

fn validate_path_str(
    input: &str,
    must_exist: bool,
    must_be_file: bool,
    allowed_exts: &[&str],
) -> Result<(), String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Path cannot be empty".into());
    }
    let p = Path::new(trimmed);

    if must_exist && !p.exists() {
        return Err(format!("Path does not exist: {}", p.display()));
    }
    if must_be_file && p.exists() && !p.is_file() {
        return Err("Expected a file path, not a directory".into());
    }
    if !allowed_exts.is_empty() {
        match p.extension().and_then(|e| e.to_str()) {
            Some(ext) if allowed_exts.iter().any(|e| e.eq_ignore_ascii_case(&ext)) => {}
            _ => return Err(format!("Expected a .{} file", allowed_exts.join(" / ."))),
        }
    }
    Ok(())
}

fn prompt_path_until_ok<D: PromptDriver>(
    driver: &D,
    title: &str,
    help: &str,
    default: &str,
    must_exist: bool,
    must_be_file: bool,
    allowed_exts: &[&str],
) -> Result<PathBuf> {
    loop {
        let answer = driver.ask_string(title, help, default)?;
        match validate_path_str(&answer, must_exist, must_be_file, allowed_exts) {
            Ok(()) => return Ok(PathBuf::from(answer)),
            Err(msg) => {
                eprintln!("✗ {}", msg);
            }
        }
    }
}
