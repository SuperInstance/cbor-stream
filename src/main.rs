//! cbor-stream: CBOR streaming encoder/decoder

use std::io::{self, Read, Write};

#[derive(Debug, Clone, PartialEq)]
pub enum CborValue {
    Unsigned(u64),
    Signed(i64),
    Bytes(Vec<u8>),
    Text(String),
    Array(Vec<CborValue>),
    Map(Vec<(CborValue, CborValue)>),
    Tag(u64, Box<CborValue>),
    Simple(u8),
    Null,
    Undefined,
    Float(f64),
}

const MAJOR_UNSIGNED: u8 = 0;
const MAJOR_TEXT: u8 = 3;
const MAJOR_ARRAY: u8 = 4;
const MAJOR_MAP: u8 = 5;
const MAJOR_TAG: u8 = 6;
const MAJOR_SIMPLE: u8 = 7;

fn encode_head(major: u8, value: u64, buf: &mut Vec<u8>) {
    if value <= 23 {
        buf.push((major << 5) | value as u8);
    } else if value <= 0xFF {
        buf.push((major << 5) | 24);
        buf.push(value as u8);
    } else if value <= 0xFFFF {
        buf.push((major << 5) | 25);
        buf.extend_from_slice(&(value as u16).to_be_bytes());
    } else if value <= 0xFFFFFFFF {
        buf.push((major << 5) | 26);
        buf.extend_from_slice(&(value as u32).to_be_bytes());
    } else {
        buf.push((major << 5) | 27);
        buf.extend_from_slice(&value.to_be_bytes());
    }
}

pub fn encode(value: &CborValue) -> Vec<u8> {
    let mut buf = Vec::new();
    encode_value(value, &mut buf);
    buf
}

fn encode_value(value: &CborValue, buf: &mut Vec<u8>) {
    match value {
        CborValue::Unsigned(n) => encode_head(MAJOR_UNSIGNED, *n, buf),
        CborValue::Signed(n) => {
            if *n >= 0 {
                encode_head(MAJOR_UNSIGNED, *n as u64, buf);
            } else {
                encode_head(1, (-1 - n) as u64, buf);
            }
        }
        CborValue::Text(s) => {
            encode_head(MAJOR_TEXT, s.len() as u64, buf);
            buf.extend_from_slice(s.as_bytes());
        }
        CborValue::Array(arr) => {
            encode_head(MAJOR_ARRAY, arr.len() as u64, buf);
            for item in arr {
                encode_value(item, buf);
            }
        }
        CborValue::Map(pairs) => {
            encode_head(MAJOR_MAP, pairs.len() as u64, buf);
            for (k, v) in pairs {
                encode_value(k, buf);
                encode_value(v, buf);
            }
        }
        CborValue::Tag(tag, inner) => {
            encode_head(MAJOR_TAG, *tag, buf);
            encode_value(inner, buf);
        }
        CborValue::Null => buf.push((MAJOR_SIMPLE << 5) | 22),
        CborValue::Undefined => buf.push((MAJOR_SIMPLE << 5) | 23),
        CborValue::Simple(n) => buf.push((MAJOR_SIMPLE << 5) | *n),
        CborValue::Bytes(b) => {
            encode_head(2, b.len() as u64, buf);
            buf.extend_from_slice(b);
        }
        CborValue::Float(f) => {
            buf.push((MAJOR_SIMPLE << 5) | 27);
            buf.extend_from_slice(&f.to_be_bytes());
        }
    }
}

pub struct StreamDecoder<R: Read> {
    reader: R,
}

impl<R: Read> StreamDecoder<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn decode_next(&mut self) -> io::Result<Option<CborValue>> {
        let mut byte = [0u8; 1];
        match self.reader.read_exact(&mut byte) {
            Ok(()) => Ok(Some(CborValue::Unsigned(byte[0] as u64 & 0x1F))),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => Ok(None),
            Err(e) => Err(e),
        }
    }
}

fn main() {
    let val = CborValue::Array(vec![
        CborValue::Unsigned(1),
        CborValue::Text("cbor".into()),
        CborValue::Null,
    ]);
    let encoded = encode(&val);
    println!("cbor-stream: encoded {} bytes", encoded.len());
}
