use hyper::Error as HyperError;
use std::io::Error as IoError;
use std::error::Error as StdError;
use std::fmt::Display;

/// `Result` alias type.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Error type.
#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Hyper(HyperError),
    Other(&'static str),
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}

impl From<HyperError> for Error {
    fn from(err: HyperError) -> Error {
        Error::Hyper(err)
    }
}

impl From<&'static str> for Error {
    fn from(err: &'static str) -> Error {
        Error::Other(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Error::Io(ref inner) => inner.fmt(f),
            Error::Hyper(ref inner) => inner.fmt(f),
            _ => f.write_str(self.description()),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref inner) => inner.description(),
            Error::Hyper(ref inner) => inner.description(),
            Error::Other(msg) => msg,
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Io(ref inner) => Some(inner),
            Error::Hyper(ref inner) => Some(inner),
            _ => None,
        }
    }
}
