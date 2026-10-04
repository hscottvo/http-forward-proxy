mod request;
mod response;
mod types;
use eyre::Result;
use std::{
    io::{Read, Write as _},
    net::{TcpListener, TcpStream},
};
use tracing::{debug, info, trace};

use crate::request::{Request, parser::RequestParser};

fn forward_request(request: &Request) -> Result<String> {
    debug!(request=%request);
    let mut stream = TcpStream::connect("httpbin.org:80")?;
    let mut buffer = [0u8; 5000];
    stream.write_all(request.to_string().as_bytes())?;
    let response_bytes = stream.read(&mut buffer)?;

    let response = String::from_utf8_lossy(&buffer[..response_bytes]).to_string();
    trace!(response=%response, response_bytes, "received response");
    Ok(response)
}

fn forward_response(stream: &mut TcpStream, response: impl Into<String>) -> Result<()> {
    let response = response.into();
    stream.write_all(&response.into_bytes()[..])?;
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut parser = RequestParser::new();
    let mut buf = [1u8; 50];
    loop {
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read == 0 {
            break;
        }
        let requests = parser.push(&buf[..bytes_read])?;
        for request in requests {
            let response = forward_request(&request)?;
            forward_response(&mut stream, response)?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let listener = TcpListener::bind("127.0.0.1:8080")?;

    for stream in listener.incoming() {
        info!("received message");
        handle_client(stream?)?;
        // stream.shutdown(Shutdown::Both)?;
    }
    Ok(())
}
