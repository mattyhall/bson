#![feature(core, io, collections)]
mod error;

use std::collections::HashMap;
use std::old_io::{Writer, MemWriter, MemReader, IoResult};
use std::str::{from_utf8};
use std::num::FromPrimitive;

pub use error::*;

#[derive(Debug, PartialEq)]
pub enum BsonValue {
    Double(f64),
    String(String),
    Doc(Document),
    Bool(bool),
    UTCDatetime(i64),
    Null,
    Regex {pat: String, opts: String},
    Int32(i32),
    Int64(i64),
}

#[derive(Debug, PartialEq, FromPrimitive)]
enum BsonCode {
    Double = 0x01,
    String = 0x02,
    Doc = 0x03,
    Bool = 0x08,
    UTCDatetime = 0x09,
    Null = 0x0A,
    Regex = 0x0B,
    Int32 = 0x10,
    Int64 = 0x12,
}

pub trait ToBson {
    fn to_bson(self) -> BsonValue;
}

impl ToBson for i32 {
    fn to_bson(self) -> BsonValue { BsonValue::Int32(self) }
}

impl ToBson for i64 {
    fn to_bson(self) -> BsonValue { BsonValue::Int64(self) }
}

impl ToBson for f64 {
    fn to_bson(self) -> BsonValue { BsonValue::Double(self) }
}

impl ToBson for String {
    fn to_bson(self) -> BsonValue { BsonValue::String(self) }
}

impl ToBson for Document {
    fn to_bson(self) -> BsonValue { BsonValue::Doc(self) }
}

impl ToBson for bool {
    fn to_bson(self) -> BsonValue { BsonValue::Bool(self) }
}

impl ToBson for BsonValue {
    fn to_bson(self) -> BsonValue { self }
}


fn write_cstring<W: Writer>(w: &mut W, s: &str) -> IoResult<()> {
    try!(w.write_str(s));
    w.write_u8(0x0)
}

fn read_cstring<R: Reader>(r: &mut R) -> Result<String, BsonError> {
    let mut bytes = Vec::new();
    loop {
        let b = try!(r.read_u8());
        if b == 0x00 {
            return Ok(try!(from_utf8(&bytes[])).to_string());
        }
        bytes.push(b);
    }
}

#[derive(Debug, PartialEq)]
pub struct Document {
    hm: HashMap<String, BsonValue> 
}

impl Document {
    pub fn new() -> Document {
        Document {hm: HashMap::new()}
    }

    pub fn insert<V: ToBson>(&mut self, key: &str, val: V) {
       self.hm.insert(key.to_string(), val.to_bson());
    }

    pub fn size(&self) -> i32 {
        // 4 bytes for the size. 1 byte for the NULL
        let mut size = 4 + 1;
        for (key, val) in self.hm.iter() {
            // One byte to specify the type of value
            size += 1;
            // An extra byte for the NULL
            size += key.len() as i32 + 1;
            size += match *val {
                BsonValue::Double(_) => 8,
                // 4 bytes for the length, one for the NULL
                BsonValue::String(ref s) => 4 + s.len() as i32 + 1,
                BsonValue::Doc(ref d) => d.size(),
                BsonValue::Bool(_) => 1,
                BsonValue::UTCDatetime(_) => 8,
                BsonValue::Null => 0,
                BsonValue::Regex{ref pat, ref opts} =>
                    pat.len() as i32 + opts.len() as i32 + 2,
                BsonValue::Int32(_) => 4,
                BsonValue::Int64(_) => 8,
            }
        }
        size
    }
 
    pub fn write<W: Writer>(&self, w: &mut W) -> IoResult<()> {
        try!(w.write_le_i32(self.size()));
        for (key, val) in self.hm.iter() {
            match *val {
                BsonValue::Double(v) => {
                    try!(w.write_u8(BsonCode::Double as u8));
                    try!(write_cstring(w, &key[]));
                    try!(w.write_le_f64(v));
                }
                BsonValue::String(ref v) => {
                    try!(w.write_u8(BsonCode::String as u8));
                    try!(write_cstring(w, &key[]));
                    // Add one for the null byte
                    try!(w.write_le_i32(v.len() as i32 + 1));
                    try!(w.write_str(&v[]));
                    try!(w.write_u8(0x0));
                }
                BsonValue::Doc(ref d) => {
                    try!(w.write_u8(BsonCode::Doc as u8));
                    try!(write_cstring(w, &key[]));
                    try!(d.write(w));
                }
                BsonValue::Bool(b) => {
                    try!(w.write_u8(BsonCode::Bool as u8));
                    try!(write_cstring(w, &key[]));
                    if b {
                        try!(w.write_u8(0x01));
                    } else {
                        try!(w.write_u8(0x00));
                    }
                },
                BsonValue::UTCDatetime(v) => {
                    try!(w.write_u8(BsonCode::UTCDatetime as u8));
                    try!(write_cstring(w, &key[]));
                    try!(w.write_le_i64(v));
                },
                BsonValue::Null => {
                    try!(w.write_u8(BsonCode::Null as u8));
                    try!(write_cstring(w, &key[]));
                }
                BsonValue::Regex {ref pat, ref opts} => {
                    try!(w.write_u8(BsonCode::Regex as u8));
                    try!(write_cstring(w, &key[]));
                    try!(write_cstring(w, &pat[]));
                    try!(write_cstring(w, &opts[]));
                },
                BsonValue::Int32(v) => {
                    try!(w.write_u8(BsonCode::Int32 as u8));
                    try!(write_cstring(w, &key[]));
                    try!(w.write_le_i32(v));
                },
                BsonValue::Int64(v) => {
                    try!(w.write_u8(BsonCode::Int64 as u8));
                    try!(write_cstring(w, &key[]));
                    try!(w.write_le_i64(v));
                },
            }
        }
        w.write_u8(0x00)
    }

    pub fn to_bytes(&self) -> IoResult<Vec<u8>> {
        let mut writer = MemWriter::new();
        try!(self.write(&mut writer));
        Ok(writer.into_inner())
    }

    pub fn read<R: Reader>(r: &mut R) -> Result<Document, BsonError> {
        let doc_len = try!(r.read_le_i32());
        let mut doc = Document::new();

        let t = try!(r.read_u8());
        let key = try!(read_cstring(r));
        let key = &key[];
        let err = BsonError::new(ErrorKind::UnrecognisedCode,
                                 Some(format!("{} is an unrecognised", t)));
        let code = try!(FromPrimitive::from_int(t as isize).ok_or(err.clone()));
        let val = match code {
            BsonCode::Double => try!(r.read_le_f64()).to_bson(),
            BsonCode::String => {
                let l = try!(r.read_le_i32());
                let val = try!(read_cstring(r));
                if l != val.len() as i32 + 1 {
                    return Err(BsonError::new(
                        ErrorKind::IncorrectLength,
                        Some("A string was the incorrect length".to_string())
                    ));
                }
                val.to_bson()
            },
            BsonCode::Doc => try!(Document::read(r)).to_bson(),
            _ => return Err(err)
        };
        doc.insert(key, val);
        if doc.size() != doc_len {
            Err(BsonError::new(
                ErrorKind::IncorrectLength,
                Some("The document was not the correct length".to_string())))
        } else {
            Ok(doc)
        }
    }

    pub fn from_bytes(b: &[u8]) -> Result<Document, BsonError> {
        let mut v = Vec::new();
        v.push_all(b);
        let mut r = MemReader::new(v);
        Document::read(&mut r)
    }
}
