use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),

    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
