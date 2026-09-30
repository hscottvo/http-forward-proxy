use crate::request::startline::StartLine;

use super::Request;
use eyre::{Result, bail};
use std::collections::{HashMap, VecDeque};
use tracing::{debug, instrument, warn};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ParsePhase {
    StartLine,
    Headers,
    Finished,
}
#[derive(Clone, Debug)]
pub struct RequestParser {
    buf: VecDeque<u8>,
    phase: ParsePhase,
    startline: Option<StartLine>,
    headers: HashMap<String, String>,
}

impl RequestParser {
    pub fn new() -> Self {
        Self {
            buf: VecDeque::new(),
            phase: ParsePhase::StartLine,
            startline: None,
            headers: HashMap::new(),
        }
    }
    #[instrument(skip(self, buf), fields(phase = ?self.phase))]
    pub fn push(&mut self, buf: &[u8]) -> Result<Option<Request>> {
        for &byte in buf {
            self.buf.push_back(byte);
        }
        while let Some(section) = self.capture_until_crlf() {
            debug!("{section:?}");
            self.parse_section(section)?;
        }

        Ok(None)
    }
    fn parse_section(&mut self, line: String) -> Result<()> {
        match self.phase {
            ParsePhase::StartLine => self.parse_startline(line)?,
            ParsePhase::Headers => self.parse_header(line)?,
            ParsePhase::Finished => todo!(),
        };
        Ok(())
    }
    #[instrument(skip(self))]
    fn parse_startline(&mut self, line: String) -> Result<()> {
        self.startline = Some(line.parse()?);
        self.phase = ParsePhase::Headers;

        debug!(startline = ?self.startline);

        Ok(())
    }

    #[instrument(skip(self))]
    fn parse_header(&mut self, line: String) -> Result<()> {
        let line = line.trim_matches(['\r', '\n']).to_owned();
        if line.is_empty() {
            // This should probably be body or something?
            self.phase = ParsePhase::Finished;
            debug!("done parsing headers");
            return Ok(());
        }
        let mut header = line.split(": ");
        if let Some(field) = header.next()
            && let Some(value) = header.next()
            && header.next().is_none()
        {
            debug!(field=?field, value=?value);
            if let Some(old_value) = self.headers.insert(field.to_owned(), value.to_owned()) {
                warn!("old value {old_value} overwritten for field {field}");
            }
            Ok(())
        } else {
            bail!("failed to parse header field")
        }
    }

    fn capture_until_crlf(&mut self) -> Option<String> {
        if let Some(cr_index) = self.buf.iter().position(|&x| x == b'\r')
            && let lf_index = cr_index + 1
            && let Some(&next) = self.buf.get(lf_index)
            && next == b'\n'
        {
            let bytes: Vec<u8> = self.buf.drain(..lf_index + 1).collect();
            let string = String::from_utf8_lossy(&bytes);
            Some(string.into_owned())
        } else {
            None
        }
    }
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
    fn parse_header() -> Result<()> {
        let mut parser = RequestParser::new();
        parser.phase = ParsePhase::Headers;
        parser.push(b"HOST: example.com\r\n")?;
        let expected_headers = HashMap::from([("HOST".to_owned(), "example.com".to_owned())]);
        assert_eq!(parser.headers, expected_headers);

        Ok(())
    }
}
