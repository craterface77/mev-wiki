use thiserror::Error;

#[derive(Debug, Error)]
pub enum SimulateError {
    #[error("EVM reverted: {0}")]
    Revert(String),

    #[error("EVM halted: {0}")]
    Halt(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Simulation unprofitable")]
    Unprofitable,

    #[error("Trap token detected at {0}")]
    TrapToken(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type SimulateResult<T> = Result<T, SimulateError>;
