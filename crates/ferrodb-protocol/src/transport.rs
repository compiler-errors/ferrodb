use std::fmt::Display;
use std::io::{Read, Write};
use std::str::FromStr;

use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Copy, Clone, Default, Debug)]
pub enum Transport {
    #[default]
    Json,
    Bincode,
    Ron,
}

impl Transport {
    pub fn stream<'c, C>(self, conn: C) -> Stream<C> {
        Stream {
            transport: self,
            conn,
        }
    }
}

impl Display for Transport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Transport::Json => write!(f, "json"),
            Transport::Bincode => write!(f, "bincode"),
            Transport::Ron => write!(f, "ron"),
        }
    }
}

impl FromStr for Transport {
    type Err = String;

    fn from_str(s: &str) -> Result<Transport, String> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Transport::Json),
            "bincode" => Ok(Transport::Bincode),
            "ron" => Ok(Transport::Ron),
            _ => Err(format!(
                "Unknown transport `{s}`. Expected one of: `json`, `bincode`, or `ron`."
            )),
        }
    }
}

pub struct Stream<C> {
    transport: Transport,
    conn: C,
}

impl<C> Stream<C>
where
    C: Read + Write,
{
    pub fn read<V: DeserializeOwned>(&mut self) -> Result<V> {
        Ok(match &self.transport {
            Transport::Json =>
                V::deserialize(&mut serde_json::Deserializer::from_reader(&mut self.conn))?,
            Transport::Bincode => bincode::deserialize_from(&mut self.conn)?,
            Transport::Ron => {
                let mut temp = String::new();
                self.conn.read_to_string(&mut temp)?;
                ron::de::from_str(&temp)?
            },
        })
    }

    pub fn write<V: Serialize>(&mut self, value: V) -> Result<()> {
        match &self.transport {
            Transport::Json => {
                serde_json::to_writer(&mut self.conn, &value)?;
            },
            Transport::Bincode => {
                bincode::serialize_into(&mut self.conn, &value)?;
            },
            Transport::Ron => {
                let temp = ron::to_string(&value)?;
                writeln!(self.conn, "{temp}")?;
            },
        }

        Ok(())
    }
}
