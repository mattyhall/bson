extern crate bson;

use std::old_io::IoResult;
use bson::{Document, BsonValue};

fn round_trip(bson: Document) -> IoResult<Vec<u8>> {
    let bytes = bson.to_bytes();
    let unwrapped = bytes.clone().unwrap();
    assert_eq!(Ok(bson), Document::from_bytes(&unwrapped[]));
    bytes
}

fn round_trip_bytes(bson: Document, expected: Vec<u8>) {
    let bytes = round_trip(bson);
    assert_eq!(bytes, Ok(expected));
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
fn test_f64() {
    let mut bson = Document::new();
    bson.insert("float", 12.12);
    round_trip_bytes(bson, vec![0x14,0x00,0x00,0x00,0x01,0x66,0x6c,0x6f,0x61,
                                0x74,0x00,0x3d,0x0a,0xd7,0xa3,0x70,0x3d,0x28,
                                0x40,0x00]);
}

#[test]
fn test_str() {
    let mut bson = Document::new();
    bson.insert("str", "string".to_string());
    round_trip_bytes(bson, vec![0x15,0x00,0x00,0x00,0x02,0x73,0x74,0x72,0x00,
                                0x07,0x00,0x00,0x00,0x73,0x74,0x72,0x69,0x6e,
                                0x67,0x00,0x00]);
}

#[test]
fn test_doc() {
    let mut bson = Document::new();
    let mut inner = Document::new();
    inner.insert("double", 12.12);
    bson.insert("d", inner);
    round_trip_bytes(bson, vec![0x1d,0x00,0x00,0x00,0x03,0x64,0x00,0x15,0x00,
                                0x00,0x00,0x01,0x64,0x6f,0x75,0x62,0x6c,0x65,
                                0x00,0x3d,0x0a,0xd7,0xa3,0x70,0x3d,0x28,0x40,
                                0x00,0x00]);
}

#[test]
fn test_bool() {
    let mut bson = Document::new();
    bson.insert("bool", true);
    round_trip_bytes(bson, vec![0x0c,0x00,0x00,0x00,0x08,0x62,0x6f,0x6f,0x6c,
                                0x00,0x01,0x00]);
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

#[test]
fn test_complex_doc() {
    let mut bson = Document::new();
    bson.insert("float", 12.12);
    let mut inner = Document::new();
    inner.insert("str", "This is a string".to_string());
    bson.insert("doc", inner);
    round_trip(bson);
}
