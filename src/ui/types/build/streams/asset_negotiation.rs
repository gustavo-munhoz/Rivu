use crate::streams::generators::{AssetNegotiationGenerator, AssetRule};
use crate::ui::types::build::BuildError;
use crate::ui::types::choices::AssetNegotiationParameters;
use std::convert::TryFrom;

impl TryFrom<AssetNegotiationParameters> for AssetNegotiationGenerator {
    type Error = BuildError;

    fn try_from(p: AssetNegotiationParameters) -> Result<Self, Self::Error> {
        let rule = AssetRule::try_from(p.rule_id)
            .map_err(|_| BuildError::InvalidParameter("rule_id must be in 1..=5".into()))?;

        AssetNegotiationGenerator::new(rule, p.balance, p.noise_pct, p.seed)
            .map_err(BuildError::from)
    }
}
