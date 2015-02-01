#![feature(io, collections)]
use std::collections::HashMap;
use std::old_io::{Writer, MemWriter, IoResult};

pub enum BsonValue {
    Int32(i32),
    Int64(i64),
    Double(f64),
    String(String),
    Doc(Document),
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

fn write_cstring<W: Writer>(w: &mut W, s: &str) -> IoResult<()> {
    try!(w.write_str(s));
    w.write_u8(0x0)
}

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
                BsonValue::Int32(_) => 4,
                BsonValue::Int64(_) => 8,
                BsonValue::Double(_) => 8,
                // 4 bytes for the length, one for the NULL
                BsonValue::String(ref s) => 4 + s.len() as i32 + 1,
                BsonValue::Doc(ref d) => d.size(),
            }
        }
        size
    }
 
    pub fn write<W: Writer>(&self, w: &mut W) -> IoResult<()> {
        try!(w.write_le_i32(self.size()));
        for (key, val) in self.hm.iter() {
            match *val {
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
            }
        }
        w.write_u8(0x00)
    }


    pub fn to_bytes(&self) -> IoResult<Vec<u8>> {
        let mut writer = MemWriter::new();
        try!(self.write(&mut writer));
        Ok(writer.into_inner())
    }
}


#[test]
fn test_i32_encode() {
    let mut bson = Document::new();
    bson.insert("int", 10i32);
    assert_eq!(bson.to_bytes(),
               Ok(vec![0x0e,0x00,0x00,0x00,0x10,0x69,0x6e,0x74,0x00,0x0a,0x00,
                       0x00,0x00,0x00]));
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
    assert_eq!(bson.to_bytes(),
               Ok(vec![0x14,0x00,0x00,0x00,0x01,0x66,0x6c,0x6f,0x61,0x74,0x00,
                       0x3d,0x0a,0xd7,0xa3,0x70,0x3d,0x28,0x40,0x00]));
}


#[test]
fn test_str_encode() {
    let mut bson = Document::new();
    bson.insert("str", "string".to_string());
    assert_eq!(bson.to_bytes(),
               Ok(vec![0x15,0x00,0x00,0x00,0x02,0x73,0x74,0x72,0x00,0x07,0x00,
                       0x00,0x00,0x73,0x74,0x72,0x69,0x6e,0x67,0x00,0x00]));
}

fn test_doc_encode() {
    let mut bson = Document::new();
    let mut inner = Document::new();
    inner.insert("int", 1i32);
    bson.insert("d", inner);
    assert_eq!(bson.to_bytes(),
               Ok(vec![0x16,0x00,0x00,0x00,0x03,0x64,0x00,0x0e,0x00,0x00,0x00,
                       0x10,0x69,0x6e,0x74,0x00,0x01,0x00,0x00,0x00,0x00,0x00]));
}
