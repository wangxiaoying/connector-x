use thiserror::Error;

#[derive(Error, Debug)]
pub enum OracleSourceError {
    #[error(transparent)]
    ConnectorXError(#[from] crate::errors::ConnectorXError),

    #[error(transparent)]
    OracleError(#[from] r2d2_oracle::oracle::Error),

    #[error(transparent)]
    OracleDbError(#[from] r2d2_oracle::oracle::DbError),

    #[error(transparent)]
    OraclePoolError(#[from] r2d2::Error),

    /// Any other errors that are too trivial to be put here explicitly.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
