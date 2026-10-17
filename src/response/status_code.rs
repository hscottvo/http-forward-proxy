use std::{fmt::Display, num::ParseIntError, str::FromStr};

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
    #[error("status code must consist of only digits")]
    NonDecimal(#[from] ParseIntError),
    #[error("status code must be 3 digits, got {0}")]
    WrongLength(usize),
    #[error("status code not supported: {0}")]
    NotSupported(usize),
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
        let length = s.len();
        if length != 3 {
            return Err(StatusCodeError::WrongLength(length));
        }

        let decimal = s.parse::<usize>()?;

        match decimal {
            200 => Ok(Self::Ok),
            201 => Ok(Self::Created),
            414 => Ok(Self::RequestUriTooLong),
            501 => Ok(Self::NotImplemented),
            other => Err(StatusCodeError::NotSupported(other)),
        }
    }
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", u16::from(*self))
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn parse_status_code() -> Result<()> {
        let status: StatusCode = "200".parse()?;

        assert_eq!(status, StatusCode::Ok);

        Ok(())
    }

    #[test]
    fn non_decimal() {
        let result: Result<StatusCode> = "abc".parse();

        assert_matches!(result, Err(StatusCodeError::NonDecimal(_)));
    }

    #[test]
    fn too_short() {
        let result: Result<StatusCode> = "2".parse();
        assert_matches!(result, Err(StatusCodeError::WrongLength(1)));
    }

    #[test]
    fn too_long() {
        let result: Result<StatusCode> = "1234".parse();
        assert_matches!(result, Err(StatusCodeError::WrongLength(4)));
    }
}
