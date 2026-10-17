use std::{collections::HashMap, fmt::Display};

use thiserror::Error;

use crate::{body::Body, request::startline::StartLine};

mod method;
pub mod parser;
mod startline;
mod target;

#[derive(Debug, PartialEq, Eq)]
pub struct Request {
    startline: StartLine,
    headers: HashMap<String, String>,
    host: String,
    body: Option<Body>,
}
impl Request {
    pub fn try_new(
        startline: StartLine,
        headers: HashMap<String, String>,
        host: String,
        body: Option<Body>,
    ) -> Result<Self> {
        let request = Self {
            startline,
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
            None => Err(RequestError::MissingHost),
        }
    }

    pub fn host(&self) -> &str {
        &self.host
    }
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("request is missing 'Host' header")]
    MissingHost,
}

type Result<T> = std::result::Result<T, RequestError>;

impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.startline)?;
        for (field, value) in &self.headers {
            writeln!(f, "{field}: {value}")?;
        }
        writeln!(f)?;
        if let Some(content) = &self.body {
            writeln!(f, "{content}")?;
        }
        Ok(())
    }
}
