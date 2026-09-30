use std::{fmt::Display, str::FromStr};

use thiserror::Error;

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

impl Display for HttpVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match *self {
            Self::Http1_1 => "HTTP/1.1",
        };
        write!(f, "{string}")
    }
}
