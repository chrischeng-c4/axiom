//! Wire protocol implementation
//!
//! Binary protocol for KV operations.

use crate::KvValue;
use std::collections::HashMap;
use std::io;
use thiserror::Error;

/// Protocol error types
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid command: {0}")]
    InvalidCommand(u8),

    #[error("Invalid value type: {0}")]
    InvalidValueType(u8),

    #[error("Payload too large: {0} bytes (max 64MB)")]
    PayloadTooLarge(u32),

    #[error("Invalid UTF-8 string")]
    InvalidUtf8,

    #[error("Unexpected end of data")]
    UnexpectedEof,
}

/// Command codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Get = 0x01,
    Set = 0x02,
    Del = 0x03,
    Exists = 0x04,
    Incr = 0x05,
    Decr = 0x06,
    Cas = 0x07,
    Ping = 0x08,
    Info = 0x09,
    // Lock commands
    Setnx = 0x0A,
    Lock = 0x0B,
    Unlock = 0x0C,
    ExtendLock = 0x0D,
    // Batch commands
    MGet = 0x0E,
    MSet = 0x0F,
    MDel = 0x10,
    // Scan command
    Scan = 0x11,
    // List commands
    LPush = 0x12,
    RPush = 0x13,
    LPop = 0x14,
    RPop = 0x15,
    BLPop = 0x16,
    BRPop = 0x17,
}

impl TryFrom<u8> for Command {
    type Error = ProtocolError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0x01 => Ok(Command::Get),
            0x02 => Ok(Command::Set),
            0x03 => Ok(Command::Del),
            0x04 => Ok(Command::Exists),
            0x05 => Ok(Command::Incr),
            0x06 => Ok(Command::Decr),
            0x07 => Ok(Command::Cas),
            0x08 => Ok(Command::Ping),
            0x09 => Ok(Command::Info),
            0x0A => Ok(Command::Setnx),
            0x0B => Ok(Command::Lock),
            0x0C => Ok(Command::Unlock),
            0x0D => Ok(Command::ExtendLock),
            0x0E => Ok(Command::MGet),
            0x0F => Ok(Command::MSet),
            0x10 => Ok(Command::MDel),
            0x11 => Ok(Command::Scan),
            0x12 => Ok(Command::LPush),
            0x13 => Ok(Command::RPush),
            0x14 => Ok(Command::LPop),
            0x15 => Ok(Command::RPop),
            0x16 => Ok(Command::BLPop),
            0x17 => Ok(Command::BRPop),
            _ => Err(ProtocolError::InvalidCommand(byte)),
        }
    }
}

/// Response status codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Ok = 0x00,
    Null = 0x01,
    Error = 0x02,
}

/// Value type codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueType {
    Null = 0x00,
    Int = 0x01,
    Float = 0x02,
    Decimal = 0x03,
    String = 0x04,
    Bytes = 0x05,
    List = 0x06,
    Map = 0x07,
    #[allow(dead_code)]
    Bool = 0x08,
    Set = 0x09,
    SortedSet = 0x0A,
}

const MAX_PAYLOAD_SIZE: u32 = 64 * 1024 * 1024; // 64MB

/// Read a request from bytes
pub fn read_request(data: &[u8]) -> Result<(Command, Vec<u8>), ProtocolError> {
    if data.len() < 5 {
        return Err(ProtocolError::UnexpectedEof);
    }

    let cmd = Command::try_from(data[0])?;
    let len = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);

    if len > MAX_PAYLOAD_SIZE {
        return Err(ProtocolError::PayloadTooLarge(len));
    }

    let expected_len = 5 + len as usize;
    if data.len() < expected_len {
        return Err(ProtocolError::UnexpectedEof);
    }

    let payload = data[5..expected_len].to_vec();
    Ok((cmd, payload))
}

/// Write a response to bytes
pub fn write_response(status: Status, payload: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(5 + payload.len());
    buf.push(status as u8);
    buf.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    buf.extend_from_slice(payload);
    buf
}

/// Encode a KvValue to bytes
pub fn encode_value(value: &KvValue) -> Vec<u8> {
    let mut buf = Vec::new();
    encode_value_into(&mut buf, value);
    buf
}

fn encode_value_into(buf: &mut Vec<u8>, value: &KvValue) {
    match value {
        KvValue::Null => {
            buf.push(ValueType::Null as u8);
        }
        KvValue::Int(n) => {
            buf.push(ValueType::Int as u8);
            buf.extend_from_slice(&n.to_be_bytes());
        }
        KvValue::Float(f) => {
            buf.push(ValueType::Float as u8);
            buf.extend_from_slice(&f.to_be_bytes());
        }
        KvValue::Decimal(d) => {
            buf.push(ValueType::Decimal as u8);
            let s = d.to_string();
            buf.extend_from_slice(&(s.len() as u16).to_be_bytes());
            buf.extend_from_slice(s.as_bytes());
        }
        KvValue::String(s) => {
            buf.push(ValueType::String as u8);
            buf.extend_from_slice(&(s.len() as u32).to_be_bytes());
            buf.extend_from_slice(s.as_bytes());
        }
        KvValue::Bytes(b) => {
            buf.push(ValueType::Bytes as u8);
            buf.extend_from_slice(&(b.len() as u32).to_be_bytes());
            buf.extend_from_slice(b);
        }
        KvValue::List(items) => {
            buf.push(ValueType::List as u8);
            buf.extend_from_slice(&(items.len() as u32).to_be_bytes());
            for item in items {
                encode_value_into(buf, item);
            }
        }
        KvValue::Map(map) => {
            buf.push(ValueType::Map as u8);
            buf.extend_from_slice(&(map.len() as u32).to_be_bytes());
            for (k, v) in map {
                buf.extend_from_slice(&(k.len() as u16).to_be_bytes());
                buf.extend_from_slice(k.as_bytes());
                encode_value_into(buf, v);
            }
        }
        KvValue::Set(set) => {
            buf.push(ValueType::Set as u8);
            buf.extend_from_slice(&(set.len() as u32).to_be_bytes());
            for member in set {
                buf.extend_from_slice(&(member.len() as u16).to_be_bytes());
                buf.extend_from_slice(member.as_bytes());
            }
        }
        KvValue::SortedSet(ss) => {
            buf.push(ValueType::SortedSet as u8);
            buf.extend_from_slice(&(ss.len() as u32).to_be_bytes());
            for (member, score) in ss {
                buf.extend_from_slice(&(member.len() as u16).to_be_bytes());
                buf.extend_from_slice(member.as_bytes());
                buf.extend_from_slice(&score.to_be_bytes());
            }
        }
    }
}

/// Decode a KvValue from bytes
pub fn decode_value(data: &[u8]) -> Result<(KvValue, usize), ProtocolError> {
    if data.is_empty() {
        return Err(ProtocolError::UnexpectedEof);
    }

    let type_byte = data[0];
    let mut pos = 1;

    match type_byte {
        0x00 => Ok((KvValue::Null, pos)),
        0x01 => {
            // Int
            if data.len() < pos + 8 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let n = i64::from_be_bytes(data[pos..pos + 8].try_into().unwrap());
            Ok((KvValue::Int(n), pos + 8))
        }
        0x02 => {
            // Float
            if data.len() < pos + 8 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let f = f64::from_be_bytes(data[pos..pos + 8].try_into().unwrap());
            Ok((KvValue::Float(f), pos + 8))
        }
        0x03 => {
            // Decimal
            if data.len() < pos + 2 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let len = u16::from_be_bytes(data[pos..pos + 2].try_into().unwrap()) as usize;
            pos += 2;
            if data.len() < pos + len {
                return Err(ProtocolError::UnexpectedEof);
            }
            let s = std::str::from_utf8(&data[pos..pos + len])
                .map_err(|_| ProtocolError::InvalidUtf8)?;
            let d =
                rust_decimal::Decimal::from_str_exact(s).map_err(|_| ProtocolError::InvalidUtf8)?;
            Ok((KvValue::Decimal(d), pos + len))
        }
        0x04 => {
            // String
            if data.len() < pos + 4 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let len = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            if data.len() < pos + len {
                return Err(ProtocolError::UnexpectedEof);
            }
            let s = std::str::from_utf8(&data[pos..pos + len])
                .map_err(|_| ProtocolError::InvalidUtf8)?
                .to_string();
            Ok((KvValue::String(s), pos + len))
        }
        0x05 => {
            // Bytes
            if data.len() < pos + 4 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let len = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            if data.len() < pos + len {
                return Err(ProtocolError::UnexpectedEof);
            }
            let b = data[pos..pos + len].to_vec();
            Ok((KvValue::Bytes(b), pos + len))
        }
        0x06 => {
            // List
            if data.len() < pos + 4 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let count = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                let (item, consumed) = decode_value(&data[pos..])?;
                items.push(item);
                pos += consumed;
            }
            Ok((KvValue::List(items), pos))
        }
        0x07 => {
            // Map
            if data.len() < pos + 4 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let count = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            let mut map = HashMap::with_capacity(count);
            for _ in 0..count {
                if data.len() < pos + 2 {
                    return Err(ProtocolError::UnexpectedEof);
                }
                let key_len = u16::from_be_bytes(data[pos..pos + 2].try_into().unwrap()) as usize;
                pos += 2;
                if data.len() < pos + key_len {
                    return Err(ProtocolError::UnexpectedEof);
                }
                let key = std::str::from_utf8(&data[pos..pos + key_len])
                    .map_err(|_| ProtocolError::InvalidUtf8)?
                    .to_string();
                pos += key_len;
                let (value, consumed) = decode_value(&data[pos..])?;
                map.insert(key, value);
                pos += consumed;
            }
            Ok((KvValue::Map(map), pos))
        }
        0x09 => {
            // Set
            use std::collections::HashSet;
            if data.len() < pos + 4 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let count = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            let mut set = HashSet::with_capacity(count);
            for _ in 0..count {
                if data.len() < pos + 2 {
                    return Err(ProtocolError::UnexpectedEof);
                }
                let member_len =
                    u16::from_be_bytes(data[pos..pos + 2].try_into().unwrap()) as usize;
                pos += 2;
                if data.len() < pos + member_len {
                    return Err(ProtocolError::UnexpectedEof);
                }
                let member = std::str::from_utf8(&data[pos..pos + member_len])
                    .map_err(|_| ProtocolError::InvalidUtf8)?
                    .to_string();
                pos += member_len;
                set.insert(member);
            }
            Ok((KvValue::Set(set), pos))
        }
        0x0A => {
            // SortedSet
            use std::collections::BTreeMap;
            if data.len() < pos + 4 {
                return Err(ProtocolError::UnexpectedEof);
            }
            let count = u32::from_be_bytes(data[pos..pos + 4].try_into().unwrap()) as usize;
            pos += 4;
            let mut ss = BTreeMap::new();
            for _ in 0..count {
                if data.len() < pos + 2 {
                    return Err(ProtocolError::UnexpectedEof);
                }
                let member_len =
                    u16::from_be_bytes(data[pos..pos + 2].try_into().unwrap()) as usize;
                pos += 2;
                if data.len() < pos + member_len {
                    return Err(ProtocolError::UnexpectedEof);
                }
                let member = std::str::from_utf8(&data[pos..pos + member_len])
                    .map_err(|_| ProtocolError::InvalidUtf8)?
                    .to_string();
                pos += member_len;
                if data.len() < pos + 8 {
                    return Err(ProtocolError::UnexpectedEof);
                }
                let score = f64::from_be_bytes(data[pos..pos + 8].try_into().unwrap());
                pos += 8;
                ss.insert(member, score);
            }
            Ok((KvValue::SortedSet(ss), pos))
        }
        _ => Err(ProtocolError::InvalidValueType(type_byte)),
    }
}

/// Parse key from payload
pub fn parse_key(payload: &[u8]) -> Result<String, ProtocolError> {
    std::str::from_utf8(payload)
        .map(|s| s.to_string())
        .map_err(|_| ProtocolError::InvalidUtf8)
}

/// Parse SET payload: key_len(2) + key + ttl(8) + value
pub fn parse_set_payload(payload: &[u8]) -> Result<(String, Option<u64>, KvValue), ProtocolError> {
    if payload.len() < 2 {
        return Err(ProtocolError::UnexpectedEof);
    }
    let key_len = u16::from_be_bytes(payload[0..2].try_into().unwrap()) as usize;
    let mut pos = 2;

    if payload.len() < pos + key_len + 8 {
        return Err(ProtocolError::UnexpectedEof);
    }
    let key = std::str::from_utf8(&payload[pos..pos + key_len])
        .map_err(|_| ProtocolError::InvalidUtf8)?
        .to_string();
    pos += key_len;

    let ttl_ms = u64::from_be_bytes(payload[pos..pos + 8].try_into().unwrap());
    let ttl = if ttl_ms == 0 { None } else { Some(ttl_ms) };
    pos += 8;

    let (value, _) = decode_value(&payload[pos..])?;
    Ok((key, ttl, value))
}

/// Parse INCR/DECR payload: key_len(2) + key + delta(8)
pub fn parse_incr_payload(payload: &[u8]) -> Result<(String, i64), ProtocolError> {
    if payload.len() < 2 {
        return Err(ProtocolError::UnexpectedEof);
    }
    let key_len = u16::from_be_bytes(payload[0..2].try_into().unwrap()) as usize;
    let mut pos = 2;

    if payload.len() < pos + key_len + 8 {
        return Err(ProtocolError::UnexpectedEof);
    }
    let key = std::str::from_utf8(&payload[pos..pos + key_len])
        .map_err(|_| ProtocolError::InvalidUtf8)?
        .to_string();
    pos += key_len;

    let delta = i64::from_be_bytes(payload[pos..pos + 8].try_into().unwrap());
    Ok((key, delta))
}

/// Parse LOCK/UNLOCK/EXTEND payload: key_len(2) + key + owner_len(2) + owner + ttl(8, only for LOCK/EXTEND)
pub fn parse_lock_payload(
    payload: &[u8],
    with_ttl: bool,
) -> Result<(String, String, Option<u64>), ProtocolError> {
    if payload.len() < 4 {
        return Err(ProtocolError::UnexpectedEof);
    }

    let key_len = u16::from_be_bytes(payload[0..2].try_into().unwrap()) as usize;
    let mut pos = 2;

    if payload.len() < pos + key_len + 2 {
        return Err(ProtocolError::UnexpectedEof);
    }

    let key = std::str::from_utf8(&payload[pos..pos + key_len])
        .map_err(|_| ProtocolError::InvalidUtf8)?
        .to_string();
    pos += key_len;

    let owner_len = u16::from_be_bytes(payload[pos..pos + 2].try_into().unwrap()) as usize;
    pos += 2;

    if payload.len() < pos + owner_len {
        return Err(ProtocolError::UnexpectedEof);
    }

    let owner = std::str::from_utf8(&payload[pos..pos + owner_len])
        .map_err(|_| ProtocolError::InvalidUtf8)?
        .to_string();
    pos += owner_len;

    let ttl = if with_ttl {
        if payload.len() < pos + 8 {
            return Err(ProtocolError::UnexpectedEof);
        }
        Some(u64::from_be_bytes(
            payload[pos..pos + 8].try_into().unwrap(),
        ))
    } else {
        None
    };

    Ok((key, owner, ttl))
}

/// Parse MGET/MDEL payload: count(2) + [key_len(2) + key]...
pub fn parse_mget_payload(payload: &[u8]) -> Result<Vec<String>, ProtocolError> {
    if payload.len() < 2 {
        return Err(ProtocolError::UnexpectedEof);
    }

    let count = u16::from_be_bytes(payload[0..2].try_into().unwrap()) as usize;
    let mut pos = 2;
    let mut keys = Vec::with_capacity(count);

    for _ in 0..count {
        if payload.len() < pos + 2 {
            return Err(ProtocolError::UnexpectedEof);
        }
        let key_len = u16::from_be_bytes(payload[pos..pos + 2].try_into().unwrap()) as usize;
        pos += 2;

        if payload.len() < pos + key_len {
            return Err(ProtocolError::UnexpectedEof);
        }
        let key = std::str::from_utf8(&payload[pos..pos + key_len])
            .map_err(|_| ProtocolError::InvalidUtf8)?
            .to_string();
        pos += key_len;
        keys.push(key);
    }

    Ok(keys)
}

/// Parse MSET payload: count(2) + ttl(8) + [key_len(2) + key + value]...
pub fn parse_mset_payload(
    payload: &[u8],
) -> Result<(Vec<(String, KvValue)>, Option<u64>), ProtocolError> {
    if payload.len() < 10 {
        // count(2) + ttl(8)
        return Err(ProtocolError::UnexpectedEof);
    }

    let count = u16::from_be_bytes(payload[0..2].try_into().unwrap()) as usize;
    let ttl_ms = u64::from_be_bytes(payload[2..10].try_into().unwrap());
    let ttl = if ttl_ms == 0 { None } else { Some(ttl_ms) };

    let mut pos = 10;
    let mut pairs = Vec::with_capacity(count);

    for _ in 0..count {
        if payload.len() < pos + 2 {
            return Err(ProtocolError::UnexpectedEof);
        }
        let key_len = u16::from_be_bytes(payload[pos..pos + 2].try_into().unwrap()) as usize;
        pos += 2;

        if payload.len() < pos + key_len {
            return Err(ProtocolError::UnexpectedEof);
        }
        let key = std::str::from_utf8(&payload[pos..pos + key_len])
            .map_err(|_| ProtocolError::InvalidUtf8)?
            .to_string();
        pos += key_len;

        let (value, consumed) = decode_value(&payload[pos..])?;
        pos += consumed;
        pairs.push((key, value));
    }

    Ok((pairs, ttl))
}

/// Encode MGET response: count(2) + [value_or_null]...
pub fn encode_mget_response(values: &[Option<KvValue>]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(values.len() as u16).to_be_bytes());

    for value_opt in values {
        match value_opt {
            Some(value) => encode_value_into(&mut buf, value),
            None => buf.push(ValueType::Null as u8),
        }
    }

    buf
}

/// Parse SCAN payload: limit(4) + prefix_len(2) + prefix
/// If prefix_len is 0, no prefix filter is applied
pub fn parse_scan_payload(payload: &[u8]) -> Result<(Option<String>, usize), ProtocolError> {
    if payload.len() < 6 {
        return Err(ProtocolError::UnexpectedEof);
    }

    let limit = u32::from_be_bytes(payload[0..4].try_into().unwrap()) as usize;
    let prefix_len = u16::from_be_bytes(payload[4..6].try_into().unwrap()) as usize;

    let prefix = if prefix_len > 0 {
        if payload.len() < 6 + prefix_len {
            return Err(ProtocolError::UnexpectedEof);
        }
        let prefix_str = std::str::from_utf8(&payload[6..6 + prefix_len])
            .map_err(|_| ProtocolError::InvalidUtf8)?
            .to_string();
        Some(prefix_str)
    } else {
        None
    };

    Ok((prefix, limit))
}

/// Encode SCAN response: count(4) + [key_len(2) + key]...
pub fn encode_scan_response(keys: &[String]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(keys.len() as u32).to_be_bytes());

    for key in keys {
        buf.extend_from_slice(&(key.len() as u16).to_be_bytes());
        buf.extend_from_slice(key.as_bytes());
    }

    buf
}

/// Parse LPUSH/RPUSH payload: key_len(2) + key + value
pub fn parse_list_push_payload(payload: &[u8]) -> Result<(String, KvValue), ProtocolError> {
    if payload.len() < 2 {
        return Err(ProtocolError::UnexpectedEof);
    }
    let key_len = u16::from_be_bytes(payload[0..2].try_into().unwrap()) as usize;
    let mut pos = 2;

    if payload.len() < pos + key_len {
        return Err(ProtocolError::UnexpectedEof);
    }
    let key = std::str::from_utf8(&payload[pos..pos + key_len])
        .map_err(|_| ProtocolError::InvalidUtf8)?
        .to_string();
    pos += key_len;

    let (value, _) = decode_value(&payload[pos..])?;
    Ok((key, value))
}

/// Parse BLPOP/BRPOP payload: timeout_ms(8) + num_keys(2) + [key_len(2) + key]...
pub fn parse_blocking_pop_payload(payload: &[u8]) -> Result<(u64, Vec<String>), ProtocolError> {
    if payload.len() < 10 {
        return Err(ProtocolError::UnexpectedEof);
    }

    let timeout_ms = u64::from_be_bytes(payload[0..8].try_into().unwrap());
    let num_keys = u16::from_be_bytes(payload[8..10].try_into().unwrap()) as usize;
    let mut pos = 10;
    let mut keys = Vec::with_capacity(num_keys);

    for _ in 0..num_keys {
        if payload.len() < pos + 2 {
            return Err(ProtocolError::UnexpectedEof);
        }
        let key_len = u16::from_be_bytes(payload[pos..pos + 2].try_into().unwrap()) as usize;
        pos += 2;

        if payload.len() < pos + key_len {
            return Err(ProtocolError::UnexpectedEof);
        }
        let key = std::str::from_utf8(&payload[pos..pos + key_len])
            .map_err(|_| ProtocolError::InvalidUtf8)?
            .to_string();
        pos += key_len;
        keys.push(key);
    }

    Ok((timeout_ms, keys))
}

/// Encode BLPOP/BRPOP response: key_len(2) + key + value (or empty for timeout)
pub fn encode_blocking_pop_response(key: &str, value: &KvValue) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(key.len() as u16).to_be_bytes());
    buf.extend_from_slice(key.as_bytes());
    encode_value_into(&mut buf, value);
    buf
}

/// Parse CAS payload: key_len(2) + key + ttl(8) + expected_value + new_value
pub fn parse_cas_payload(
    payload: &[u8],
) -> Result<(String, Option<u64>, KvValue, KvValue), ProtocolError> {
    if payload.len() < 2 {
        return Err(ProtocolError::UnexpectedEof);
    }
    let key_len = u16::from_be_bytes(payload[0..2].try_into().unwrap()) as usize;
    let mut pos = 2;

    if payload.len() < pos + key_len + 8 {
        return Err(ProtocolError::UnexpectedEof);
    }
    let key = std::str::from_utf8(&payload[pos..pos + key_len])
        .map_err(|_| ProtocolError::InvalidUtf8)?
        .to_string();
    pos += key_len;

    let ttl_ms = u64::from_be_bytes(payload[pos..pos + 8].try_into().unwrap());
    let ttl = if ttl_ms == 0 { None } else { Some(ttl_ms) };
    pos += 8;

    let (expected_value, consumed) = decode_value(&payload[pos..])?;
    pos += consumed;

    let (new_value, _) = decode_value(&payload[pos..])?;

    Ok((key, ttl, expected_value, new_value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_int() {
        let value = KvValue::Int(42);
        let encoded = encode_value(&value);
        let (decoded, _) = decode_value(&encoded).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_encode_decode_string() {
        let value = KvValue::String("hello".to_string());
        let encoded = encode_value(&value);
        let (decoded, _) = decode_value(&encoded).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_encode_decode_list() {
        let value = KvValue::List(vec![
            KvValue::Int(1),
            KvValue::String("two".to_string()),
            KvValue::Float(3.0),
        ]);
        let encoded = encode_value(&value);
        let (decoded, _) = decode_value(&encoded).unwrap();
        assert_eq!(value, decoded);
    }
}
