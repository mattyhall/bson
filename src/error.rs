use std::old_io::{IoError};
use std::str::{Utf8Error};
use std::error::{Error, FromError};
use std::fmt;

#[derive(Debug)]
pub enum ErrorKind {
    StringDecodeError(Utf8Error),
    IoError(IoError),
}

#[derive(Debug)]
pub struct BsonError {
    kind: ErrorKind,
    desc: &'static str,
    detail: Option<String>,
}

impl FromError<IoError> for BsonError {
    fn from_error(e: IoError) -> BsonError {
        BsonError {
            kind: ErrorKind::IoError(e),
            desc: "Reader or writer call failed",
            detail: None
        }
    }
}

impl FromError<Utf8Error> for BsonError {
    fn from_error(e: Utf8Error) -> BsonError {
        BsonError {
            kind: ErrorKind::StringDecodeError(e),
            desc: "String not UTF-8 encoded",
            detail: None
        }
    }
}

impl fmt::Display for BsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description())
    }
}

impl Error for BsonError {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::IoError(ref e) => e.desc,
            _ => self.desc
        }
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
