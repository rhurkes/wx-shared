#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;

pub mod util;

#[derive(Serialize, Deserialize)]
pub struct EventMessage<'a> {
    pub ingest_ts: u64,
    pub event_ts: u64,
    pub event_type: &'a str,
    pub data: String,
}

#[derive(Debug)]
pub struct OtherError {
    pub message: String
}

impl OtherError {
    pub fn new(msg: &str) -> OtherError {
        OtherError{message: msg.to_string()}
    }
}

impl std::fmt::Display for OtherError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SuperErrorSideKick is here!")    // TODO
    }
}

impl std::error::Error for OtherError {
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
    Other(OtherError),
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

impl From<OtherError> for Error {
    fn from(err: OtherError) -> Error {
        Error::Other(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            Error::Rocks(ref err) => write!(f, "RocksDB error: {}", err),
            Error::Serde(ref err) => write!(f, "Serde error: {}", err),
            Error::Utf8(ref err) => write!(f, "Utf8 error: {}", err),
            Error::Other(ref err) => write!(f, "Other error: {}", err),
        }
    }
}
