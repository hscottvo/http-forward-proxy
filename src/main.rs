#![cfg_attr(test, feature(hash_map_macro))]
mod body;
mod request;
mod response;
mod types;
use eyre::Result;
use std::{
    io::{Read, Write as _},
    net::{TcpListener, TcpStream},
};
use tracing::{debug, info, instrument, trace};

use crate::request::{Request, parser::RequestParser};

fn forward_request(request: &Request) -> Result<String> {
    debug!(request=%request);
    let host = request.host().to_owned() + ":80";
    let mut stream = TcpStream::connect(host)?;
    let mut buffer = [0u8; 50];
    stream.write_all(request.to_string().as_bytes())?;

    let mut response = String::new();
    while let response_bytes = stream.read(&mut buffer)?
        && response_bytes > 0
    {
        debug!(bytes = response_bytes, "fuck 2");
        let chunk_string = String::from_utf8_lossy(&buffer[..response_bytes]);
        response.push_str(&chunk_string);
    }

    Ok(response)
}

fn forward_response(stream: &mut TcpStream, response: impl Into<String>) -> Result<()> {
    let response = response.into();
    stream.write_all(&response.into_bytes()[..])?;
    Ok(())
}

#[instrument(skip(stream))]
fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut parser = RequestParser::new();
    let mut buf = [1u8; 5000];
    loop {
        trace!("loop");
        let bytes_read = stream.read(&mut buf)?;
        if bytes_read == 0 {
            break;
        }
        let requests = parser.push(&buf[..bytes_read])?;
        debug!(
            ?requests,
            num_requests = requests.len(),
            parse_buf_len = parser.buf_len()
        );
        for request in requests {
            let response = forward_request(&request)?;
            debug!(response=%response, "received response");
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
