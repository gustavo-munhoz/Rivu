use crate::streams::generators::{AgrawalFunction, AgrawalGenerator};
use crate::ui::types::{build::BuildError, choices::*};
use std::convert::TryFrom;

impl TryFrom<AgrawalParameters> for AgrawalGenerator {
    type Error = BuildError;

    fn try_from(p: AgrawalParameters) -> Result<Self, Self::Error> {
        let func = match p.function_id {
            1 => AgrawalFunction::F1,
            2 => AgrawalFunction::F2,
            3 => AgrawalFunction::F3,
            4 => AgrawalFunction::F4,
            5 => AgrawalFunction::F5,
            6 => AgrawalFunction::F6,
            7 => AgrawalFunction::F7,
            8 => AgrawalFunction::F8,
            9 => AgrawalFunction::F9,
            10 => AgrawalFunction::F10,
            _ => {
                return Err(BuildError::InvalidParameter(
                    "function_id must be in 1..=10".into(),
                ));
            }
        };

        let max_instances = p
            .max_instances
            .map(|v| {
                usize::try_from(v).map_err(|_| {
                    BuildError::InvalidParameter("max_instances too large for usize".into())
                })
            })
            .transpose()?;

        AgrawalGenerator::new(func, p.balance, p.perturb_fraction, max_instances, p.seed)
            .map_err(BuildError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    fn base() -> AgrawalParameters {
        AgrawalParameters {
            function_id: 1,
            balance: false,
            perturb_fraction: 0.5,
            max_instances: None,
            seed: 42,
        }
    }

    #[test]
    fn ok_for_all_function_ids_1_to_10() {
        for id in 1u8..=10u8 {
            let mut p = base();
            p.function_id = id;
            let res = AgrawalGenerator::try_from(p);
            assert!(
                res.is_ok(),
                "function_id {id} should be OK, got {:?}",
                res.err()
            );
        }
    }

    #[test]
    fn error_on_function_id_below_range() {
        let mut p = base();
        p.function_id = 0;
        let err = AgrawalGenerator::try_from(p).unwrap_err();
        assert!(
            err.to_string().contains("function_id must be in 1..=10"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn error_on_function_id_above_range() {
        let mut p = base();
        p.function_id = 11;
        let err = AgrawalGenerator::try_from(p).unwrap_err();
        assert!(
            err.to_string().contains("function_id must be in 1..=10"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn ok_when_max_instances_is_none() {
        let p = base();
        let res = AgrawalGenerator::try_from(p);
        assert!(res.is_ok(), "None max_instances should be OK");
    }

    #[test]
    fn ok_when_max_instances_is_usize_max() {
        let mut p = base();
        p.max_instances = Some(usize::MAX as u64);
        let res = AgrawalGenerator::try_from(p);
        assert!(res.is_ok(), "usize::MAX should convert fine");
    }

    #[test]
    fn error_when_perturb_fraction_out_of_range() {
        let mut p = base();
        p.perturb_fraction = 1.5;
        let err = AgrawalGenerator::try_from(p).unwrap_err();
        assert!(
            err.to_string()
                .contains("perturb_fraction must be in 0.0..=1.0"),
            "unexpected error: {err}"
        );
    }
}
