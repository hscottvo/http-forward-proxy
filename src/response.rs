pub enum StatusCode {
    Ok,
    RequestUriTooLong,
    NotImplemented,
}

impl From<StatusCode> for u16 {
    fn from(value: StatusCode) -> Self {
        match value {
            StatusCode::Ok => 200,
            StatusCode::RequestUriTooLong => 414,
            StatusCode::NotImplemented => 501,
        }
    }
}
