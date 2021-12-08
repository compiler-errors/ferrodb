/**!
 * The connection protocol introduction as follows:
 * 
 * 1. Client, over plaintext sends `HELLO! FERRODB {transport}\n`
 * 2. Client, serializes Ping message over {transport} serialization method.
 * 3. Server reads this plaintext message, and then reads serialized Ping 
 *     message using the serialization specified in the first plaintext string.
 * 4. Server verifies the protocol version, and serializes a Pong message.
 * 5. This concludes the introduction, and the server will wait for serialized 
 *     Commands from the client.
 */

#![feature(derive_default_enum)]

mod transport;

use serde::{Deserialize, Serialize};

pub use self::transport::{Stream, Transport};

pub const DEFAULT_PORT: &'static str = "1337";
pub const PROTOCOL_VERSION: usize = 0;
pub const PREAMBLE: &'static str = "HELLO! FERRODB ";

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
