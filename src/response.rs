mod parser;
mod status_code;
mod statusline;
use std::{collections::HashMap, fmt::Display, str::FromStr};

use thiserror::Error;

use crate::{body::Body, response::statusline::StatusLine};

#[derive(Debug, PartialEq, Eq)]
pub struct Response {
    status_line: StatusLine,
    headers: HashMap<String, String>,
    host: String,
    body: Option<Body>,
}
impl Response {
    pub fn try_new(
        status_line: StatusLine,
        headers: HashMap<String, String>,
        host: String,
        body: Option<Body>,
    ) -> Result<Self> {
        let request = Self {
            status_line,
            headers,
            host,
            body,
        };
        request.require_host()?;
        Ok(request)
    }

    fn require_host(&self) -> Result<()> {
        match self.headers.get("Host") {
            Some(_) => Ok(()),
            None => Err(ResponseError::MissingHost),
        }
    }

    pub fn host(&self) -> &str {
        &self.host
    }
}

#[derive(Error, Debug)]
pub enum ResponseError {
    #[error("request is missing 'Host' header")]
    MissingHost,
}

type Result<T> = std::result::Result<T, ResponseError>;

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.status_line)?;
        for (field, value) in &self.headers {
            writeln!(f, "{field}: {value}")?;
        }
        writeln!(f)?;
        Ok(())
    }
}
