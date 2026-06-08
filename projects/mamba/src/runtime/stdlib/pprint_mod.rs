use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// pprint module for Mamba (#446).
///
/// Provides: pformat(obj), pprint(obj)
/// Pretty-prints nested data structures with indentation.
use std::collections::HashMap;

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

dispatch_unary!(dispatch_pprint, mb_pprint_pprint);
dispatch_unary!(dispatch_pformat, mb_pprint_pformat);

/// Register the pprint module.
pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("pprint", dispatch_pprint as usize),
        ("pformat", dispatch_pformat as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("pprint", attrs);
}

fn pformat_value(val: MbValue, indent: usize, depth: usize) -> String {
    let pad = " ".repeat(indent * depth);
    let inner = " ".repeat(indent * (depth + 1));

    if val.is_none() {
        return "None".to_string();
    }
    if let Some(b) = val.as_bool() {
        return if b { "True" } else { "False" }.to_string();
    }
    if let Some(i) = val.as_int() {
        return format!("{i}");
    }
    if let Some(f) = val.as_float() {
        return format!("{f}");
    }

    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => format!("'{s}'"),
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap();
                    if items.is_empty() {
                        "[]".to_string()
                    } else {
                        let parts: Vec<String> = items
                            .iter()
                            .map(|v| format!("{inner}{}", pformat_value(*v, indent, depth + 1)))
                            .collect();
                        format!("[\n{}\n{pad}]", parts.join(",\n"))
                    }
                }
                ObjData::Dict(ref lock) => {
                    let map = lock.read().unwrap();
                    if map.is_empty() {
                        "{}".to_string()
                    } else {
                        let parts: Vec<String> = map
                            .iter()
                            .map(|(k, v)| {
                                format!("{inner}'{k}': {}", pformat_value(*v, indent, depth + 1))
                            })
                            .collect();
                        format!("{{\n{}\n{pad}}}", parts.join(",\n"))
                    }
                }
                ObjData::Tuple(items) if items.is_empty() => "()".to_string(),
                ObjData::Tuple(items) => {
                    let parts: Vec<String> = items
                        .iter()
                        .map(|v| format!("{inner}{}", pformat_value(*v, indent, depth + 1)))
                        .collect();
                    if items.len() == 1 {
                        format!("(\n{},\n{pad})", parts[0].trim())
                    } else {
                        format!("(\n{}\n{pad})", parts.join(",\n"))
                    }
                }
                _ => format!("<object>"),
            }
        }
    } else {
        format!("<unknown>")
    }
}

/// pprint.pformat(obj) -> pretty-formatted string
pub fn mb_pprint_pformat(val: MbValue) -> MbValue {
    let s = pformat_value(val, 1, 0);
    MbValue::from_ptr(MbObject::new_str(s))
}

/// pprint.pprint(obj) -> print and return None
pub fn mb_pprint_pprint(val: MbValue) -> MbValue {
    let s = pformat_value(val, 1, 0);
    println!("{s}");
    MbValue::none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pformat_int() {
        let result = mb_pprint_pformat(MbValue::from_int(42));
        unsafe {
            if let ObjData::Str(ref s) = (*result.as_ptr().unwrap()).data {
                assert_eq!(s, "42");
            }
        }
    }

    #[test]
    fn test_pformat_list() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        let result = mb_pprint_pformat(list);
        unsafe {
            if let ObjData::Str(ref s) = (*result.as_ptr().unwrap()).data {
                assert!(s.contains("1"));
                assert!(s.contains("2"));
            }
        }
    }
}
