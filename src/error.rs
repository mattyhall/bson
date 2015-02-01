use std::old_io::{IoError};
use std::str::{Utf8Error};
use std::error::{Error, FromError};
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    StringDecodeError(Utf8Error),
    IoError(IoError),
    IncorrectLength,
    UnrecognisedCode,
}

#[derive(Debug, PartialEq)]
pub struct BsonError {
    kind: ErrorKind,
    desc: &'static str,
    detail: Option<String>,
}

impl BsonError {
    pub fn new(k: ErrorKind, detail: Option<String>) -> BsonError {
        let desc = match k {
            ErrorKind::StringDecodeError(_) => "String not UTF-8 encoded",
            ErrorKind::IoError(_) => "Reader or writer call failed",
            ErrorKind::IncorrectLength => "Input was shorter than expected",
            ErrorKind::UnrecognisedCode => "A bson code was not recognised",
        };
        BsonError {
            kind: k,
            desc: desc,
            detail: detail
        }
    } 
}

impl FromError<IoError> for BsonError {
    fn from_error(e: IoError) -> BsonError {
        BsonError::new(ErrorKind::IoError(e), None)
    }
}

impl FromError<Utf8Error> for BsonError {
    fn from_error(e: Utf8Error) -> BsonError {
        BsonError::new(ErrorKind::StringDecodeError(e), None)
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
            _ => self.desc,
        }
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
