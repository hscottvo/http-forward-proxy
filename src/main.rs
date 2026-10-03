mod request;
mod response;
mod types;
use eyre::Result;
use std::{
    io::{Read, Write as _},
    net::{Shutdown, TcpListener, TcpStream},
};
use tracing::{debug, info, trace};

use crate::request::parser::RequestParser;

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut parser = RequestParser::new();
    let mut buf = [1u8; 50];
    let request = loop {
        let bytes_read = stream.read(&mut buf)?;
        if let Some(response) = parser.push(&buf[..bytes_read])? {
            break response;
        }
    };
    trace!(request = %request);

    let mut x = TcpStream::connect("httpbin.org:80")?;
    let mut y = [0u8; 5000];
    let a = x.write(&request.to_string().into_bytes())?;
    debug!(bytes=?a, "wrote to httpbin.org");
    let z = x.read(&mut y)?;
    trace!(response=%String::from_utf8_lossy(&y[..]), response_bytes=?z, "received response");
    stream.write_all(&y[..z])?;
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
