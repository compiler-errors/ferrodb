#![feature(let_else)]

use std::io::{Read, Write};
use std::str::FromStr;
use std::thread::JoinHandle;

use anyhow::{anyhow, bail, Result};
use ferrodb_protocol::{Ping, Pong, Transport, PROTOCOL_VERSION, PREAMBLE};

pub fn spawn_server_loop(port: u16) -> JoinHandle<Result<()>> {
    todo!()
}

pub fn spawn_server_standalone<C>(conn: C) -> JoinHandle<Result<()>>
where
    C: Read + Write + Send + 'static,
{
    std::thread::spawn(|| server_standalone(conn))
}

fn server_standalone<C>(mut conn: C) -> Result<()>
where
    C: Read + Write,
{
    let hello_line = read_line(&mut conn)?;

    let kind = hello_line
        .trim()
        .strip_prefix(PREAMBLE)
        .ok_or_else(|| anyhow!("Expected `{}` preamble, got: {hello_line}", PREAMBLE))?;
    let transport = Transport::from_str(kind).map_err(anyhow::Error::msg)?;

    let mut stream = transport.stream(&mut conn);

    let ping: Ping = stream.read()?;

    if ping.protocol_version == PROTOCOL_VERSION {
        stream.write(Pong::Ok)?;
    } else {
        stream.write(Pong::WrongProtocol)?;
        bail!(
            "Terminated connection. Client protocol = {}, ours = {}",
            ping.protocol_version,
            PROTOCOL_VERSION
        );
    }

    println!("ok.");
    Ok(())
}

fn read_line<R: Read>(r: &mut R) -> Result<String> {
    let mut s = vec![];

    loop {
        let mut byte = 0;
        r.read_exact(std::slice::from_mut(&mut byte))?;

        if byte == b'\n' {
            break;
        }

        s.push(byte);
    }

    Ok(String::from_utf8(s)?)
}
