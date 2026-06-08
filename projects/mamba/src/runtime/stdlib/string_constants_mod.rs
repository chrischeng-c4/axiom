use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// string module for Mamba (#452).
///
/// Provides string constants: ascii_lowercase, ascii_uppercase, ascii_letters,
/// digits, hexdigits, octdigits, punctuation, whitespace, printable.
use std::collections::HashMap;

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

dispatch_nullary!(dispatch_Formatter, mb_string_formatter);
dispatch_unary!(dispatch_Template, mb_string_template);
dispatch_unary!(dispatch_capwords, mb_string_capwords);

/// Register the string module.
pub fn register() {
    let mut attrs = HashMap::new();

    attrs.insert(
        "ascii_lowercase".to_string(),
        MbValue::from_ptr(MbObject::new_str("abcdefghijklmnopqrstuvwxyz".to_string())),
    );
    attrs.insert(
        "ascii_uppercase".to_string(),
        MbValue::from_ptr(MbObject::new_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string())),
    );
    attrs.insert(
        "ascii_letters".to_string(),
        MbValue::from_ptr(MbObject::new_str(
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string(),
        )),
    );
    attrs.insert(
        "digits".to_string(),
        MbValue::from_ptr(MbObject::new_str("0123456789".to_string())),
    );
    attrs.insert(
        "hexdigits".to_string(),
        MbValue::from_ptr(MbObject::new_str("0123456789abcdefABCDEF".to_string())),
    );
    attrs.insert(
        "octdigits".to_string(),
        MbValue::from_ptr(MbObject::new_str("01234567".to_string())),
    );
    attrs.insert(
        "punctuation".to_string(),
        MbValue::from_ptr(MbObject::new_str(
            "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".to_string(),
        )),
    );
    attrs.insert(
        "whitespace".to_string(),
        MbValue::from_ptr(MbObject::new_str(" \t\n\r\x0b\x0c".to_string())),
    );

    // Callables — register as dispatched function values.
    let dispatchers: Vec<(&str, usize)> = vec![
        ("Formatter", dispatch_Formatter as usize),
        ("Template", dispatch_Template as usize),
        ("capwords", dispatch_capwords as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    super::register_module("string", attrs);
}

/// string.Formatter() -> dict-marker stub
pub fn mb_string_formatter() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut m = lock.write().unwrap();
            m.insert(
                "__class__".into(),
                MbValue::from_ptr(MbObject::new_str("Formatter".to_string())),
            );
        }
    }
    MbValue::from_ptr(dict)
}

/// `Template.substitute(mapping)` / `safe_substitute(mapping)` on the Template
/// dict-stub. Implements `$identifier`, `${identifier}`, and `$$` escapes.
/// `safe=false` (substitute): missing key → KeyError, lone/invalid `$` →
/// ValueError. `safe=true` (safe_substitute): leave missing/invalid literal.
pub fn mb_template_substitute(stub: MbValue, mapping: MbValue, safe: bool) -> MbValue {
    fn str_of(v: MbValue) -> Option<String> {
        v.as_ptr().and_then(|p| unsafe {
            if let ObjData::Str(ref s) = (*p).data {
                Some(s.clone())
            } else {
                None
            }
        })
    }
    // Template string from the stub's `template` field.
    let tmpl: String = stub
        .as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::Dict(ref lock) = (*p).data {
                lock.read()
                    .unwrap()
                    .get(&super::super::dict_ops::DictKey::Str(
                        "template".to_string(),
                    ))
                    .and_then(|v| str_of(*v))
            } else {
                None
            }
        })
        .unwrap_or_default();

    // Look up `name` in the mapping dict; returns its str() value or None.
    let lookup = |name: &str| -> Option<String> {
        mapping.as_ptr().and_then(|p| unsafe {
            if let ObjData::Dict(ref lock) = (*p).data {
                lock.read()
                    .unwrap()
                    .get(&super::super::dict_ops::DictKey::Str(name.to_string()))
                    .map(|v| {
                        let s = super::super::builtins::mb_str(*v);
                        str_of(s).unwrap_or_default()
                    })
            } else {
                None
            }
        })
    };

    let chars: Vec<char> = tmpl.chars().collect();
    let mut out = String::new();
    let mut i = 0usize;
    let is_id_start = |c: char| c.is_ascii_alphabetic() || c == '_';
    let is_id_cont = |c: char| c.is_ascii_alphanumeric() || c == '_';
    while i < chars.len() {
        if chars[i] != '$' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        // at '$'
        if i + 1 >= chars.len() {
            // trailing '$'
            if safe {
                out.push('$');
                i += 1;
                continue;
            }
            raise_value_error("Invalid placeholder in string: line 1, col 1");
            return MbValue::none();
        }
        let n = chars[i + 1];
        if n == '$' {
            out.push('$');
            i += 2;
            continue;
        }
        let (name, next) = if n == '{' {
            // ${name}
            let mut j = i + 2;
            let start = j;
            while j < chars.len() && chars[j] != '}' {
                j += 1;
            }
            if j >= chars.len() {
                if safe {
                    out.push_str(&chars[i..].iter().collect::<String>());
                    break;
                }
                raise_value_error("Invalid placeholder in string: line 1, col 1");
                return MbValue::none();
            }
            (chars[start..j].iter().collect::<String>(), j + 1)
        } else if is_id_start(n) {
            let mut j = i + 1;
            while j < chars.len() && is_id_cont(chars[j]) {
                j += 1;
            }
            (chars[i + 1..j].iter().collect::<String>(), j)
        } else {
            // '$' followed by a non-identifier char
            if safe {
                out.push('$');
                i += 1;
                continue;
            }
            raise_value_error("Invalid placeholder in string: line 1, col 1");
            return MbValue::none();
        };
        match lookup(&name) {
            Some(v) => {
                out.push_str(&v);
            }
            None => {
                if safe {
                    // keep the original placeholder text
                    out.push_str(&chars[i..next].iter().collect::<String>());
                } else {
                    // KeyError.__str__ adds the repr quoting itself; pass the
                    // bare key name to avoid double-quoting.
                    super::super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(name.clone())),
                    );
                    return MbValue::none();
                }
            }
        }
        i = next;
    }
    MbValue::from_ptr(MbObject::new_str(out))
}

fn raise_value_error(msg: &str) {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

/// string.Template(template) -> dict-marker stub
pub fn mb_string_template(template: MbValue) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut m = lock.write().unwrap();
            m.insert(
                "__class__".into(),
                MbValue::from_ptr(MbObject::new_str("Template".to_string())),
            );
            m.insert("template".into(), template);
        }
    }
    MbValue::from_ptr(dict)
}

/// string.capwords(s) — split, capitalize each word, rejoin
pub fn mb_string_capwords(val: MbValue) -> MbValue {
    use super::super::rc::ObjData;
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                let result: String = s
                    .split_whitespace()
                    .map(|w| {
                        let mut c = w.chars();
                        match c.next() {
                            Some(first) => {
                                let upper: String = first.to_uppercase().collect();
                                format!("{upper}{}", c.as_str().to_lowercase())
                            }
                            None => String::new(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                return MbValue::from_ptr(MbObject::new_str(result));
            }
        }
    }
    MbValue::none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capwords() {
        let s = MbValue::from_ptr(MbObject::new_str("hello world foo".to_string()));
        let result = mb_string_capwords(s);
        unsafe {
            if let super::super::super::rc::ObjData::Str(ref r) = (*result.as_ptr().unwrap()).data {
                assert_eq!(r, "Hello World Foo");
            }
        }
    }
}
