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
    #[error("Client {0} tried to modify foreign transaction {1}")]
    AccessViolation(u16, u32),
    #[error("Tried to dispute order {0} more than once")]
    MultipleDispute(u32),
    #[error("Client {0} tried to resolve tx {1} which was not under dispute")]
    ResolveUndisputed(u16, u32),
    #[error("Client {0} tried to chargeback tx {1} which was not under dispute")]
    ChargebackUndisputed(u16, u32),
}
