use crate::streams::Stream;
use crate::streams::arff::ArffFileStream;
use crate::streams::generators::{AgrawalGenerator, AssetNegotiationGenerator, SeaGenerator};
use crate::ui::types::build::BuildError;
use crate::ui::types::choices::StreamChoice;

mod agrawal;
mod arff_file;
mod asset_negotiation;
mod sea_generator;

pub fn build_stream(choice: StreamChoice) -> Result<Box<dyn Stream>, BuildError> {
    match choice {
        StreamChoice::ArffFile(p) => {
            let s = ArffFileStream::try_from(p)?;
            Ok(Box::new(s))
        }
        StreamChoice::SeaGenerator(p) => {
            let s = SeaGenerator::try_from(p)?;
            Ok(Box::new(s))
        }
        StreamChoice::AgrawalGenerator(p) => {
            let s = AgrawalGenerator::try_from(p)?;
            Ok(Box::new(s))
        }
        StreamChoice::AssetNegotiationGenerator(p) => {
            let s = AssetNegotiationGenerator::try_from(p)?;
            Ok(Box::new(s))
        }
    }
}
