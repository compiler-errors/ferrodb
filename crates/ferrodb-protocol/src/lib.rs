#![feature(derive_default_enum)]

mod transport;

use serde::{Deserialize, Serialize};

pub use self::transport::{Stream, Transport};

pub const DEFAULT_PORT: &'static str = "1337";
pub const PROTOCOL_VERSION: usize = 0;

#[derive(Debug, Serialize, Deserialize)]
pub struct Ping {
    pub protocol_version: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Pong {
    Ok,
    WrongProtocol,
}

enum Command {
    Select(String),
    Goodbye,
}

enum SelectResponse {
    SomeRows(String),
    Error(String),
    Done,
}
