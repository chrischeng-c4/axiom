use super::super::rc::{MbObject, ObjData};
use super::super::string_ops::{
    ascii_string_from_codepoints, surrogate_codepoints, value_to_string,
};
use super::super::value::MbValue;

/// ascii(obj) — return an ASCII-safe repr string.
/// Like repr() but escapes all non-ASCII characters as \xNN, \uNNNN, or \UNNNNNNNN.
pub fn mb_ascii(val: MbValue) -> MbValue {
    let s = ascii_repr(val);
    MbValue::from_ptr(MbObject::new_str(s))
}

fn ascii_repr(val: MbValue) -> String {
    if let Some(i) = val.as_int() {
        format!("{i}")
    } else if let Some(f) = val.as_float() {
        format!("{f}")
    } else if let Some(b) = val.as_bool() {
        (if b { "True" } else { "False" }).to_string()
    } else if val.is_none() {
        "None".to_string()
    } else if let Some(ptr) = val.as_ptr() {
        if let Some(codepoints) = surrogate_codepoints(val) {
            return ascii_string_from_codepoints(&codepoints);
        }
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => {
                    let escaped = escape_non_ascii(s);
                    format!("'{escaped}'")
                }
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap();
                    let parts: Vec<String> = items.iter().map(|v| ascii_repr(*v)).collect();
                    format!("[{}]", parts.join(", "))
                }
                ObjData::Tuple(items) => {
                    let parts: Vec<String> = items.iter().map(|v| ascii_repr(*v)).collect();
                    if items.len() == 1 {
                        format!("({},)", parts[0])
                    } else {
                        format!("({})", parts.join(", "))
                    }
                }
                _ => value_to_string(val),
            }
        }
    } else {
        String::new()
    }
}

fn escape_non_ascii(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        match c {
            '\'' => result.push_str("\\'"),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_ascii() && c >= ' ' => result.push(c),
            c if (c as u32) < 0x100 => result.push_str(&format!("\\x{:02x}", c as u32)),
            c if (c as u32) < 0x10000 => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push_str(&format!("\\U{:08x}", c as u32)),
        }
    }
    result
}
