//! Bencode encoding/decoding
//!
//! Implements bencode format as defined in BEP 3.
//! Supports integers, byte strings, lists, and dictionaries.

use std::collections::BTreeMap;
use std::fmt;

/// Bencode value type
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Integer: i<number>e
    Integer(i64),
    /// Byte string: <length>:<bytes>
    Bytes(Vec<u8>),
    /// List: l<items>e
    List(Vec<Value>),
    /// Dictionary: d<key><value>...e (keys are byte strings, sorted)
    Dict(BTreeMap<Vec<u8>, Value>),
}

impl Value {
    /// Get as integer
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(n) => Some(*n),
            _ => None,
        }
    }

    /// Get as byte string
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(b) => Some(b),
            _ => None,
        }
    }

    /// Get as string (UTF-8)
    pub fn as_string(&self) -> Option<&str> {
        self.as_bytes().and_then(|b| std::str::from_utf8(b).ok())
    }

    /// Get as list
    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }

    /// Get as dictionary
    pub fn as_dict(&self) -> Option<&BTreeMap<Vec<u8>, Value>> {
        match self {
            Value::Dict(d) => Some(d),
            _ => None,
        }
    }

    /// Get dictionary value by string key
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Dict(d) => d.get(key.as_bytes()),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Bytes(b) => {
                if let Ok(s) = std::str::from_utf8(b) {
                    write!(f, "\"{}\"", s)
                } else {
                    write!(f, "<{} bytes>", b.len())
                }
            }
            Value::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Dict(d) => {
                write!(f, "{{")?;
                for (i, (k, v)) in d.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    if let Ok(s) = std::str::from_utf8(k) {
                        write!(f, "\"{}\": {}", s, v)?;
                    } else {
                        write!(f, "<key>: {}", v)?;
                    }
                }
                write!(f, "}}")
            }
        }
    }
}

/// Bencode decoder error
#[derive(Debug, Clone)]
pub enum DecodeError {
    InvalidFormat(String),
    UnexpectedEnd,
    InvalidInteger,
    InvalidLength,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            DecodeError::UnexpectedEnd => write!(f, "Unexpected end of input"),
            DecodeError::InvalidInteger => write!(f, "Invalid integer"),
            DecodeError::InvalidLength => write!(f, "Invalid length specification"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Bencode decoder
pub struct Decoder;

impl Decoder {
    /// Decode bencode bytes into a Value
    pub fn decode(data: &[u8]) -> Result<Value, DecodeError> {
        let (value, remaining) = Self::decode_value(data)?;
        if !remaining.is_empty() {
            return Err(DecodeError::InvalidFormat(
                "Extra data after bencode value".to_string(),
            ));
        }
        Ok(value)
    }

    fn decode_value(data: &[u8]) -> Result<(Value, &[u8]), DecodeError> {
        if data.is_empty() {
            return Err(DecodeError::UnexpectedEnd);
        }

        match data[0] {
            b'i' => Self::decode_integer(&data[1..]),
            b'l' => Self::decode_list(&data[1..]),
            b'd' => Self::decode_dict(&data[1..]),
            b'0'..=b'9' => Self::decode_bytes(data),
            _ => Err(DecodeError::InvalidFormat(format!(
                "Unexpected byte: {}",
                data[0] as char
            ))),
        }
    }

    fn decode_integer(data: &[u8]) -> Result<(Value, &[u8]), DecodeError> {
        let end = data
            .iter()
            .position(|&b| b == b'e')
            .ok_or(DecodeError::UnexpectedEnd)?;

        let num_str = std::str::from_utf8(&data[..end])
            .map_err(|_| DecodeError::InvalidInteger)?;

        let num = num_str
            .parse::<i64>()
            .map_err(|_| DecodeError::InvalidInteger)?;

        Ok((Value::Integer(num), &data[end + 1..]))
    }

    fn decode_bytes(data: &[u8]) -> Result<(Value, &[u8]), DecodeError> {
        let colon = data
            .iter()
            .position(|&b| b == b':')
            .ok_or(DecodeError::InvalidLength)?;

        let len_str = std::str::from_utf8(&data[..colon])
            .map_err(|_| DecodeError::InvalidLength)?;

        let len = len_str
            .parse::<usize>()
            .map_err(|_| DecodeError::InvalidLength)?;

        let start = colon + 1;
        let end = start + len;

        if end > data.len() {
            return Err(DecodeError::UnexpectedEnd);
        }

        Ok((Value::Bytes(data[start..end].to_vec()), &data[end..]))
    }

    fn decode_list(data: &[u8]) -> Result<(Value, &[u8]), DecodeError> {
        let mut list = Vec::new();
        let mut remaining = data;

        loop {
            if remaining.is_empty() {
                return Err(DecodeError::UnexpectedEnd);
            }

            if remaining[0] == b'e' {
                return Ok((Value::List(list), &remaining[1..]));
            }

            let (value, rest) = Self::decode_value(remaining)?;
            list.push(value);
            remaining = rest;
        }
    }

    fn decode_dict(data: &[u8]) -> Result<(Value, &[u8]), DecodeError> {
        let mut dict = BTreeMap::new();
        let mut remaining = data;

        loop {
            if remaining.is_empty() {
                return Err(DecodeError::UnexpectedEnd);
            }

            if remaining[0] == b'e' {
                return Ok((Value::Dict(dict), &remaining[1..]));
            }

            // Keys must be byte strings in sorted order
            let (key_value, rest) = Self::decode_bytes(remaining)?;
            let key = match key_value {
                Value::Bytes(b) => b,
                _ => unreachable!(),
            };

            if rest.is_empty() {
                return Err(DecodeError::UnexpectedEnd);
            }

            let (value, rest) = Self::decode_value(rest)?;
            dict.insert(key, value);
            remaining = rest;
        }
    }
}

/// Bencode encoder
pub struct Encoder;

impl Encoder {
    /// Encode a Value into bencode bytes
    pub fn encode(value: &Value) -> Vec<u8> {
        let mut result = Vec::new();
        Self::encode_value(value, &mut result);
        result
    }

    fn encode_value(value: &Value, buf: &mut Vec<u8>) {
        match value {
            Value::Integer(n) => {
                buf.push(b'i');
                buf.extend_from_slice(n.to_string().as_bytes());
                buf.push(b'e');
            }
            Value::Bytes(b) => {
                buf.extend_from_slice(b.len().to_string().as_bytes());
                buf.push(b':');
                buf.extend_from_slice(b);
            }
            Value::List(l) => {
                buf.push(b'l');
                for item in l {
                    Self::encode_value(item, buf);
                }
                buf.push(b'e');
            }
            Value::Dict(d) => {
                buf.push(b'd');
                for (k, v) in d {
                    Self::encode_value(&Value::Bytes(k.clone()), buf);
                    Self::encode_value(v, buf);
                }
                buf.push(b'e');
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_integer() {
        let result = Decoder::decode(b"i42e").unwrap();
        assert_eq!(result, Value::Integer(42));

        let result = Decoder::decode(b"i-10e").unwrap();
        assert_eq!(result, Value::Integer(-10));
    }

    #[test]
    fn test_decode_bytes() {
        let result = Decoder::decode(b"4:spam").unwrap();
        assert_eq!(result, Value::Bytes(b"spam".to_vec()));

        let result = Decoder::decode(b"0:").unwrap();
        assert_eq!(result, Value::Bytes(Vec::new()));
    }

    #[test]
    fn test_decode_list() {
        let result = Decoder::decode(b"li1ei2ei3ee").unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3)
            ])
        );
    }

    #[test]
    fn test_decode_dict() {
        let result = Decoder::decode(b"d3:bari1e3:fooi2ee").unwrap();
        let mut expected = BTreeMap::new();
        expected.insert(b"bar".to_vec(), Value::Integer(1));
        expected.insert(b"foo".to_vec(), Value::Integer(2));
        assert_eq!(result, Value::Dict(expected));
    }

    #[test]
    fn test_encode_integer() {
        let value = Value::Integer(42);
        let encoded = Encoder::encode(&value);
        assert_eq!(encoded, b"i42e");
    }

    #[test]
    fn test_encode_bytes() {
        let value = Value::Bytes(b"spam".to_vec());
        let encoded = Encoder::encode(&value);
        assert_eq!(encoded, b"4:spam");
    }

    #[test]
    fn test_encode_list() {
        let value = Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let encoded = Encoder::encode(&value);
        assert_eq!(encoded, b"li1ei2ei3ee");
    }

    #[test]
    fn test_round_trip() {
        let original = b"d3:bari1e3:fooi2ee";
        let decoded = Decoder::decode(original).unwrap();
        let encoded = Encoder::encode(&decoded);
        assert_eq!(encoded, original);
    }
}

