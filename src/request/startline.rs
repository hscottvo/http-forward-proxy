use std::str::FromStr;

use thiserror::Error;

use crate::request::{method::Method, types::HttpVersion};

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

impl FromStr for StartLine {
    type Err = StartLineError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

#[derive(Error, Debug)]
pub enum StartLineError {
    #[error("missing method")]
    MissingMethod,
    #[error("missing target")]
    MissingTarget,
    #[error("missing http version")]
    MissingHttpVersion,
    #[error("malformed startline")]
    Malformed,
}
