use std::ffi::OsString;
use std::io;

use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("(de)Serialization error")]
    Json(#[from] serde_json::Error),
    #[error("File error {0}")]
    Io(#[from] io::Error),
    #[error("Error updating: {0}")]
    Update(#[from] UpdateError),
}

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Bad file name {0:?}")]
    BadFileName(OsString),
    #[error("File error")]
    Io(#[from] io::Error),
    #[error(transparent)]
    Update(#[from] self_update::errors::Error),
}