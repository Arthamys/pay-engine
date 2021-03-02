use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Could not read CSV ({0})")]
    ParseError(#[from] csv::Error),
    #[error("Client does not have sufficient funds available")]
    InssuficientFunds,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Could not deserialize input")]
    DeserializeError,
    #[error("Could not serialize output")]
    SerializeError,
}
