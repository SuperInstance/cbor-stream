# CBOR Stream

**A Rust library for streaming CBOR (Concise Binary Object Representation) encoding and decoding** — RFC 8949 compliant binary serialization with a typed value model and incremental stream parser.

## Why It Matters

CBOR (RFC 8949) is a binary data format designed for the Internet of Things and constrained devices. It's more compact than JSON (50–70% smaller for typical payloads), supports binary data natively (no Base64 encoding needed), and has a formal type system.

CBOR is used in:
- **COAP** (Constrained Application Protocol) — the IoT equivalent of HTTP
- **WebAuthn / FIDO2** — passwordless authentication (YubiKey, Face ID)
- **SenML** — sensor measurement data (RFC 8428)
- **CDDL** — CBOR data definition language

The format uses **major types** (3 bits) + **additional information** (5 bits) for compact length encoding: values 0–23 fit in a single byte, 24–255 use 2 bytes, 256–65535 uses 3 bytes, etc.

## How It Works

**Type system** (`CborValue`): Models all CBOR major types:
- 0: Unsigned integer (variable length)
- 1: Negative integer (encoded as −1 − n)
- 2: Byte string
- 3: Text string (UTF-8)
- 4: Array (homogeneous or heterogeneous)
- 5: Map (key-value pairs)
- 6: Tagged value (semantic tag + value)
- 7: Simple values (null, undefined, bool, float)

**Head encoding** (`encode_head`): The initial byte's top 3 bits are the major type; the low 5 bits encode the value or a length specifier. Values 0–23 are inline. 24 means "next byte is the value", 25 = "next 2 bytes" (u16), 26 = "next 4 bytes" (u32), 27 = "next 8 bytes" (u64). All multi-byte values are big-endian.

**Encoder**: Recursively walks the `CborValue` tree, writing the head + payload for each value. Arrays and maps emit their count in the head, followed by each element.

**Stream decoder** (`StreamDecoder<R: Read>`): Wraps any `Read` stream and decodes CBOR values incrementally. `decode_next()` reads one value at a time — suitable for message-oriented protocols where CBOR items arrive over a socket.

## Quick Start

```rust
use cbor_stream::{CborValue, encode};

// Build a CBOR map
let value = CborValue::Array(vec![
    CborValue::Unsigned(42),
    CborValue::Text("hello cbor".into()),
    CborValue::Null,
    CborValue::Array(vec![
        CborValue::Unsigned(1),
        CborValue::Unsigned(2),
        CborValue::Unsigned(3),
    ]),
]);

// Encode to bytes
let encoded = encode(&value);
println!("Encoded {} bytes", encoded.len());

// Stream decode
use cbor_stream::StreamDecoder;
use std::io::Cursor;
let mut decoder = StreamDecoder::new(Cursor::new(&encoded));
if let Some(val) = decoder.decode_next().unwrap() {
    println!("Decoded: {:?}", val);
}
```

## API

- **`CborValue`** — Enum: `Unsigned`, `Signed`, `Bytes`, `Text`, `Array`, `Map`, `Tag`, `Simple`, `Null`, `Undefined`, `Float`
- **`encode(value)` → `Vec<u8>`** — Serialize a CborValue to bytes
- **`StreamDecoder<R: Read>`** — Incremental decoder: `new(reader)`, `decode_next()`

## Architecture Notes

Provides the binary serialization layer for SuperInstance IoT and edge computing pipelines. CBOR's compact encoding makes it ideal for bandwidth-constrained environments where JSON would be too verbose. See the [architecture overview](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## License

MIT
