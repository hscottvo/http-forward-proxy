mod request;
mod response;
mod types;
use eyre::Result;
use std::{
    io::{Read, Write as _},
    net::{Shutdown, TcpListener, TcpStream},
};
use tracing::{debug, info};

use crate::request::parser::RequestParser;

const DUMMY_RESPONSE: &str = r#"HTTP/1.1 200 OK
Content-Type: text/plain
Content-Length: 5

Hello"#;

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut parser = RequestParser::new();
    let mut buf = [1u8; 50];
    let request = loop {
        let bytes_read = stream.read(&mut buf)?;
        if let Some(response) = parser.push(&buf[..bytes_read])? {
            break response;
        }
    };
    debug!(request = %request);
    stream.write_all(DUMMY_RESPONSE.as_bytes())?;
    stream.shutdown(Shutdown::Both)?;
    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        info!("received message");
        handle_client(stream?)?;
    }
    Ok(())
}
