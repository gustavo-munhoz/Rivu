use crate::streams::generators::{SeaFunction, SeaGenerator};
use crate::ui::types::{build::BuildError, choices::*};
use std::convert::TryFrom;

impl TryFrom<SeaParameters> for SeaGenerator {
    type Error = BuildError;

    fn try_from(parameters: SeaParameters) -> Result<Self, Self::Error> {
        let func = match parameters.function_id {
            1 => SeaFunction::F1,
            2 => SeaFunction::F2,
            3 => SeaFunction::F3,
            4 => SeaFunction::F4,
            _ => {
                return Err(BuildError::InvalidParameter(
                    "function_id must be 1..=4".into(),
                ));
            }
        };

        let noice_percentage = (parameters.noise_pct * 100.0).round().clamp(0.0, 100.0) as u32;

        let max_instances = parameters
            .max_instances
            .map(|v| {
                usize::try_from(v).map_err(|_| {
                    BuildError::InvalidParameter("max_instances too large for usize".into())
                })
            })
            .transpose()?;

        SeaGenerator::new(
            func,
            parameters.balance,
            noice_percentage,
            max_instances,
            parameters.seed,
        )
        .map_err(BuildError::from)
    }
}
