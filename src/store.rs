use crate::domain::{Event, FetchFailure, WxApp};
use crate::error::{Error, WxError};
use bincode::{deserialize, serialize};
use serde::Serialize;
use std::collections::HashMap;
use zmq::{Context, Message, Socket};

pub enum Command {
    Put,
    Get,
    PutEvent,
    GetEvents,
    GetAllEvents,
    PutFetchFailure,
    GetFetchFailures,
}

impl Command {
    pub fn from(value: u8) -> Option<Command> {
        match value {
            0 => Some(Command::Put),
            1 => Some(Command::Get),
            2 => Some(Command::PutEvent),
            3 => Some(Command::GetEvents),
            4 => Some(Command::GetAllEvents),
            5 => Some(Command::PutFetchFailure),
            6 => Some(Command::GetFetchFailures),
            _ => None,
        }
    }

    fn value(&self) -> u8 {
        match *self {
            Command::Put => 0,
            Command::Get => 1,
            Command::PutEvent => 2,
            Command::GetEvents => 3,
            Command::GetAllEvents => 4,
            Command::PutFetchFailure => 5,
            Command::GetFetchFailures => 6,
        }
    }
}

pub enum Status {
    OkByte,
    ErrorByte,
}

impl Status {
    pub fn from(value: u8) -> Option<Status> {
        match value {
            1 => Some(Status::ErrorByte),
            0 => Some(Status::OkByte),
            _ => None,
        }
    }

    pub fn value(&self) -> u8 {
        match *self {
            Status::OkByte => 0,
            Status::ErrorByte => 1,
        }
    }
}

pub struct Client {
    socket: Socket,
}

impl Client {
    pub fn new() -> Client {
        let ctx = Context::new();
        let addr = "tcp://127.0.0.1:31337";
        let socket = ctx.socket(zmq::REQ).unwrap();
        socket.set_rcvtimeo(1000).unwrap(); // 1s timeout on recv
        socket.connect(addr).unwrap();

        Client { socket }
    }

    fn send_command(&self, cmd_type: Command, payload: &[u8]) -> Result<Vec<u8>, Error> {
        let mut message = vec![cmd_type.value()];
        message.extend_from_slice(&payload);
        let mut response = Message::new();
        self.socket.send(&message, 0)?;
        self.socket.recv(&mut response, 0)?;

        if response.len() < 2 {
            return Err(Error::Wx(<WxError>::new("invalid response payload")));
        }

        let status = Status::from(response[0]);

        match status {
            Some(Status::OkByte) => Ok(response[1..].to_vec()),
            Some(Status::ErrorByte) => {
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
        self.send_command(Command::Put, &payload)?;

        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Vec<u8>, Error> {
        let payload = serialize(&key)?;
        let response = self.send_command(Command::Get, &payload)?.to_vec();

        Ok(response)
    }

    pub fn put_event(&self, event: &Event) -> Result<u64, Error> {
        let payload = serialize(event)?;
        let response = self.send_command(Command::PutEvent, &payload)?;
        let ts: u64 = deserialize(&response)?;

        Ok(ts)
    }

    pub fn get_events(&self, ts: u64) -> Result<Vec<Event>, Error> {
        let payload: Vec<u8> = if ts != 0 {
            serialize(&ts.to_string())?
        } else {
            Vec::new()
        };
        let results = self.send_command(Command::GetEvents, &payload)?;
        let events: Vec<Event> = deserialize(&results)?;

        Ok(events)
    }

    pub fn get_all_events(&self) -> Result<Vec<Event>, Error> {
        let results = self.send_command(Command::GetAllEvents, &[])?;
        let events: Vec<Event> = deserialize(&results)?;

        Ok(events)
    }

    pub fn put_fetch_failure(&self, failure: &FetchFailure) -> Result<(), Error> {
        let payload = serialize(failure)?;
        self.send_command(Command::PutFetchFailure, &payload)?;

        Ok(())
    }

    pub fn get_fetch_failures(&self) -> Result<HashMap<WxApp, u16>, Error> {
        let results = self.send_command(Command::GetFetchFailures, &[])?;
        let failures: Vec<FetchFailure> = deserialize(&results)?;
        let mut failure_map: HashMap<WxApp, u16> = HashMap::new();

        failures.into_iter().for_each(|x| {
            let counter = failure_map.entry(x.app).or_insert(0);
            *counter += 1;
        });

        Ok(failure_map)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
