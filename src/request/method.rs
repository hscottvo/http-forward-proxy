use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use thiserror::Error;
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Method {
    Get,
    Put,
    Post,
    Delete,
}

#[cfg(test)]
pub type Result<T> = std::result::Result<T, MethodError>;

#[derive(Error, Debug)]
pub enum MethodError {
    #[error("method not support: {0}")]
    NotSupported(String),
}

impl Debug for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
            Self::Put => write!(f, "PUT"),
            Self::Post => write!(f, "POST"),
            Self::Delete => write!(f, "DELETE"),
        }
    }
}

impl FromStr for Method {
    type Err = MethodError;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::Get),
            "PUT" => Ok(Method::Put),
            "POST" => Ok(Method::Post),
            "DELETE" => Ok(Method::Delete),
            _ => Err(MethodError::NotSupported(s.to_owned())),
        }
    }
}

impl From<Method> for String {
    fn from(value: Method) -> Self {
        match value {
            Method::Get => "GET".to_owned(),
            Method::Put => "PUT".to_owned(),
            Method::Post => "POST".to_owned(),
            Method::Delete => "DELETE".to_owned(),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;
    #[test]
    fn method_parse_roundtrip() -> Result<()> {
        let methods = [Method::Get, Method::Put, Method::Post, Method::Delete];
        for method in methods {
            let method_string = method.to_string();
            let parsed_method = method_string.parse()?;
            assert_eq!(method, parsed_method);
        }
        Ok(())
    }

    #[test]
    fn wrong_case_returns_error() {
        assert_matches!("get".parse::<Method>(), Err(MethodError::NotSupported(_)));
    }

    #[test]
    fn empty_string_returns_error() {
        assert_matches!("".parse::<Method>(), Err(MethodError::NotSupported(_)));
    }
}
