use std::str::FromStr;

use thiserror::Error;

use super::method::Method;

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum RequestSection {
    StartLine(StartLine),
    Headers,
    Finished,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum HttpVersion {
    Http1_1,
}
#[derive(Error, Debug)]
pub enum HttpVersionError {
    #[error("unknown version")]
    UnknownVersion,
    #[error("version not supported")]
    VersionNotSupported,
}

impl FromStr for HttpVersion {
    type Err = HttpVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.1" => Ok(Self::Http1_1),
            "HTTP/2" => Err(HttpVersionError::VersionNotSupported),
            _ => Err(HttpVersionError::UnknownVersion),
        }
    }
}

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
