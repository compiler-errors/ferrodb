use std::net::{TcpStream, ToSocketAddrs};

use anyhow::{Context, Result};
use ferrodb_client::spawn_client;
use ferrodb_protocol::{Transport, DEFAULT_PORT};
use ferrodb_server::{spawn_server_loop, spawn_server_standalone};
use ferrodb_util::read_write;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(about = "A Rusty NanoDB", author = "compiler-errors")]
enum Args {
    /// Run ferrodb in client mode, connecting to a server.
    Client {
        #[structopt(default_value = "::1")]
        /// Hostname of the ferrodb server
        hostname: String,
        #[structopt(short, long, default_value = DEFAULT_PORT)]
        /// Port of the ferrodb server
        port: u16,
        #[structopt(short, long, default_value)]
        /// Serialization format of messages passed between server and client.
        /// Currently supported: `json`, `bincode`, or `ron`.
        transport: Transport,
    },
    /// Run ferrodb in server mode, listening for connections.
    Server {
        #[structopt(short, long, default_value = DEFAULT_PORT)]
        /// Port that the server should listen on
        port: u16,
    },
    /// Run ferrodb in standalone mode, which will launch a client and server
    /// together.
    Standalone {
        #[structopt(short, long, default_value)]
        /// Serialization format of messages passed between server and client.
        /// Currently supported: `json`, `bincode`, or `ron`.
        transport: Transport,
    },
}

pub fn main() -> ! {
    let code = match go() {
        Ok(()) => 0,
        Err(e) => {
            println!("{e}");
            -1
        },
    };

    std::process::exit(code)
}

fn go() -> Result<()> {
    let cmd = Args::from_args_safe()?;
    println!("Args: {:#?}", cmd);

    match cmd {
        Args::Client {
            hostname,
            port,
            transport,
        } => {
            let addr = (hostname.as_str(), port)
                .to_socket_addrs()
                .with_context(|| {
                    format!(
                        "Error while connecting to server: Couldn't resolve server address \
                         {hostname}:{port}"
                    )
                })?
                .next()
                .unwrap();

            println!("Connecting to: {:?}", addr);

            let conn = TcpStream::connect(addr)?;

            let client = spawn_client(conn, transport);
            client.join().expect("Client panicked")?;
        },
        Args::Server { port } => {
            let server = spawn_server_loop(port);
            server.join().expect("Server panicked")?;
        },
        Args::Standalone { transport } => {
            let (conn1, conn2) = read_write();

            let server = spawn_server_standalone(conn2);
            let client = spawn_client(conn1, transport);

            server.join().expect("Server panicked")?;
            client.join().expect("Client panicked")?;
        },
    }

    Ok(())
}
