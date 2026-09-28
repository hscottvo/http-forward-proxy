use super::method::Method;
use super::types::StartLine;

use super::Request;
use eyre::{OptionExt as _, Result};
use std::collections::VecDeque;
use tracing::{debug, instrument};

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
}

impl RequestParser {
    pub fn new() -> Self {
        Self {
            buf: VecDeque::new(),
            phase: ParsePhase::StartLine,
            startline: None,
        }
    }
    #[instrument(skip(self, buf), fields(phase = ?self.phase))]
    pub fn push(&mut self, buf: &[u8]) -> Result<Option<Request>> {
        for &byte in buf {
            self.buf.push_back(byte);
        }
        if let Some(section) = self.capture_until_crlf() {
            debug!("{section:?}");
            self.parse_section(section)?;
        }

        Ok(None)
    }
    fn parse_section(&mut self, line: String) -> Result<()> {
        match self.phase {
            ParsePhase::StartLine => self.parse_startline(line)?,
            ParsePhase::Headers => todo!(),
            ParsePhase::Finished => todo!(),
        };
        Ok(())
    }
    #[instrument(skip(self))]
    fn parse_startline(&mut self, line: String) -> Result<()> {
        let line = line.trim_matches(['\r', '\n']).to_owned();
        let mut parts = line.split(' ');

        let method = parts
            .next()
            .ok_or_eyre("invalid startline: missing method")?
            .to_owned()
            .parse()?;

        let target = parts
            .next()
            .ok_or_eyre("invalid startline: missing target")?
            .to_owned();

        let version = parts
            .next()
            .ok_or_eyre("invalide startline: missing version")?
            .parse()?;

        self.phase = ParsePhase::Headers;
        self.startline = Some(StartLine::new(method, target, version));

        debug!(startline = ?self.startline, "leksfjskld");

        Ok(())
    }
    fn capture_until_crlf(&mut self) -> Option<String> {
        if let Some(cr_index) = self.buf.iter().position(|&x| x == b'\r')
            && let lf_index = cr_index + 1
            && let Some(&next) = self.buf.get(lf_index)
            && next == b'\n'
        {
            debug!("CRLF found at index {:?} and {:?}", cr_index, cr_index + 1);
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

    use std::io::Read;

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
}
