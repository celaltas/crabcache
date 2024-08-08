use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationType {
    Null,
    Err,
    Integer,
    String,
    Array,
}

impl SerializationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SerializationType::Null => "null",
            SerializationType::Err => "err",
            SerializationType::Integer => "integer",
            SerializationType::String => "string",
            SerializationType::Array => "array",
        }
    }

    pub fn as_num(&self) -> u8 {
        match self {
            SerializationType::Null => 0,
            SerializationType::Err => 1,
            SerializationType::Integer => 2,
            SerializationType::String => 3,
            SerializationType::Array => 4,
        }
    }
}

pub fn response_nil(out: &mut Vec<u8>) {
    out.push(SerializationType::Null.as_num());
}

pub fn response_err(out: &mut Vec<u8>, code: u32, message: &str) {
    out.push(SerializationType::Err.as_num());
    out.write_u32::<LittleEndian>(code).unwrap();
    let len = message.len() as u32;
    out.write_u32::<LittleEndian>(len).unwrap();
    out.extend_from_slice(message.as_bytes());
}

pub fn response_integer(out: &mut Vec<u8>, value: i64) {
    out.push(SerializationType::Integer.as_num());
    out.write_i64::<LittleEndian>(value).unwrap();
}

pub fn response_string(out: &mut Vec<u8>, value: &[u8]) {
    out.push(SerializationType::String.as_num());
    let len = value.len() as u32;
    out.write_u32::<LittleEndian>(len).unwrap();
    out.extend_from_slice(value);
}

pub fn response_array(out: &mut Vec<u8>, n: u32) {
    out.push(SerializationType::Array.as_num());
    out.write_u32::<LittleEndian>(n).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_nil() {
        let mut out = Vec::new();
        response_nil(&mut out);
        assert_eq!(out, vec![0]);
    }

    #[test]
    fn test_response_err() {
        let mut out = Vec::new();
        response_err(&mut out, 123, "Error message");
        assert_eq!(out.len(), 22);

        let expected = vec![
            SerializationType::Err.as_num(),
            0x7b,
            0x00,
            0x00,
            0x00,
            0x0d,
            0x00,
            0x00,
            0x00,
            b'E',
            b'r',
            b'r',
            b'o',
            b'r',
            b' ',
            b'm',
            b'e',
            b's',
            b's',
            b'a',
            b'g',
            b'e',
        ];
        assert_eq!(out, expected);
    }

    #[test]
    fn test_response_integer() {
        let mut out = Vec::new();
        response_integer(&mut out, 123456789);
        assert_eq!(out.len(), 9);

        let expected = vec![
            SerializationType::Integer.as_num(),
            0x15,
            0xcd,
            0x5b,
            0x07,
            0x00,
            0x00,
            0x00,
            0x00,
        ];
        assert_eq!(out, expected);
    }
}
