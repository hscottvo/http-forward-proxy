use std::fmt::Display;
use std::str::FromStr;

use thiserror::Error;

use crate::request::method::{Method, MethodError};
use crate::types::{HttpVersion, HttpVersionError};

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct StartLine {
    method: Method,
    target: String,
    version: HttpVersion,
}
impl StartLine {
    pub fn new(method: Method, target: String, version: HttpVersion) -> Self {
        Self {
            method,
            target,
            version,
        }
    }
}

impl Display for StartLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.method, self.target, self.version)
    }
}

impl FromStr for StartLine {
    type Err = StartLineError;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim_matches(['\r', '\n']).to_owned();
        let mut parts = s.split(' ');

        let method = parts
            .next()
            .ok_or(StartLineError::MissingMethod)?
            .to_owned()
            .parse()?;

        let target = parts
            .next()
            .ok_or(StartLineError::MissingTarget)?
            .to_owned();

        let version = parts
            .next()
            .ok_or(StartLineError::MissingHttpVersion)?
            .parse()?;

        if !parts.next().is_none() {
            return Err(StartLineError::Malformed);
        }
        Ok(StartLine::new(method, target, version))
    }
}

#[derive(Error, Debug)]
pub enum StartLineError {
    #[error("missing method")]
    MissingMethod,
    #[error(transparent)]
    MethodParse(#[from] MethodError),
    #[error("missing target")]
    MissingTarget,
    #[error("missing http version")]
    MissingHttpVersion,
    #[error(transparent)]
    HttpParse(#[from] HttpVersionError),
    #[error("malformed startline")]
    Malformed,
}

type Result<T> = std::result::Result<T, StartLineError>;
