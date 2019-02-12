#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;

use self::error::{Error, WxError};
use bincode::{deserialize, serialize};
use serde::Serialize;
use zmq::{Context, Message, Socket};

pub mod error;
pub mod util;

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub data: String,
    pub event_ts: u64,
    pub expires_ts: Option<u64>,
    pub ingest_ts: u64,
    pub loc: Option<Coordinates>,
    pub poly: Option<Vec<Coordinates>>,
    pub src: String,
    pub wfo: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Coordinates {
    pub lat: f32,
    pub lon: f32,
}

pub enum StoreCommand {
    Put,
    Get,
    PutEvent,
    GetEvents,
}

impl StoreCommand {
    pub fn from(value: u8) -> Option<StoreCommand> {
        match value {
            0 => Some(StoreCommand::Put),
            1 => Some(StoreCommand::Get),
            2 => Some(StoreCommand::PutEvent),
            3 => Some(StoreCommand::GetEvents),
            _ => None,
        }
    }

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
    pub fn from(value: u8) -> Option<StoreStatus> {
        match value {
            1 => Some(StoreStatus::ErrorByte),
            0 => Some(StoreStatus::OkByte),
            _ => None,
        }
    }

    pub fn value(&self) -> u8 {
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
        socket.set_rcvtimeo(1).unwrap();    // 1ms timeout on recv
        socket.connect(addr).unwrap();

        StoreClient { socket }
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
            Some(StoreStatus::OkByte) => Ok(response[1..].to_vec()),
            Some(StoreStatus::ErrorByte) => {
                let error_msg: &str = deserialize(&response[1..])?;
                Err(Error::Wx(<WxError>::new(error_msg)))
            }
            _ => Err(Error::Wx(<WxError>::new("unknown response payload"))),
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

    pub fn put_event(&self, event: &Event) -> Result<u64, Error> {
        dbg!(&event);
        let payload = serialize(event)?;
        let response = self.send_command(StoreCommand::PutEvent, &payload)?;
        let ts: u64 = deserialize(&response)?;

        Ok(ts)
    }

    pub fn get_events(&self, ts: u64) -> Result<Vec<Event>, Error> {
        let payload: Vec<u8> = if ts != 0 { serialize(&ts)? } else { Vec::new() };
        let results = self.send_command(StoreCommand::GetEvents, &payload)?;
        let mut events: Vec<Vec<u8>> = deserialize(&results).unwrap();
        let events: Vec<Event> = events.iter_mut().map(|x| deserialize(x).unwrap()).collect();

        Ok(events)
    }
}

impl Default for StoreClient {
    fn default() -> Self {
        Self::new()
    }
}
