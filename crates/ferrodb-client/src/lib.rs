use std::io::{Read, Write};
use std::thread::JoinHandle;

use anyhow::Result;
use ferrodb_protocol::{Ping, Pong, Transport, PROTOCOL_VERSION};

pub fn spawn_client<C>(conn: C, transport: Transport) -> JoinHandle<Result<()>>
where
    C: Read + Write + Send + 'static,
{
    std::thread::spawn(move || client(conn, transport))
}

fn client<C>(mut conn: C, transport: Transport) -> Result<()>
where
    C: Read + Write,
{
    write!(conn, "HELLO FERRODB {transport}\n")?;

    let mut stream = transport.stream(conn);

    stream.write(Ping {
        protocol_version: PROTOCOL_VERSION,
    })?;

    let pong: Pong = stream.read()?;

    Ok(())
}
