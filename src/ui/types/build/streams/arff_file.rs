use crate::streams::arff::ArffFileStream;
use crate::ui::types::build::BuildError;
use crate::ui::types::choices::ArffParameters;

impl TryFrom<ArffParameters> for ArffFileStream {
    type Error = BuildError;

    fn try_from(p: ArffParameters) -> Result<Self, Self::Error> {
        ArffFileStream::new(p.path, p.class_index).map_err(BuildError::from)
    }
}
