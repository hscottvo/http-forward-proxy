use std::fmt::Display;
use std::str::FromStr;

use thiserror::Error;

use crate::request::method::{Method, MethodError};
use crate::request::target::{Target, TargetError};
use crate::types::{HttpVersion, HttpVersionError};

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct StartLine {
    method: Method,
    target: Target,
    version: HttpVersion,
}
impl StartLine {
    const fn new(method: Method, target: Target, version: HttpVersion) -> Self {
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
        let mut parts = s.split(' ');

        let method = parts
            .next()
            .filter(|&s| !s.is_empty())
            .ok_or(StartLineError::MissingMethod)?
            .to_owned()
            .parse()?;

        let target = parts
            .next()
            .filter(|&s| !s.is_empty())
            .ok_or(StartLineError::MissingTarget)?
            .to_owned()
            .parse()?;

        let version = parts
            .next()
            .filter(|&s| !s.is_empty())
            .ok_or(StartLineError::MissingHttpVersion)?
            .parse()?;

        if parts.next().is_some() {
            return Err(StartLineError::Malformed);
        }
        Ok(Self::new(method, target, version))
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
    #[error(transparent)]
    TargetParse(#[from] TargetError),
    #[error("missing http version")]
    MissingHttpVersion,
    #[error(transparent)]
    HttpParse(#[from] HttpVersionError),
    #[error("malformed startline")]
    Malformed,
}

type Result<T> = std::result::Result<T, StartLineError>;

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn happy_path() -> Result<()> {
        let startline: StartLine = "GET https://httpbin.org HTTP/1.1".parse()?;

        assert_eq!(
            startline,
            StartLine {
                method: Method::Get,
                target: "https://httpbin.org".parse()?,
                version: HttpVersion::Http1_1
            }
        );
        Ok(())
    }

    #[test]
    fn empty_string_returns_missing_error() {
        let startline: Result<StartLine> = "".parse();
        assert_matches!(startline, Err(StartLineError::MissingMethod));
    }

    #[test]
    fn missing_target_returns_error() {
        let startline: Result<StartLine> = "GET".parse();
        assert_matches!(startline, Err(StartLineError::MissingTarget));

        let startline: Result<StartLine> = "GET ".parse();
        assert_matches!(startline, Err(StartLineError::MissingTarget));
    }

    #[test]
    fn missing_http_version_returns_error() {
        let startline: Result<StartLine> = "GET https://httpbin.org".parse();
        assert_matches!(startline, Err(StartLineError::MissingHttpVersion));

        let startline: Result<StartLine> = "GET https://httpbin.org ".parse();
        assert_matches!(startline, Err(StartLineError::MissingHttpVersion));
    }

    #[test]
    fn bad_method_returns_error() {
        let result: Result<StartLine> = "get / HTTP/1.1".parse();
        assert_matches!(
            result,
            Err(StartLineError::MethodParse(MethodError::NotSupported(_)))
        );
    }

    #[test]
    fn bad_target_parse_returns_error() {
        let result: Result<StartLine> = "GET hello HTTP/2".parse();
        assert_matches!(result, Err(StartLineError::TargetParse(TargetError::Parse)));
    }

    #[test]
    fn unsupported_version_returns_error() {
        let result: Result<StartLine> = "GET / HTTP/2".parse();
        assert_matches!(
            result,
            Err(StartLineError::HttpParse(
                HttpVersionError::VersionNotSupported
            ))
        );
    }

    #[test]
    fn extra_token_returns_error() {
        let result: Result<StartLine> = "GET / HTTP/1.1 test".parse();
        assert_matches!(result, Err(StartLineError::Malformed));
    }

    #[test]
    fn startline_round_trip() -> Result<()> {
        let startline = StartLine::new("GET".parse()?, "/".parse()?, "HTTP/1.1".parse()?);
        let startline_string = startline.to_string();
        assert_eq!(startline_string.parse::<StartLine>()?, startline);
        Ok(())
    }
}
