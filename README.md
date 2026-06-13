# cbor-stream

A Rust library for **streaming CBOR (Concise Binary Object Representation)** encoding and decoding, implementing RFC 8949 with a focus on incremental parsing and compact wire formats for IoT, edge computing, and low-bandwidth protocols.

## Why It Matters

CBOR (RFC 8949) is the binary data format standardized by the IETF for resource-constrained environments. It is the preferred serialization format for:

- **CBOR Web Tokens (CWT)** — RFC 8392, the binary alternative to JWT
- **CoAP** — RFC 7252, the REST protocol for IoT devices
- **Android CameraX** — image metadata encoding
- **Apple HomeKit** — accessory protocol serialization
- **Cardano blockchain** — transaction serialization

CBOR is ~50% smaller than JSON for typical payloads and supports direct binary encoding without string conversion overhead. Streaming decoding is essential for network protocols where messages arrive in fragments.

## How It Works

### Major Type Encoding

CBOR uses a 3-bit major type in the high bits of the initial byte, followed by additional information encoding the value length:

| Major Type | Bits | Semantics |
|-----------|------|-----------|
| 0 | `000_____` | Unsigned integer |
| 1 | `001_____` | Negative integer (−1 − value) |
| 2 | `010_____` | Byte string |
| 3 | `011_____` | Text string |
| 4 | `100_____` | Array |
| 5 | `101_____` | Map |
| 6 | `110_____` | Tagged value |
| 7 | `111_____` | Simple/float |

### Variable-Length Integer Encoding

The "additional information" field (low 5 bits of the initial byte) determines value length:

| AI Value | Following Bytes | Range |
|----------|----------------|-------|
| 0–23 | 0 (immediate) | 0–23 |
| 24 | 1 | 0–255 |
| 25 | 2 | 0–65535 |
| 26 | 4 | 0−2³² |
| 27 | 8 | 0−2⁶⁴ |

This is a compact variable-length encoding: small values cost 1 byte, the maximum (64-bit) costs 9 bytes.

### Signed Integer Encoding (Major Type 1)

Negative integers use a complement encoding: $n$ is encoded as $-1 - n$. Thus $-1$ encodes as `0x20` (major 1, AI 0), $-2$ as `0x21`, etc.

### Streaming Decoder

The `StreamDecoder<R: Read>` wraps any reader and decodes CBOR items incrementally:

```rust
pub fn decode_next(&mut self) -> io::Result<Option<CborValue>>
```

This supports fragmented network reads — the decoder processes bytes as they arrive, returning `None` on incomplete data.

### Complexity Analysis

| Operation | Time | Space |
|-----------|------|-------|
| `encode(value)` | O(n) | O(n) |
| `decode_next()` | O(1) per item | O(1) |
| Full encode of n-element map | O(n) | O(n) |

## Quick Start

```rust
use cbor_stream::{CborValue, encode};

let val = CborValue::Array(vec![
    CborValue::Unsigned(42),
    CborValue::Text("hello".into()),
    CborValue::Null,
    CborValue::Float(3.14),
]);
let encoded: Vec<u8> = encode(&val);
println!("CBOR: {} bytes", encoded.len()); // compact binary
```

### Streaming from a reader:

```rust
use cbor_stream::StreamDecoder;
use std::io::Cursor;

let data = encode(&CborValue::Unsigned(42));
let reader = Cursor::new(data);
let mut decoder = StreamDecoder::new(reader);
if let Some(val) = decoder.decode_next().unwrap() {
    println!("Decoded: {:?}", val);
}
```

## API

| Type / Function | Description |
|-----------------|-------------|
| `CborValue` | Enum: Unsigned, Signed, Bytes, Text, Array, Map, Tag, Simple, Null, Undefined, Float |
| `encode(&CborValue) → Vec<u8>` | Full encoder |
| `StreamDecoder<R: Read>` | Incremental decoder |
| `StreamDecoder::decode_next() → Result<Option<CborValue>>` | Decode one item |

## Architecture Notes

The **γ + η = C** link: the head encoder (γ) maps values to the CBOR byte structure, while the major-type/AI decomposition (η) provides the framing boundary. Together they conserve the serialization invariant C — for any `CborValue`, the encoded bytes can be decoded back to an identical value. The streaming decoder maintains this invariant incrementally, ensuring partial reads never produce malformed values.

## References

- Bormann, C., & Hoffman, P. (2020). *Concise Binary Object Representation (CBOR).* RFC 8949.
- Bormann, C. (2020). *CBOR Tags.* RFC 8949, Section 3.4.
- Jones, M., Bradley, J., & Sakimura, N. (2018). *CBOR Web Token (CWT).* RFC 8392.
- Shelby, Z., Hartke, K., & Bormann, C. (2014). *The Constrained Application Protocol (CoAP).* RFC 7252.
- Borman, C. (2013). *A Roadmap for the Internet of Things.* (CBOR design rationale.)

## License

MIT
