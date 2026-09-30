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

    let mut x = TcpStream::connect("www.example.com:80")?;
    let mut y = [0u8; 1000];
    let a = x.write(&request.to_string().into_bytes())?;
    debug!("wrote {:?} bytes to example.com", a);
    let z = x.read(&mut y)?;
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
