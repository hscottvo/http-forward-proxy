use crate::{body::Body, response::status_line::StatusLine};

use super::Response;
use eyre::{OptionExt, Result, bail};
use std::collections::{HashMap, VecDeque};
use tracing::{debug, instrument, trace, warn};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
enum ParsePhase {
    StartLine,
    Headers,
    Body,
    Finished,
}
#[derive(Clone, Debug)]
pub struct ResponseParser {
    buf: VecDeque<u8>,
    phase: ParsePhase,
    status_line: Option<StatusLine>,
    headers: HashMap<String, String>,
    body: Option<Body>,
}

impl ResponseParser {
    pub fn new() -> Self {
        Self {
            buf: VecDeque::new(),
            phase: ParsePhase::StartLine,
            status_line: None,
            headers: HashMap::new(),
            body: None,
        }
    }
    #[instrument(skip(self, buf), fields(phase = ?self.phase))]
    pub fn push(&mut self, buf: &[u8]) -> Result<Vec<Response>> {
        let buf_string = String::from_utf8_lossy_owned(buf.to_vec());
        debug!(buffer = buf_string);
        for &byte in buf {
            self.buf.push_back(byte);
        }
        let mut ret = Vec::new();
        while let Some(response) = self.parse()? {
            trace!(response = %response, buffer_length = self.buf_len());
            ret.push(response);
        }
        Ok(ret)
    }

    #[instrument(skip(self))]
    fn parse(&mut self) -> Result<Option<Response>> {
        if self.phase == ParsePhase::StartLine
            && let Some(line) = self.capture_until_crlf()
        {
            self.parse_status_line(line)?;
        }

        while self.phase == ParsePhase::Headers
            && let Some(line) = self.capture_until_crlf()
        {
            self.parse_header(line)?;
        }

        if self.phase == ParsePhase::Body {
            self.parse_body()?;
        }
        if self.phase == ParsePhase::Finished {
            let request = Response::new(
                self.status_line.clone().ok_or_eyre("missing startline")?,
                self.headers.clone(),
                self.headers
                    .get("Host")
                    .ok_or_eyre("missing host header")?
                    .clone(),
                self.body.clone(),
            );
            self.reset();
            return Ok(Some(request));
        }
        Ok(None)
    }

    #[instrument(skip(self, line), fields(line=line.as_ref()))]
    fn parse_status_line(&mut self, line: impl AsRef<str>) -> Result<()> {
        self.status_line = Some(line.as_ref().parse()?);
        self.phase = ParsePhase::Headers;

        trace!(startline = ?self.status_line);

        Ok(())
    }

    #[instrument(skip(self, line), fields(line=line.as_ref()))]
    fn parse_header(&mut self, line: impl AsRef<str>) -> Result<()> {
        let line = line.as_ref();
        if line.is_empty() {
            if contains_body() {
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
                warn!(old_value, "value overwritten for field {field}");
            }
            Ok(())
        } else {
            bail!("failed to parse header field")
        }
    }

    #[instrument(skip(self))]
    fn parse_body(&mut self) -> Result<()> {
        if self.body.is_none() {
            let content_length = self
                .headers
                .get("Content-Length")
                .ok_or_eyre("missing Content-Length header")?
                .parse::<usize>()?;
            self.body = Some(Body::new(content_length));
        }

        let body = self
            .body
            .as_ref()
            .ok_or_eyre("failed to get body field from parser state")?;
        let amount_to_read = body.left_to_read();
        let Some(content) = self.capture_exact(amount_to_read) else {
            return Ok(());
        };
        let body = self
            .body
            .as_mut()
            .ok_or_eyre("failed to get body field from parser state")?;
        body.push(content)?;
        if body.is_finished() {
            self.phase = ParsePhase::Finished;
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
            Some(string.trim_matches(['\r', '\n']).to_owned())
        } else {
            None
        }
    }

    fn capture_exact(&mut self, num_bytes: usize) -> Option<String> {
        if self.buf.is_empty() {
            None
        } else {
            let bytes: Vec<u8> = self.buf.drain(..num_bytes).collect();
            Some(String::from_utf8_lossy(&bytes).into_owned())
        }
    }

    fn reset(&mut self) {
        self.phase = ParsePhase::StartLine;
        self.status_line = None;
        self.headers = HashMap::new();
        self.body = None;
    }
    pub fn buf_len(&self) -> usize {
        self.buf.len()
    }
}

const fn contains_body() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use crate::{response::status_code::StatusCode, types::HttpVersion};

    use super::*;
    #[test]
    fn happy_path() -> Result<()> {
        let mut parser = ResponseParser::new();
        let response_string = r#"HTTP/1.1 201 Created
Content-Type: application/json
Location: http://example.com/users/123

{
  "message": "New user created",
  "user": {
    "id": 123,
    "firstName": "Example",
    "lastName": "Person",
    "email": "bsmth@example.com"
  }
}"#;
        let version = HttpVersion::Http1_1;
        let status = StatusCode::Created;
        let reason_phrase = "Created";
        let status_line = StatusLine::new(version, status, reason_phrase.to_owned());

        let binding = parser.push(response_string.as_bytes())?;
        let response = binding.first().unwrap();

        assert_eq!(response.status_line, status_line);

        Ok(())
    }
}
