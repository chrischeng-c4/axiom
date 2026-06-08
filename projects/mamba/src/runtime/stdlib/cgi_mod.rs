use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// cgi module for Mamba (#1261 long-tail).
///
/// Replaces the long_tail stub which returned empty dicts/lists for all
/// the parsing helpers with real implementations of the pure-function
/// subset: `escape`, `parse_qs`, `parse_qsl`, `parse_header`.
///
/// The class-based API (`FieldStorage`, `MiniFieldStorage`, etc.) still
/// resolves to class shells, since those require file-IO and bound-method
/// dispatch the runtime doesn't yet support. The `print_*` family stays
/// as no-op shells; they only write to stdout in CPython.
///
/// `cgi` is deprecated in CPython 3.13+ but plenty of older code still
/// uses `cgi.escape` and `cgi.parse_header`, so it remains worth porting.
use std::collections::HashMap;

unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(args_ptr, nargs)
    }
}

unsafe fn as_str(val: MbValue) -> Option<String> {
    let ptr = val.as_ptr()?;
    match &(*ptr).data {
        ObjData::Str(s) => Some(s.clone()),
        ObjData::Bytes(b) => std::str::from_utf8(b).ok().map(str::to_string),
        _ => None,
    }
}

fn as_truthy(val: MbValue) -> bool {
    if let Some(b) = val.as_bool() {
        return b;
    }
    if let Some(i) = val.as_int() {
        return i != 0;
    }
    val.as_ptr().is_some()
}

/// HTML-escape `&`, `<`, `>`. If `quote` is True, also escape `"`.
fn html_escape(input: &str, quote: bool) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' if quote => out.push_str("&quot;"),
            other => out.push(other),
        }
    }
    out
}

unsafe extern "C" fn dispatch_escape(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(|v| as_str(v))
        .unwrap_or_default();
    let quote = args.get(1).copied().map(as_truthy).unwrap_or(false);
    MbValue::from_ptr(MbObject::new_str(html_escape(&s, quote)))
}

unsafe extern "C" fn dispatch_parse_qs(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    super::http_mod::mb_urllib_parse_qs(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_parse_qsl(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    super::http_mod::mb_urllib_parse_qsl(args.first().copied().unwrap_or_else(MbValue::none))
}

/// Tokenize a content-type-style header value on `;` boundaries, while
/// respecting double-quoted strings (`name="foo;bar"` is one token).
fn split_header_segments(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if in_quotes => {
                if let Some(&next) = chars.peek() {
                    cur.push(next);
                    chars.next();
                }
            }
            '"' => {
                in_quotes = !in_quotes;
                cur.push('"');
            }
            ';' if !in_quotes => {
                out.push(std::mem::take(&mut cur));
            }
            other => cur.push(other),
        }
    }
    if !cur.is_empty() || out.is_empty() {
        out.push(cur);
    }
    out
}

fn unquote_param(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        let inner = &trimmed[1..trimmed.len() - 1];
        let mut out = String::with_capacity(inner.len());
        let mut chars = inner.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(&next) = chars.peek() {
                    out.push(next);
                    chars.next();
                    continue;
                }
            }
            out.push(c);
        }
        out
    } else {
        trimmed.to_string()
    }
}

/// CPython `cgi.parse_header(line)` -> (main_value, params_dict).
/// `Content-Type: text/html; charset=utf-8; name="foo"` parses as
/// (`text/html`, `{charset: utf-8, name: foo}`). Parameter names are
/// lowercased; values are unquoted.
fn parse_header(line: &str) -> (String, Vec<(String, String)>) {
    let segments = split_header_segments(line);
    let main = segments
        .first()
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    let mut params = Vec::new();
    for seg in segments.iter().skip(1) {
        let seg = seg.trim();
        if seg.is_empty() {
            continue;
        }
        let (k, v) = match seg.find('=') {
            Some(idx) => (&seg[..idx], &seg[idx + 1..]),
            None => (seg, ""),
        };
        let key = k.trim().to_lowercase();
        let value = unquote_param(v);
        params.push((key, value));
    }
    (main, params)
}

unsafe extern "C" fn dispatch_parse_header(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let line = args
        .first()
        .copied()
        .and_then(|v| as_str(v))
        .unwrap_or_default();
    let (main, params) = parse_header(&line);
    let dict = MbObject::new_dict();
    if let ObjData::Dict(ref lock) = (*dict).data {
        let mut map = lock.write().unwrap();
        for (k, v) in params {
            let dk = super::super::dict_ops::DictKey::Str(k);
            map.insert(dk, MbValue::from_ptr(MbObject::new_str(v)));
        }
    }
    let tuple = MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_str(main)),
        MbValue::from_ptr(dict),
    ]);
    MbValue::from_ptr(tuple)
}

unsafe extern "C" fn dispatch_noop(_args: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn dispatch_empty_dict(_args: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_class_shell(_args: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

pub fn register() {
    let mut attrs: HashMap<String, MbValue> = HashMap::new();

    // Real pure-function ports.
    attrs.insert(
        "escape".into(),
        MbValue::from_func(dispatch_escape as *const () as usize),
    );
    attrs.insert(
        "parse_qs".into(),
        MbValue::from_func(dispatch_parse_qs as *const () as usize),
    );
    attrs.insert(
        "parse_qsl".into(),
        MbValue::from_func(dispatch_parse_qsl as *const () as usize),
    );
    attrs.insert(
        "parse_header".into(),
        MbValue::from_func(dispatch_parse_header as *const () as usize),
    );

    // Class shells (file-IO and bound-method dispatch not yet supported).
    for cls in [
        "FieldStorage",
        "MiniFieldStorage",
        "FormContentDict",
        "InterpFormContentDict",
        "FormContent",
        "SvFormContentDict",
    ] {
        attrs.insert(
            cls.into(),
            MbValue::from_func(dispatch_class_shell as *const () as usize),
        );
    }

    // Stub helpers (CPython prints them to stdout in CGI handlers).
    for name in [
        "parse",
        "parse_multipart",
        "test",
        "print_environ",
        "print_form",
        "print_directory",
        "print_arguments",
        "print_environ_usage",
    ] {
        let dispatch = if name == "parse" || name == "parse_multipart" {
            dispatch_empty_dict as *const () as usize
        } else {
            dispatch_noop as *const () as usize
        };
        attrs.insert(name.into(), MbValue::from_func(dispatch));
    }

    attrs.insert("maxlen".into(), MbValue::from_int(0));
    attrs.insert(
        "logfile".into(),
        MbValue::from_ptr(MbObject::new_str(String::new())),
    );
    attrs.insert(
        "logfp".into(),
        MbValue::from_ptr(MbObject::new_str(String::new())),
    );

    super::register_module("cgi", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_basic_chars() {
        assert_eq!(
            html_escape("<a href='x'>&", false),
            "&lt;a href='x'&gt;&amp;"
        );
    }

    #[test]
    fn escape_quote_flag() {
        assert_eq!(html_escape("\"hi\"", false), "\"hi\"");
        assert_eq!(html_escape("\"hi\"", true), "&quot;hi&quot;");
    }

    #[test]
    fn escape_passes_safe_chars() {
        let s = "abc 123 !? — ñ";
        assert_eq!(html_escape(s, true), s);
    }

    #[test]
    fn parse_header_simple() {
        let (main, params) = parse_header("text/html");
        assert_eq!(main, "text/html");
        assert!(params.is_empty());
    }

    #[test]
    fn parse_header_with_params() {
        let (main, params) = parse_header("text/html; charset=utf-8");
        assert_eq!(main, "text/html");
        assert_eq!(params, vec![("charset".to_string(), "utf-8".to_string())]);
    }

    #[test]
    fn parse_header_quoted_value() {
        let (main, params) = parse_header("attachment; filename=\"foo bar.txt\"");
        assert_eq!(main, "attachment");
        assert_eq!(
            params,
            vec![("filename".to_string(), "foo bar.txt".to_string())]
        );
    }

    #[test]
    fn parse_header_quoted_semicolon() {
        let (main, params) = parse_header("form-data; name=\"a;b\"; filename=foo");
        assert_eq!(main, "form-data");
        assert_eq!(
            params,
            vec![
                ("name".to_string(), "a;b".to_string()),
                ("filename".to_string(), "foo".to_string()),
            ]
        );
    }

    #[test]
    fn parse_header_lowercases_param_names() {
        let (_, params) = parse_header("text/plain; Charset=UTF-8");
        assert_eq!(params, vec![("charset".to_string(), "UTF-8".to_string())]);
    }

    #[test]
    fn parse_header_handles_escaped_quote() {
        let (_, params) = parse_header("form-data; name=\"he said \\\"hi\\\"\"");
        assert_eq!(
            params,
            vec![("name".to_string(), "he said \"hi\"".to_string())]
        );
    }

    #[test]
    fn parse_header_empty_string() {
        let (main, params) = parse_header("");
        assert_eq!(main, "");
        assert!(params.is_empty());
    }

    #[test]
    fn parse_header_trailing_semicolon() {
        let (main, params) = parse_header("text/html;");
        assert_eq!(main, "text/html");
        assert!(params.is_empty());
    }

    #[test]
    fn split_header_segments_basic() {
        let segs = split_header_segments("a; b=c; d=\"e;f\"");
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[2], " d=\"e;f\"");
    }

    #[test]
    fn unquote_param_strips_quotes() {
        assert_eq!(unquote_param("\"hi\""), "hi");
        assert_eq!(unquote_param("hi"), "hi");
        assert_eq!(unquote_param("  spaced  "), "spaced");
    }
}
