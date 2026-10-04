use crate::request::{body::Body, startline::StartLine};

use super::Request;
use eyre::{OptionExt, Result, bail};
use std::collections::{HashMap, VecDeque};
use tracing::{instrument, trace, warn};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ParsePhase {
    StartLine,
    Headers,
    Body,
    Finished,
}
#[derive(Clone, Debug)]
pub struct RequestParser {
    buf: VecDeque<u8>,
    phase: ParsePhase,
    startline: Option<StartLine>,
    headers: HashMap<String, String>,
    body: Option<Body>,
}

impl RequestParser {
    pub fn new() -> Self {
        Self {
            buf: VecDeque::new(),
            phase: ParsePhase::StartLine,
            startline: None,
            headers: HashMap::new(),
            body: None,
        }
    }
    #[instrument(skip(self, buf), fields(phase = ?self.phase))]
    pub fn push(&mut self, buf: &[u8]) -> Result<Vec<Request>> {
        for &byte in buf {
            self.buf.push_back(byte);
        }
        while let Some(section) = self.capture_until_crlf() {
            trace!("{section:?}");
            self.parse_section(&section)?;
        }

        if self.phase == ParsePhase::Body
            && let Some(section) = self.capture_all()
        {
            self.parse_body(&section)?;
        }
        if self.phase == ParsePhase::Finished {
            let request = Request::try_new(
                self.startline.clone().ok_or_eyre("missing startline")?,
                self.headers.clone(),
                self.body.clone(),
            )?;
            self.reset();
            return Ok(vec![request]);
        }
        Ok(vec![])
    }
    #[instrument(skip(self))]
    fn parse_section(&mut self, line: &str) -> Result<()> {
        match self.phase {
            ParsePhase::StartLine => self.parse_startline(line)?,
            ParsePhase::Headers => self.parse_header(line)?,
            ParsePhase::Body => bail!("tried parsing to crlf for body"),
            ParsePhase::Finished => {}
        }
        Ok(())
    }
    #[instrument(skip(self))]
    fn parse_startline(&mut self, line: &str) -> Result<()> {
        self.startline = Some(line.parse()?);
        self.phase = ParsePhase::Headers;

        trace!(startline = ?self.startline);

        Ok(())
    }

    #[instrument()]
    fn parse_header(&mut self, line: &str) -> Result<()> {
        let line = line.trim_matches(['\r', '\n']).to_owned();
        if line.is_empty() {
            if contains_body(&self.headers) {
                self.phase = ParsePhase::Body;
            } else {
                self.phase = ParsePhase::Finished;
            }
            return Ok(());
        }
        let mut header = line.split(": ");
        if let Some(field) = header.next()
            && let Some(value) = header.next()
            && header.next().is_none()
        {
            trace!(field=?field, value=?value);
            if let Some(old_value) = self.headers.insert(field.to_owned(), value.to_owned()) {
                warn!("old value {old_value} overwritten for field {field}");
            }
            Ok(())
        } else {
            bail!("failed to parse header field")
        }
    }

    #[instrument(skip(self))]
    fn parse_body(&mut self, content: &str) -> Result<()> {
        if self.body.is_none() {
            let content_length = self
                .headers
                .get("Content-Length")
                .ok_or_eyre("missing Content-Length header")?
                .parse::<usize>()?;
            self.body = Some(Body::new(content_length));
        }
        if let Some(body) = &mut self.body {
            body.push(content)?;
            if body.is_finished() {
                self.phase = ParsePhase::Finished;
            }
        }

        Ok(())
    }

    fn capture_until_crlf(&mut self) -> Option<String> {
        if let Some(cr_index) = self.buf.iter().position(|&x| x == b'\r')
            && let lf_index = cr_index + 1
            && let Some(&next) = self.buf.get(lf_index)
            && next == b'\n'
        {
            let bytes: Vec<u8> = self.buf.drain(..=lf_index).collect();
            let string = String::from_utf8_lossy(&bytes);
            Some(string.into_owned())
        } else {
            None
        }
    }

    fn capture_all(&mut self) -> Option<String> {
        if self.buf.is_empty() {
            None
        } else {
            let bytes: Vec<u8> = self.buf.drain(..).collect();
            Some(String::from_utf8_lossy(&bytes).into_owned())
        }
    }

    fn reset(&mut self) {
        self.phase = ParsePhase::StartLine;
        self.startline = None;
        self.headers = HashMap::new();
        self.body = None;
    }
}

fn contains_body(headers: &HashMap<String, String>) -> bool {
    headers.contains_key("Transfer-Encoding") || headers.contains_key("Content-Length")
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use super::*;

    #[test]
    fn split_by_crlf() -> Result<()> {
        let mut parser = RequestParser::new();

        let section_1 = parser.phase;
        parser.push(b"GET /")?;
        let section_2 = parser.phase;
        assert_eq!(section_2, section_1);

        parser.push(b" HTTP/1.1\r\n")?;
        let section_3 = parser.phase;
        assert_ne!(section_3, section_2);

        Ok(())
    }

    #[test]
    fn parse_startline() -> Result<()> {
        let mut parser = RequestParser::new();
        parser.push(b"GET http://example.com/ HTTP/1.1\r\n")?;
        assert_eq!(parser.phase, ParsePhase::Headers);
        Ok(())
    }

    #[test]
    fn retains_extra_bytes_after_parse() -> Result<()> {
        let mut parser = RequestParser::new();
        parser.push(b"GET http://example.com/ HTTP/1.1\r\n12345")?;
        assert_eq!(parser.buf, b"12345".as_slice());
        Ok(())
    }

    #[test]
    fn skips_section_parsing_for_body() -> Result<()> {
        let mut parser = RequestParser::new();
        parser.push(
            b"GET http://example.com/ HTTP/1.1\r\nHost: example.com\r\nContent-Length: 5\r\n\r\n",
        )?;
        assert_eq!(parser.phase, ParsePhase::Body);
        let request = parser.push(b"hello")?;
        assert_eq!(parser.phase, ParsePhase::Finished);
        assert_eq!(request.len(), 1);
        Ok(())
    }

    #[test]
    fn retains_extra_bytes_after_parse_body() -> Result<()> {
        let mut parser = RequestParser::new();
        parser.push(b"GET http://example.com/ HTTP/1.1\r\nHost: example.com\r\nContent-Length: 5\r\n\r\nhelloGET")?;
        assert_eq!(parser.buf, b"GET".as_slice());
        Ok(())
    }

    #[test]
    fn parse_header() -> Result<()> {
        let mut parser = RequestParser::new();
        parser.phase = ParsePhase::Headers;
        parser.push(b"HOST: example.com\r\n")?;
        let expected_headers = HashMap::from([("HOST".to_owned(), "example.com".to_owned())]);
        assert_eq!(parser.headers, expected_headers);

        Ok(())
    }
}
