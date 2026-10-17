use std::{fmt::Display, str::FromStr};

use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum StatusCode {
    Ok,
    Created,
    RequestUriTooLong,
    NotImplemented,
}

#[derive(Error, Debug)]
pub enum StatusCodeError {
    #[error("status code not supported: {0}")]
    NotSupported(String),
}

type Result<T> = std::result::Result<T, StatusCodeError>;

impl From<StatusCode> for u16 {
    fn from(value: StatusCode) -> Self {
        match value {
            StatusCode::Ok => 200,
            StatusCode::Created => 201,
            StatusCode::RequestUriTooLong => 414,
            StatusCode::NotImplemented => 501,
        }
    }
}

impl FromStr for StatusCode {
    type Err = StatusCodeError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "200" => Ok(Self::Ok),
            "201" => Ok(Self::Created),
            "414" => Ok(Self::RequestUriTooLong),
            "501" => Ok(Self::NotImplemented),
            other => Err(StatusCodeError::NotSupported(other.to_owned())),
        }
    }
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u16::from(*self))
    }
}
