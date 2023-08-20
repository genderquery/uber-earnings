//! # Session file
//!
//! Create a file called `.uber_earnings_session` in your home directory or
//! `uber_earnings_session` in your config directory with the two following
//! cookies from your browser:
//!
//! ```plain
//! sid=YOUR_SID
//! scid=YOUR_CSID
//! ```
//!

use std::{fs, io, path::Path, str::FromStr};

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use dirs::{config_dir, home_dir};
use thiserror::Error;

pub type UberResult<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to find a session file in expected locations")]
    SessionFileNotFound,
    #[error("Unable to read session id from path '{path}': {source}")]
    SessionFileReadError {
        path: String,
        #[source]
        source: io::Error,
    },
}

pub fn read_session_from_file(path: impl AsRef<Path>) -> UberResult<String> {
    let path = path.as_ref();
    fs::read_to_string(path)
        .map(|s| {
            let lines: Vec<_> = s.trim().lines().collect();
            lines.as_slice().join(";")
        })
        .map_err(|source| Error::SessionFileReadError {
            path: path.to_string_lossy().to_string(),
            source,
        })
}

pub fn read_session_from_config_file() -> UberResult<String> {
    let path = if let Some(home_path) = home_dir()
        .map(|dir| dir.join(".uber_earnings_session"))
        .filter(|file| file.exists())
    {
        home_path
    } else if let Some(config_path) = config_dir()
        .map(|dir| dir.join("uber_earnings_session"))
        .filter(|file| file.exists())
    {
        config_path
    } else {
        return Err(Error::SessionFileNotFound);
    };
    read_session_from_file(path)
}

pub struct Record {
    title: String,
    date: NaiveDate,
    time: NaiveTime,
}

pub mod serde;
