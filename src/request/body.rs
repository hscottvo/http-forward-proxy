use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Body {
    content: String,
    left_to_read: usize,
}

impl Body {
    pub const fn new(left_to_read: usize) -> Self {
        Self {
            content: String::new(),
            left_to_read,
        }
    }
    pub fn push(&mut self, content: impl AsRef<str>) -> Result<()> {
        let content_size = content.as_ref().len();
        if content_size > self.left_to_read {
            return Err(BodyError::TooManyBytes {
                left_to_read: self.left_to_read,
                content_size,
            });
        }

        self.content.push_str(content.as_ref());
        self.left_to_read -= content_size;
        Ok(())
    }

    pub const fn left_to_read(&self) -> usize {
        self.left_to_read
    }

    pub const fn is_finished(&self) -> bool {
        self.left_to_read == 0
    }
}

impl Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

#[derive(Error, Debug)]
pub enum BodyError {
    #[error("expected up to {left_to_read} bytes, got {content_size}")]
    TooManyBytes {
        left_to_read: usize,
        content_size: usize,
    },
}

type Result<T> = std::result::Result<T, BodyError>;

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;
    #[test]
    fn push_partial() -> Result<()> {
        let mut body = Body::new(10);

        body.push("hi")?;

        assert_eq!(body.left_to_read, 8);
        assert_eq!(body.content, "hi");

        body.push(" hello")?;

        assert_eq!(body.left_to_read, 2);
        assert_eq!(body.content, "hi hello");

        Ok(())
    }

    #[test]
    fn push_exact() -> Result<()> {
        let mut body = Body::new(5);

        body.push("hello")?;

        assert_eq!(body.left_to_read, 0);
        assert_eq!(body.content, "hello");

        Ok(())
    }

    #[test]
    fn push_too_many() {
        let mut body = Body::new(5);
        let result = body.push("hello there");

        assert_matches!(
            result,
            Err(BodyError::TooManyBytes {
                left_to_read: 5,
                content_size: 11
            })
        );
    }
}
