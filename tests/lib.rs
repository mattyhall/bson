extern crate bson;

use bson::{Document, BsonValue};

fn round_trip(bson: Document, expected: Vec<u8>) {
    let bytes = bson.to_bytes();
    assert_eq!(bytes, Ok(expected));
    let unwrapped = bytes.unwrap();
    assert_eq!(Ok(bson), Document::from_bytes(&unwrapped[]));
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
    round_trip(bson, vec![0x14,0x00,0x00,0x00,0x01,0x66,0x6c,0x6f,0x61,0x74,
                          0x00,0x3d,0x0a,0xd7,0xa3,0x70,0x3d,0x28,0x40,0x00]);
}

#[test]
fn test_str_encode() {
    let mut bson = Document::new();
    bson.insert("str", "string".to_string());
    round_trip(bson, vec![0x15,0x00,0x00,0x00,0x02,0x73,0x74,0x72,0x00,0x07,
                          0x00,0x00,0x00,0x73,0x74,0x72,0x69,0x6e,0x67,0x00,
                          0x00]);
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
