use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// textwrap module for Mamba (#448, #1261 long-tail wire).
///
/// Provides: wrap, fill, dedent, indent, shorten.
///
/// Module-attr entries are wired through identity-stable callable
/// dispatchers (`unsafe extern "C" fn(args_ptr, nargs)` trampolines)
/// that unpack flat-positional args and call the real `mb_textwrap_*`
/// Rust impls. Same shape as `cmath_mod` (#1265 Task #38). The earlier
/// registration recorded each entry as a plain string identifier, which
/// raised `TypeError: 'str' object is not callable` at every user call
/// site -- this wire makes `textwrap.wrap("...", 70)` etc. actually
/// reachable from Python while also closing the #1261 Gate 2 module-
/// attr-read perf surface.
use std::collections::HashMap;

// ── Variadic dispatchers (callable from module-attr context) ──
// NOTE: dispatcher fn names must start with `dispatch_` so the surface
// walker (projects/mamba/src/surface.rs::pick_tuple_dispatcher) recognises
// them. Without the prefix Gate 3 surface scores 0/N.

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.first().copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! disp_binary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.first().copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

disp_binary!(dispatch_wrap, mb_textwrap_wrap);
disp_binary!(dispatch_fill, mb_textwrap_fill);
disp_unary!(dispatch_dedent, mb_textwrap_dedent);
disp_binary!(dispatch_indent, mb_textwrap_indent);
disp_binary!(dispatch_shorten, mb_textwrap_shorten);

/// Register the textwrap module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("wrap", dispatch_wrap as *const () as usize),
        ("fill", dispatch_fill as *const () as usize),
        ("dedent", dispatch_dedent as *const () as usize),
        ("indent", dispatch_indent as *const () as usize),
        ("shorten", dispatch_shorten as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    super::register_module("textwrap", attrs);
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// textwrap.wrap(text, width=70) -> list of lines
pub fn mb_textwrap_wrap(text: MbValue, width: MbValue) -> MbValue {
    let s = extract_str(text).unwrap_or_default();
    let w = width.as_int().unwrap_or(70) as usize;
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in s.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= w {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(MbValue::from_ptr(MbObject::new_str(current)));
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(MbValue::from_ptr(MbObject::new_str(current)));
    }
    MbValue::from_ptr(MbObject::new_list(lines))
}

/// textwrap.fill(text, width=70) -> single string with newlines
pub fn mb_textwrap_fill(text: MbValue, width: MbValue) -> MbValue {
    let s = extract_str(text).unwrap_or_default();
    let w = width.as_int().unwrap_or(70) as usize;
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in s.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= w {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    MbValue::from_ptr(MbObject::new_str(lines.join("\n")))
}

/// textwrap.dedent(text) -> remove common leading whitespace
pub fn mb_textwrap_dedent(text: MbValue) -> MbValue {
    let s = extract_str(text).unwrap_or_default();
    let lines: Vec<&str> = s.lines().collect();
    let min_indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    let result: String = lines
        .iter()
        .map(|l| {
            if l.len() >= min_indent {
                &l[min_indent..]
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    MbValue::from_ptr(MbObject::new_str(result))
}

/// textwrap.indent(text, prefix) -> add prefix to each line
pub fn mb_textwrap_indent(text: MbValue, prefix: MbValue) -> MbValue {
    let s = extract_str(text).unwrap_or_default();
    let p = extract_str(prefix).unwrap_or_default();
    let result: String = s
        .lines()
        .map(|l| format!("{p}{l}"))
        .collect::<Vec<_>>()
        .join("\n");
    MbValue::from_ptr(MbObject::new_str(result))
}

/// textwrap.shorten(text, width) -> truncate to width with [...]
pub fn mb_textwrap_shorten(text: MbValue, width: MbValue) -> MbValue {
    let s = extract_str(text).unwrap_or_default();
    let w = width.as_int().unwrap_or(70) as usize;
    if s.len() <= w {
        return MbValue::from_ptr(MbObject::new_str(s));
    }
    let placeholder = " [...]";
    if w <= placeholder.len() {
        return MbValue::from_ptr(MbObject::new_str("[...]".to_string()));
    }
    let truncated = &s[..w - placeholder.len()];
    // Cut at last word boundary
    let cut = truncated.rfind(' ').unwrap_or(truncated.len());
    let result = format!("{}{}", &truncated[..cut], placeholder);
    MbValue::from_ptr(MbObject::new_str(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    #[test]
    fn test_dedent() {
        let text = s("    hello\n    world");
        let result = mb_textwrap_dedent(text);
        unsafe {
            if let ObjData::Str(ref r) = (*result.as_ptr().unwrap()).data {
                assert_eq!(r, "hello\nworld");
            }
        }
    }

    #[test]
    fn test_indent() {
        let text = s("hello\nworld");
        let result = mb_textwrap_indent(text, s("  "));
        unsafe {
            if let ObjData::Str(ref r) = (*result.as_ptr().unwrap()).data {
                assert_eq!(r, "  hello\n  world");
            }
        }
    }

    #[test]
    fn test_wrap_fill_and_shorten() {
        let as_str = |v: MbValue| -> String {
            unsafe {
                if let ObjData::Str(ref r) = (*v.as_ptr().unwrap()).data {
                    r.clone()
                } else {
                    String::new()
                }
            }
        };
        // wrap: 3 words at width=5 → 3 single-word lines
        let wrapped = mb_textwrap_wrap(s("aaa bbb ccc"), MbValue::from_int(5));
        unsafe {
            if let ObjData::List(ref lock) = (*wrapped.as_ptr().unwrap()).data {
                assert_eq!(lock.read().unwrap().len(), 3);
            }
        }
        // fill: same input → newline-joined single string
        assert!(as_str(mb_textwrap_fill(s("aaa bbb ccc"), MbValue::from_int(5))).contains('\n'));
        // shorten: text longer than width → placeholder ' [...]' suffix
        assert!(as_str(mb_textwrap_shorten(
            s("one two three four five"),
            MbValue::from_int(12)
        ))
        .ends_with("[...]"));
    }
}
