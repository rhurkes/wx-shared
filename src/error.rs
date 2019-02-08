use std::boxed::Box;

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
    ParseInt(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError),
    Chrono(chrono::format::ParseError),
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

impl From<std::num::ParseIntError> for Error {
    fn from (err: std::num::ParseIntError) -> Error {
        Error::ParseInt(err)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from (err: std::num::ParseFloatError) -> Error {
        Error::ParseFloat(err)
    }
}

impl From<chrono::format::ParseError> for Error {
    fn from (err: chrono::format::ParseError) -> Error {
        Error::Chrono(err)
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
            Error::ParseInt(ref err) => write!(f, "parse int error: {}", err),
            Error::ParseFloat(ref err) => write!(f, "parse float error: {}", err),
            Error::Chrono(ref err) => write!(f, "chrono error: {}", err),
        }
    }
}
