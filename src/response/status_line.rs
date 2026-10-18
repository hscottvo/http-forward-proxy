use std::{fmt::Display, str::FromStr};

use thiserror::Error;

use crate::{
    response::status_code::{StatusCode, StatusCodeError},
    types::{HttpVersion, HttpVersionError},
};

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct StatusLine {
    version: HttpVersion,
    status: StatusCode,
    reason_phrase: String,
}

impl StatusLine {
    pub const fn new(version: HttpVersion, status: StatusCode, reason_phrase: String) -> Self {
        Self {
            version,
            status,
            reason_phrase,
        }
    }
}

impl Display for StatusLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.version, self.status, self.reason_phrase)
    }
}

impl FromStr for StatusLine {
    type Err = StatusLineError;

    fn from_str(s: &str) -> Result<Self> {
        // Err(StatusLineError::Malformed)
        let mut parts = s.split(' ');
        let version = parts
            .next()
            .filter(|&s| !s.is_empty())
            .ok_or(StatusLineError::MissingHttpVersion)?
            .parse()?;

        let status = parts
            .next()
            .filter(|&s| !s.is_empty())
            .ok_or(StatusLineError::MissingStatus)?
            .parse()?;

        let reason_phrase = parts
            .next()
            .ok_or(StatusLineError::MissingReasonPhrase)?
            .to_owned();
        if parts.next().is_some() {
            return Err(StatusLineError::Malformed);
        }
        Ok(Self::new(version, status, reason_phrase))
    }
}

#[derive(Error, Debug)]
pub enum StatusLineError {
    #[error("missing http version")]
    MissingHttpVersion,
    #[error(transparent)]
    HttpParse(#[from] HttpVersionError),
    #[error("missing status")]
    MissingStatus,
    #[error(transparent)]
    StatusParse(#[from] StatusCodeError),
    #[error("missing reason phrase")]
    MissingReasonPhrase,
    #[error("malformed statusline")]
    Malformed,
}

type Result<T> = std::result::Result<T, StatusLineError>;

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn parse_status_line() -> Result<()> {
        let string = "HTTP/1.1 201 Created";
        let status_line: StatusLine = string.parse()?;
        let expected = StatusLine {
            version: HttpVersion::Http1_1,
            status: StatusCode::Created,
            reason_phrase: "Created".to_owned(),
        };

        assert_eq!(expected, status_line);

        Ok(())
    }

    #[test]
    fn parse_status_line_without_reason_phrase() -> Result<()> {
        let string = "HTTP/1.1 201 ";
        let status_line: StatusLine = string.parse()?;
        let expected = StatusLine {
            version: HttpVersion::Http1_1,
            status: StatusCode::Created,
            reason_phrase: String::new(),
        };

        assert_eq!(expected, status_line);

        Ok(())
    }

    #[test]
    fn missing_version() {
        let string = "";
        let result: Result<StatusLine> = string.parse();
        assert_matches!(result, Err(StatusLineError::MissingHttpVersion));
    }

    #[test]
    fn bad_version() {
        let string = "bad_version ";
        let result: Result<StatusLine> = string.parse();
        assert_matches!(result, Err(StatusLineError::HttpParse(_)));
    }

    #[test]
    fn missing_status() {
        let string = "HTTP/1.1";
        let result: Result<StatusLine> = string.parse();
        assert_matches!(result, Err(StatusLineError::MissingStatus));
    }

    #[test]
    fn bad_status() {
        let string = "HTTP/1.1 hi ";
        let result: Result<StatusLine> = string.parse();
        assert_matches!(result, Err(StatusLineError::StatusParse(_)));
    }

    #[test]
    fn missing_reason_phrase() {
        let string = "HTTP/1.1 200";
        let result: Result<StatusLine> = string.parse();
        assert_matches!(result, Err(StatusLineError::MissingReasonPhrase));
    }
}
