// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::HTTP_TIMEOUT;
use logger::*;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::thread::sleep;
use std::time::Duration;

/// This implements a very primitive HTTP/1.0 client which will issue a GET
/// request and return only the body of the response.
pub fn http_get<T: ToSocketAddrs>(address: T, uri: &str) -> Result<String, ()> {
    let request = format!("GET {} HTTP/1.0\r\n\r\n", uri);

    let timeout = Some(Duration::new(0, HTTP_TIMEOUT as u32));

    let mut stream =
        TcpStream::connect(address).map_err(|e| error!("failed to connect: {:?}", e))?;

    stream
        .set_write_timeout(timeout)
        .map_err(|_| error!("failed to set write timeout"))?;
    stream
        .set_read_timeout(timeout)
        .map_err(|_| error!("failed to set read timeout"))?;
    stream
        .write_all(request.as_bytes())
        .map_err(|_| error!("failed to send http get request"))?;
    sleep(timeout.unwrap());

    let mut content = String::new();
    stream
        .read_to_string(&mut content)
        .map_err(|e| error!("error reading: {:?}", e))?;

    let last_line: &str = content.split("\r\n").last().unwrap();
    Ok(last_line.to_owned())
}
