use std::{fmt::Display, str::FromStr};

use thiserror::Error;

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum Target {
    Absolute(String),
    Origin(String),
    Authority(String),
    Asterisk,
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Absolute(s) | Self::Origin(s) | Self::Authority(s) => {
                write!(f, "{s}")
            }
            Self::Asterisk => write!(f, "*"),
        }
    }
}

impl FromStr for Target {
    type Err = TargetError;

    fn from_str(s: &str) -> Result<Self> {
        if s == "*" {
            return Ok(Self::Asterisk);
        }
        if s.starts_with('/') {
            return Ok(Self::Origin(s.to_owned()));
        }
        if s.contains("://") {
            return Ok(Self::Absolute(s.to_owned()));
        }
        if s.contains(':') {
            return Ok(Self::Authority(s.to_owned()));
        }
        Err(TargetError::Parse)
    }
}

#[derive(Error, Debug)]
pub enum TargetError {
    #[error("failed to parse target")]
    Parse,
}
type Result<T> = std::result::Result<T, TargetError>;
