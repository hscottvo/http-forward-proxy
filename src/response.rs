mod parser;
mod status_code;
mod status_line;
use std::{collections::HashMap, fmt::Display};

use crate::{body::Body, response::status_line::StatusLine};
pub use parser::ResponseParser;

#[derive(Debug, PartialEq, Eq)]
pub struct Response {
    status_line: StatusLine,
    headers: HashMap<String, String>,
    host: String,
    body: Option<Body>,
}
impl Response {
    pub const fn new(
        status_line: StatusLine,
        headers: HashMap<String, String>,
        host: String,
        body: Option<Body>,
    ) -> Self {
        Self {
            status_line,
            headers,
            host,
            body,
        }
    }
}

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
