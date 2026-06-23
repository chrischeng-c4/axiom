//! @codegen-skip: handwrite-pre-standardize
//!
//! Reason: written before the stdlib-binding codegen vocabulary covers
//! the `attrs.insert(<name>, MbValue::from_func(...))` shape used here.
//! Will be converted to a CODEGEN-BEGIN/END block once the standardize
//! loop reaches Phase 2 stdlib bindings.

// HANDWRITE-BEGIN reason: stdlib binding emit rule not in primitive
// vocabulary yet — when the section type `stdlib-module-binding` is
// added the surface "kwlist + iskeyword" pair becomes a generator
// input rather than hand-written Rust.

//! @codegen-skip: handwrite-pre-standardize
//!
//! keyword module for Mamba (#690).
//!
//! Implements Python 3.12 `keyword` stdlib: kwlist, iskeyword, softkwlist,
//! issoftkeyword. All lists and membership checks match CPython 3.12 exactly.
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! shims (register_module + dispatch_unary + str-borrowed extract) is
//! not yet emitted by score codegen. Tracked as part of the brute-force
//! Phase-2 sweep; will be replaced when aw standardize lands the
//! stdlib-shim section type. Issue #1414 cluster anchor + see
//! `.aw/handoffs/1414-patrol-handoff.md`.
use super::super::rc::MbObject;
use super::super::value::MbValue;
use std::collections::HashMap;

/// Python 3.12 hard keywords — 35 total (authoritative list).
const KEYWORDS: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue",
    "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "if", "import",
    "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while",
    "with", "yield",
];

/// Python 3.12 soft keywords — 4 total, sorted alphabetically (matches CPython).
/// `type` is the PEP 695 type-alias soft keyword (`type X = int`).
const SOFT_KEYWORDS: &[&str] = &["_", "case", "match", "type"];

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

dispatch_unary!(dispatch_iskeyword, mb_keyword_iskeyword);
dispatch_unary!(dispatch_issoftkeyword, mb_keyword_issoftkeyword);

pub fn register() {
    let mut attrs = HashMap::new();

    // kwlist / softkwlist are *list attributes* in CPython, not callables.
    attrs.insert("kwlist".to_string(), mb_keyword_kwlist());
    attrs.insert("softkwlist".to_string(), mb_keyword_softkwlist());

    // iskeyword / issoftkeyword are unary callable dispatchers.
    let dispatchers: Vec<(&str, usize)> = vec![
        ("iskeyword", dispatch_iskeyword as usize),
        ("issoftkeyword", dispatch_issoftkeyword as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    super::register_module("keyword", attrs);
}

/// keyword.kwlist — returns the list of Python 3.12 hard keywords.
pub fn mb_keyword_kwlist() -> MbValue {
    let elements: Vec<MbValue> = KEYWORDS
        .iter()
        .map(|s| MbValue::from_ptr(MbObject::new_str(s.to_string())))
        .collect();
    MbValue::from_ptr(MbObject::new_list(elements))
}

/// keyword.iskeyword(s) -> bool
/// Returns True if s is a Python 3.12 hard keyword, False otherwise.
pub fn mb_keyword_iskeyword(s: MbValue) -> MbValue {
    let Some(s) = extract_str(s) else {
        return raise_type_error("keyword.iskeyword() argument must be str");
    };
    MbValue::from_bool(KEYWORDS.contains(&s.as_str()))
}

/// keyword.softkwlist — returns the list of Python 3.12 soft keywords.
pub fn mb_keyword_softkwlist() -> MbValue {
    let elements: Vec<MbValue> = SOFT_KEYWORDS
        .iter()
        .map(|s| MbValue::from_ptr(MbObject::new_str(s.to_string())))
        .collect();
    MbValue::from_ptr(MbObject::new_list(elements))
}

/// keyword.issoftkeyword(s) -> bool
/// Returns True if s is a Python 3.12 soft keyword, False otherwise.
pub fn mb_keyword_issoftkeyword(s: MbValue) -> MbValue {
    let Some(s) = extract_str(s) else {
        return raise_type_error("keyword.issoftkeyword() argument must be str");
    };
    MbValue::from_bool(SOFT_KEYWORDS.contains(&s.as_str()))
}

#[inline]
fn extract_str(val: MbValue) -> Option<String> {
    if let Some(ptr) = val.as_ptr() {
        use super::super::rc::ObjData;
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                return Some(s.clone());
            }
        }
    }
    None
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: R2
    #[test]
    fn test_iskeyword_true() {
        let if_val = MbValue::from_ptr(MbObject::new_str("if".to_string()));
        let def_val = MbValue::from_ptr(MbObject::new_str("def".to_string()));
        assert_eq!(mb_keyword_iskeyword(if_val).as_bool(), Some(true));
        assert_eq!(mb_keyword_iskeyword(def_val).as_bool(), Some(true));
    }

    // REQ: R2
    #[test]
    fn test_iskeyword_false() {
        let foo_val = MbValue::from_ptr(MbObject::new_str("foo".to_string()));
        let empty_val = MbValue::from_ptr(MbObject::new_str("".to_string()));
        assert_eq!(mb_keyword_iskeyword(foo_val).as_bool(), Some(false));
        assert_eq!(mb_keyword_iskeyword(empty_val).as_bool(), Some(false));
    }

    // REQ: R2
    #[test]
    fn test_issoftkeyword() {
        let match_val = MbValue::from_ptr(MbObject::new_str("match".to_string()));
        let if_val = MbValue::from_ptr(MbObject::new_str("if".to_string()));
        assert_eq!(mb_keyword_issoftkeyword(match_val).as_bool(), Some(true));
        assert_eq!(mb_keyword_issoftkeyword(if_val).as_bool(), Some(false));
    }

    // REQ: R2
    #[test]
    fn test_kwlist_length() {
        let list = mb_keyword_kwlist();
        let ptr = list.as_ptr().expect("kwlist must return a pointer");
        let len = unsafe {
            use super::super::super::rc::ObjData;
            if let ObjData::List(ref rw) = (*ptr).data {
                rw.read().unwrap().len()
            } else {
                panic!("kwlist must return a List")
            }
        };
        assert_eq!(len, 35, "CPython 3.12 has exactly 35 hard keywords");
    }

    // REQ: R2
    #[test]
    fn test_softkwlist_length() {
        let list = mb_keyword_softkwlist();
        let ptr = list.as_ptr().expect("softkwlist must return a pointer");
        let len = unsafe {
            use super::super::super::rc::ObjData;
            if let ObjData::List(ref rw) = (*ptr).data {
                rw.read().unwrap().len()
            } else {
                panic!("softkwlist must return a List")
            }
        };
        assert_eq!(
            len, 4,
            "CPython 3.12 has exactly 4 soft keywords (_, case, match, type)"
        );
    }
}
