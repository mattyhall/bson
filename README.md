# Rust bson

A Rust implementation of bson. So far, the following types are supported:
   * Double
   * String
   * Doc
   * Bool
   * UTCDatetime
   * Null
   * Regex
   * Int32
   * Int64


Usage:

```rust
let mut bson = Document::new();
bson.insert("float", 12.12)
    .insert("int", 12i32)
    .insert("string", "This is a string");
```
