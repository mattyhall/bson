use std::collections::HashMap;
use std::old_io::{Writer, MemWriter, IoResult};

pub enum BsonValue {
    Int32(i32),
    Int64(i64),
    Double(f64),
}

pub trait ToBson {
    fn to_bson(&self) -> BsonValue;
}

impl ToBson for i32 {
    fn to_bson(&self) -> BsonValue { BsonValue::Int32(*self) }
}

impl ToBson for i64 {
    fn to_bson(&self) -> BsonValue { BsonValue::Int64(*self) }
}

impl ToBson for f64 {
    fn to_bson(&self) -> BsonValue { BsonValue::Double(*self) }
}

struct Document<'d> {
    hm: HashMap<&'d str, BsonValue> 
}

impl<'d> Document<'d> {
    pub fn new() -> Document<'d> {
        Document {hm: HashMap::new()}
    }

    pub fn insert<V: ToBson>(&mut self, key: &'d str, val: V) {
       self.hm.insert(key, val.to_bson());
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
                BsonValue::Double(_) => 8
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
                    try!(w.write_str(*key));
                    try!(w.write_u8(0x0));
                    try!(w.write_le_i32(v));
                },
                BsonValue::Int64(v) => {
                    try!(w.write_u8(0x12));
                    try!(w.write_str(*key));
                    try!(w.write_u8(0x0));
                    try!(w.write_le_i64(v));
                },
                BsonValue::Double(v) => {
                    try!(w.write_u8(0x01));
                    try!(w.write_str(*key));
                    try!(w.write_u8(0x0));
                    try!(w.write_le_f64(v));
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
fn simple_bson() {
    let mut bson = Document::new();
    bson.insert("key", 10i32);
    assert_eq!(bson.to_bytes(),
               Ok(vec![0x0e,0x00,0x00,0x00,0x10,0x6b,0x65,
                       0x79,0x00,0x0a,0x00,0x00,0x00,0x00]));
}
