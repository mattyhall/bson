#![feature(core, io, collections)]
mod error;

use std::collections::HashMap;
use std::old_io::{Writer, MemWriter, MemReader, IoResult};
use std::str::{from_utf8};

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
                    try!(w.write_u8(0x01));
                    try!(write_cstring(w, &key[]));
                    try!(w.write_le_f64(v));
                }
                BsonValue::String(ref v) => {
                    try!(w.write_u8(0x02));
                    try!(write_cstring(w, &key[]));
                    // Add one for the null byte
                    try!(w.write_le_i32(v.len() as i32 + 1));
                    try!(w.write_str(&v[]));
                    try!(w.write_u8(0x0));
                }
                BsonValue::Doc(ref d) => {
                    try!(w.write_u8(0x03));
                    try!(write_cstring(w, &key[]));
                    try!(d.write(w));
                }
                BsonValue::Bool(b) => {
                    try!(w.write_u8(0x08));
                    try!(write_cstring(w, &key[]));
                    if b {
                        try!(w.write_u8(0x01));
                    } else {
                        try!(w.write_u8(0x00));
                    }
                },
                BsonValue::UTCDatetime(v) => {
                    try!(w.write_u8(0x09));
                    try!(write_cstring(w, &key[]));
                    try!(w.write_le_i64(v));
                },
                BsonValue::Null => {
                    try!(w.write_u8(0x0A));
                    try!(write_cstring(w, &key[]));
                }
                BsonValue::Regex {ref pat, ref opts} => {
                    try!(w.write_u8(0x0B));
                    try!(write_cstring(w, &key[]));
                    try!(write_cstring(w, &pat[]));
                    try!(write_cstring(w, &opts[]));
                },
                BsonValue::Int32(v) => {
                    try!(w.write_u8(0x10));
                    try!(write_cstring(w, &key[]));
                    try!(w.write_le_i32(v));
                },
                BsonValue::Int64(v) => {
                    try!(w.write_u8(0x12));
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
        try!(r.read_le_i32());
        let mut doc = Document::new();

        let t = try!(r.read_u8());
        match t {
            0x01 => {
                let key = try!(read_cstring(r));
                let val = try!(r.read_le_f64());
                doc.insert(&key[], val);
            }
            _ => {}
        }
        Ok(doc)
    }

    pub fn from_bytes(b: &[u8]) -> Result<Document, BsonError> {
        let mut v = Vec::new();
        v.push_all(b);
        let mut r = MemReader::new(v);
        Document::read(&mut r)
    }
}


#[test]
fn test_i64_encode() {
    let mut bson = Document::new();
    bson.insert("int", 3000000000i64);
    assert_eq!(bson.to_bytes(),
               Ok(vec![0x12,0x00,0x00,0x00,0x12,0x69,0x6e,0x74,0x00,0x00,0x5e,
                       0xd0,0xb2,0x00,0x00,0x00,0x00,0x00]));
}

#[test]
fn test_f64_encode() {
    let mut bson = Document::new();
    bson.insert("float", 12.12);
    let bytes = bson.to_bytes();
    assert_eq!(bytes,
               Ok(vec![0x14,0x00,0x00,0x00,0x01,0x66,0x6c,0x6f,0x61,0x74,0x00,
                       0x3d,0x0a,0xd7,0xa3,0x70,0x3d,0x28,0x40,0x00]));
    assert_eq!(bson, Document::from_bytes(&bytes.unwrap()[]).unwrap());
}

#[test]
fn test_str_encode() {
    let mut bson = Document::new();
    bson.insert("str", "string".to_string());
    assert_eq!(bson.to_bytes(),
               Ok(vec![0x15,0x00,0x00,0x00,0x02,0x73,0x74,0x72,0x00,0x07,0x00,
                       0x00,0x00,0x73,0x74,0x72,0x69,0x6e,0x67,0x00,0x00]));
}

#[test]
fn test_doc_encode() {
    let mut bson = Document::new();
    let mut inner = Document::new();
    inner.insert("int", 1i32);
    bson.insert("d", inner);
    assert_eq!(bson.to_bytes(),
               Ok(vec![0x16,0x00,0x00,0x00,0x03,0x64,0x00,0x0e,0x00,0x00,0x00,
                       0x10,0x69,0x6e,0x74,0x00,0x01,0x00,0x00,0x00,0x00,0x00]));
}

#[test]
fn test_bool_encode() {
    let mut bson = Document::new();
    bson.insert("bool", true);
    assert_eq!(bson.to_bytes(),
               Ok(vec![0x0c,0x00,0x00,0x00,0x08,0x62,0x6f,0x6f,0x6c,0x00,0x01,
                       0x00]));
}

#[test]
fn test_null_encode() {
    let mut bson = Document::new();
    bson.insert("null", BsonValue::Null);
    assert_eq!(bson.to_bytes(),
               Ok(vec![0x0b,0x00,0x00,0x00,0x0a,0x6e,0x75,0x6c,0x6c,0x00,0x00]));
}

#[test]
fn test_i32_encode() {
    let mut bson = Document::new();
    bson.insert("int", 10i32);
    assert_eq!(bson.to_bytes(),
               Ok(vec![0x0e,0x00,0x00,0x00,0x10,0x69,0x6e,0x74,0x00,0x0a,0x00,
                       0x00,0x00,0x00]));
}
