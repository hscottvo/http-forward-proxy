use std::{collections::HashMap, fmt::Display};

use crate::request::startline::StartLine;

mod method;
pub mod parser;
mod startline;

#[derive(Debug)]
pub struct Request {
    startline: StartLine,
    headers: HashMap<String, String>,
    body: String,
}
impl Request {
    pub fn new(startline: StartLine, headers: HashMap<String, String>, body: String) -> Self {
        Self {
            startline,
            headers,
            body,
        }
    }
}

impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.startline)?;
        for (field, value) in &self.headers {
            writeln!(f, "{}: {}", field, value)?;
        }
        Ok(())
    }
}
