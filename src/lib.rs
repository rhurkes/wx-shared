#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;

use bincode::{deserialize, serialize};
use serde::Serialize;
use slog::Drain;
use std::boxed::Box;
use std::ops::Deref;
use zmq::{Context, Message, Socket};

pub mod util;

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub data: String,
    pub event_ts: u64,
    pub expires_ts: u64,
    pub ingest_ts: u64,
    pub loc: (f32, f32),
    pub poly: Vec<(f32, f32)>,
    pub source: String,
}

#[derive(Debug)]
pub struct WxError {
    pub message: String
}

impl WxError {
    pub fn new(msg: &str) -> WxError {
        WxError{message: msg.to_string()}
    }
}

impl std::fmt::Display for WxError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for WxError {
    fn description(&self) -> &str {
        self.message.as_str()
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Rocks(rocksdb::Error),
    Serde(serde_json::error::Error),
    Utf8(std::str::Utf8Error),
    Bincode(bincode::ErrorKind),
    ZeroMQ(zmq::Error),
    Wx(WxError),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<rocksdb::Error> for Error {
    fn from(err: rocksdb::Error) -> Error {
        Error::Rocks(err)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Error {
        Error::Serde(err)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Error {
        Error::Utf8(err)
    }
}

impl From<WxError> for Error {
    fn from(err: WxError) -> Error {
        Error::Wx(err)
    }
}

impl From<bincode::ErrorKind> for Error {
    fn from (err: bincode::ErrorKind) -> Error {
        Error::Bincode(err)
    }
}

impl From<zmq::Error> for Error {
    fn from (err: zmq::Error) -> Error {
        Error::ZeroMQ(err)
    }
}

impl<T> From<Box<T>> for Error where Error: From<T> {
    fn from (err: Box<T>) -> Error {
        Error::from(*err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "io error: {}", err),
            Error::Rocks(ref err) => write!(f, "rocksdb error: {}", err),
            Error::Serde(ref err) => write!(f, "serde error: {}", err),
            Error::Utf8(ref err) => write!(f, "utf8 error: {}", err),
            Error::Bincode(ref err) => write!(f, "bincode error: {}", err),
            Error::ZeroMQ(ref err) => write!(f, "zmq error: {}", err),
            Error::Wx(ref err) => write!(f, "wx error: {}", err),
        }
    }
}

pub struct Logger {
    pub instance: slog::Logger,
}

impl Logger {
    pub fn new(app_name: &'static str) -> Logger {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();

        Logger {
            instance: slog::Logger::root(drain, o!("app" => app_name)),
        }
    }
}

impl Deref for Logger {
    type Target = slog::Logger;
    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

pub enum StoreCommand {
    Put,
    Get,
    PutEvent,
    GetEvents,
}

impl StoreCommand {
    fn value(&self) -> u8 {
        match *self {
            StoreCommand::Put => 0,
            StoreCommand::Get => 1,
            StoreCommand::PutEvent => 2,
            StoreCommand::GetEvents => 3,
        }
    }
}

pub enum StoreStatus {
    OkByte,
    ErrorByte,
}

impl StoreStatus {
    fn from(value: u8) -> Option<StoreStatus> {
        match value {
            1 => Some(StoreStatus::ErrorByte),
            0 => Some(StoreStatus::OkByte),
            _ => None,
        }
    }

    fn value(&self) -> u8 {
        match *self {
            StoreStatus::OkByte => 0,
            StoreStatus::ErrorByte => 1,
        }
    }
}

pub struct StoreClient {
    socket: Socket,
}

impl StoreClient {
    pub fn new() -> StoreClient {
        let ctx = Context::new();
        let addr = "tcp://127.0.0.1:31337";
        let socket = ctx.socket(zmq::REQ).unwrap();
        socket.connect(addr).unwrap();

        StoreClient {
            socket,
        }
    }

    fn send_command(&self, cmd_type: StoreCommand, payload: &[u8]) -> Result<Vec<u8>, Error> {
        let mut message = vec![cmd_type.value()];
        message.extend_from_slice(&payload);
        let mut response = Message::new();
        self.socket.send(&message, 0)?;
        self.socket.recv(&mut response, 0)?;

        if response.len() < 2 {
            return Err(Error::Wx(<WxError>::new("invalid response payload")));
        }

        let status = StoreStatus::from(response[0]);

        match status {
            Some(StoreStatus::OkByte) => {
                Ok(response[1..].to_vec())
            }
            Some(StoreStatus::ErrorByte) => {
                let error_msg: &str = deserialize(&response[1..])?;
                return Err(Error::Wx(<WxError>::new(error_msg)));
            }
            _ => {
                return Err(Error::Wx(<WxError>::new("unknown response payload")));
            }
        }
    }

    pub fn put<T: Serialize>(&self, key: &str, value: T) -> Result<(), Error> {
        let serialized_value = serialize(&value)?;
        let kv: (&str, &[u8]) = (key, &serialized_value);
        let payload = serialize(&kv)?;
        self.send_command(StoreCommand::Put, &payload)?;

        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Vec<u8>, Error> {
        let payload = serialize(&key)?;
        let response = self.send_command(StoreCommand::Get, &payload)?.to_vec();

        Ok(response)
    }

    pub fn put_event(&self, event: Event) -> Result<u64, Error> {
        let payload = serialize(&event)?;
        let response = self.send_command(StoreCommand::PutEvent, &payload)?;
        let ts: u64 = deserialize(&response)?;

        Ok(ts)
    }

    pub fn get_events(&self, ts: u64) -> Result<Vec<Event>, Error> {
        let payload: Vec<u8> = if ts != 0 {
            serialize(&ts)?
        } else {
            Vec::new()
        };
        let results = self.send_command(StoreCommand::GetEvents, &payload)?;
        let mut events: Vec<Vec<u8>> = deserialize(&results).unwrap();
        let events: Vec<Event> = events.iter_mut()
            .map(|x| deserialize(x).unwrap())
            .collect();

        Ok(events)
    }
}
