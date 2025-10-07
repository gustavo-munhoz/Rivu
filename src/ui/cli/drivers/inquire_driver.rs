use crate::ui::cli::drivers::PromptDriver;
use anyhow::Result;
use inquire::{Confirm, CustomType, Text, validator::Validation};

pub struct InquireDriver;

impl PromptDriver for InquireDriver {
    fn ask_bool(&self, title: &str, help: &str, default: bool) -> Result<bool> {
        Ok(Confirm::new(title)
            .with_default(default)
            .with_help_message(help)
            .prompt()?)
    }

    fn ask_string(&self, title: &str, help: &str, default: &str) -> Result<String> {
        Ok(Text::new(title)
            .with_initial_value(default)
            .with_help_message(help)
            .prompt()?)
    }

    fn ask_u64(
        &self,
        title: &str,
        help: &str,
        default: u64,
        min: Option<u64>,
        max: Option<u64>,
    ) -> Result<u64> {
        let mut q = CustomType::<u64>::new(title)
            .with_default(default)
            .with_help_message(help);

        if let (Some(lo), Some(hi)) = (min, max) {
            q = q.with_validator(move |x: &u64| {
                if *x >= lo && *x <= hi {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid(
                        format!("Must be between {lo} and {hi}").into(),
                    ))
                }
            });
        } else if let Some(lo) = min {
            q = q.with_validator(move |x: &u64| {
                if *x >= lo {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid(format!("Must be ≥ {lo}").into()))
                }
            });
        } else if let Some(hi) = max {
            q = q.with_validator(move |x: &u64| {
                if *x <= hi {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid(format!("Must be ≤ {hi}").into()))
                }
            });
        }

        Ok(q.prompt()?)
    }

    fn ask_f64(
        &self,
        title: &str,
        help: &str,
        default: f64,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Result<f64> {
        let mut q = CustomType::<f64>::new(title)
            .with_default(default)
            .with_help_message(help);

        if let (Some(lo), Some(hi)) = (min, max) {
            q = q.with_validator(move |x: &f64| {
                if *x >= lo && *x <= hi {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid(
                        format!("Must be between {lo} and {hi}").into(),
                    ))
                }
            });
        } else if let Some(lo) = min {
            q = q.with_validator(move |x: &f64| {
                if *x >= lo {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid(format!("Must be ≥ {lo}").into()))
                }
            });
        } else if let Some(hi) = max {
            q = q.with_validator(move |x: &f64| {
                if *x <= hi {
                    Ok(Validation::Valid)
                } else {
                    Ok(Validation::Invalid(format!("Must be ≤ {hi}").into()))
                }
            });
        }

        Ok(q.prompt()?)
    }
}
