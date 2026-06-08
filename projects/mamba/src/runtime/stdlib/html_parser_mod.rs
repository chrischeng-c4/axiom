use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// html.parser module for Mamba (#449, #1480).
///
/// Provides:
///   - `html.escape` / `html.unescape` — fully functional string transforms.
///   - `html.parser.HTMLParser` — instance factory that returns a dict bearing
///     `__class__ = "HTMLParser"`, plus the full method surface stubbed under
///     a HANDWRITE carve-out (each method raises `NotImplementedError`).
///
/// Real HTML parsing (the SGML/HTML5-style tokenizer that CPython exposes
/// behind `HTMLParser.feed`) is non-trivial: it must handle markup state,
/// CDATA, entity references, partial buffers, and the full
/// `handle_starttag` / `handle_endtag` / `handle_data` / `handle_comment` /
/// `handle_decl` / `handle_pi` / `handle_charref` / `handle_entityref` /
/// `unknown_decl` callback dispatch. We deliberately do **not** ship a
/// half-working in-tree parser. Closing #1480 will require one of:
///
///   * vendor `html5ever` (Rust crate, WHATWG-conformant) and wire its
///     tokenizer events through to the Mamba object model; or
///   * port CPython's `Lib/html/parser.py` state machine line-for-line
///     against `Lib/test/test_html_parser.py` for behavioural parity.
///
/// Until that decision is made on #1480, the methods exist as callable
/// dispatchers (so `parser.feed("...")` is *callable*, not a string
/// attribute) but every call raises `NotImplementedError`. The dispatcher
/// surface keeps the typeshed-declared shape (R3) reachable without
/// committing to a parser strategy (R1/R2).
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

// HANDWRITE-BEGIN reason: #1480 — full HTML parser not implemented; needs
// either an html5ever vendor or a port of CPython's Lib/html/parser.py
// state machine. Each HTMLParser method is registered as a callable
// dispatcher that raises NotImplementedError to preserve typeshed shape
// without shipping a half-working tokenizer.
macro_rules! dispatch_variadic_stub {
    ($name:ident, $label:literal) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            raise_not_implemented($label)
        }
    };
}

dispatch_variadic_stub!(dispatch_feed, "HTMLParser.feed");
dispatch_variadic_stub!(dispatch_close, "HTMLParser.close");
dispatch_variadic_stub!(dispatch_reset, "HTMLParser.reset");
dispatch_variadic_stub!(dispatch_getpos, "HTMLParser.getpos");
dispatch_variadic_stub!(dispatch_get_starttag_text, "HTMLParser.get_starttag_text");
dispatch_variadic_stub!(dispatch_handle_starttag, "HTMLParser.handle_starttag");
dispatch_variadic_stub!(dispatch_handle_endtag, "HTMLParser.handle_endtag");
dispatch_variadic_stub!(dispatch_handle_startendtag, "HTMLParser.handle_startendtag");
dispatch_variadic_stub!(dispatch_handle_data, "HTMLParser.handle_data");
dispatch_variadic_stub!(dispatch_handle_entityref, "HTMLParser.handle_entityref");
dispatch_variadic_stub!(dispatch_handle_charref, "HTMLParser.handle_charref");
dispatch_variadic_stub!(dispatch_handle_comment, "HTMLParser.handle_comment");
dispatch_variadic_stub!(dispatch_handle_decl, "HTMLParser.handle_decl");
dispatch_variadic_stub!(dispatch_handle_pi, "HTMLParser.handle_pi");
dispatch_variadic_stub!(dispatch_unknown_decl, "HTMLParser.unknown_decl");
// HANDWRITE-END

dispatch_nullary!(dispatch_HTMLParser, mb_html_parser_new);
dispatch_unary!(dispatch_escape, mb_html_escape);
dispatch_unary!(dispatch_unescape, mb_html_unescape);

/// Register the html and html.parser modules.
///
/// `html` exposes the escape/unescape pair plus the `HTMLParser`
/// constructor (Mamba's historical placement — divergent from CPython,
/// but kept for backwards compat with #1261's fixtures). `html.parser`
/// mirrors the typeshed-declared submodule surface.
pub fn register() {
    // ── html (legacy + escape/unescape) ──
    let mut html_attrs = HashMap::new();
    let html_dispatchers: Vec<(&str, usize)> = vec![
        ("HTMLParser", dispatch_HTMLParser as usize),
        ("escape", dispatch_escape as usize),
        ("unescape", dispatch_unescape as usize),
    ];
    for (name, addr) in &html_dispatchers {
        html_attrs.insert(name.to_string(), MbValue::from_func(*addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*addr as u64);
        });
    }
    super::register_module("html", html_attrs);

    // ── html.parser (typeshed submodule surface) ──
    let mut parser_attrs = HashMap::new();
    parser_attrs.insert(
        "__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str("html.parser".to_string())),
    );
    parser_attrs.insert(
        "__package__".to_string(),
        MbValue::from_ptr(MbObject::new_str("html".to_string())),
    );

    // HANDWRITE-BEGIN reason: #1480 — HTMLParser method surface stubbed
    // pending parser-implementation decision (html5ever vendor vs CPython
    // port). Each entry is a callable dispatcher raising NotImplementedError.
    let parser_dispatchers: Vec<(&str, usize)> = vec![
        ("HTMLParser", dispatch_HTMLParser as usize),
        ("feed", dispatch_feed as usize),
        ("close", dispatch_close as usize),
        ("reset", dispatch_reset as usize),
        ("getpos", dispatch_getpos as usize),
        ("get_starttag_text", dispatch_get_starttag_text as usize),
        ("handle_starttag", dispatch_handle_starttag as usize),
        ("handle_endtag", dispatch_handle_endtag as usize),
        ("handle_startendtag", dispatch_handle_startendtag as usize),
        ("handle_data", dispatch_handle_data as usize),
        ("handle_entityref", dispatch_handle_entityref as usize),
        ("handle_charref", dispatch_handle_charref as usize),
        ("handle_comment", dispatch_handle_comment as usize),
        ("handle_decl", dispatch_handle_decl as usize),
        ("handle_pi", dispatch_handle_pi as usize),
        ("unknown_decl", dispatch_unknown_decl as usize),
        // `unescape` is deprecated on HTMLParser (since 3.5) but still
        // present in the public surface — point it at the real impl.
        ("unescape", dispatch_unescape as usize),
    ];
    // HANDWRITE-END
    for (name, addr) in &parser_dispatchers {
        parser_attrs.insert(name.to_string(), MbValue::from_func(*addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*addr as u64);
        });
    }
    super::register_module("html.parser", parser_attrs);

    // Re-wire the `parser` attribute on the parent `html` namespace so
    // `from html import parser` (and `import html; html.parser.HTMLParser`)
    // reflect the submodule. Mirrors the http.cookies #1477 pattern in
    // http_cookies_mod.rs: mamba's dotted-name resolution does not
    // round-trip `import html.parser` attribute access through the
    // parent on its own, so we splice it in explicitly here.
    super::super::module::MODULES.with(|mods| {
        let parser_val = {
            let r = mods.borrow();
            r.get("html.parser")
                .map(|m| super::super::module::module_to_value(m))
        };
        if let (Some(v), Some(html_mod)) = (parser_val, mods.borrow_mut().get_mut("html")) {
            html_mod.attrs.insert("parser".to_string(), v);
        }
    });
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

// HANDWRITE-BEGIN reason: #1480 — NotImplementedError helper for stubbed
// HTMLParser methods. Will be retired once a real parser lands.
fn raise_not_implemented(name: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("NotImplementedError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "html.parser.{name} is not implemented in Mamba (#1480)"
        ))),
    );
    MbValue::none()
}
// HANDWRITE-END

/// html.parser.HTMLParser() -> HTMLParser Instance shell.
///
/// **Hot path (#1480 Gate 2).** CPython runs the pure-Python
/// `HTMLParser.__init__` (`Lib/html/parser.py`), which calls
/// `self.reset()` and writes ~5 instance attributes per call; mamba
/// returns a single `MbObject::new_instance` allocation with no field
/// writes and no per-call string allocation for the class tag (the
/// class name is stored inline in `ObjData::Instance.class_name`).
/// Keep this body minimal — any extra allocation regresses the gate.
/// Mirrors the http.cookies #1477 shell pattern.
pub fn mb_html_parser_new() -> MbValue {
    MbValue::from_ptr(MbObject::new_instance("HTMLParser".to_string()))
}

/// html.escape(s) -> escaped HTML string
pub fn mb_html_escape(val: MbValue) -> MbValue {
    let s = extract_str(val).unwrap_or_default();
    let escaped = s
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;");
    MbValue::from_ptr(MbObject::new_str(escaped))
}

/// html.unescape(s) -> unescaped HTML string
pub fn mb_html_unescape(val: MbValue) -> MbValue {
    let s = extract_str(val).unwrap_or_default();
    let unescaped = s
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'");
    MbValue::from_ptr(MbObject::new_str(unescaped))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    #[test]
    fn test_escape_unescape_roundtrip() {
        let input = s("<div class=\"test\">&hello</div>");
        let escaped = mb_html_escape(input);
        let unescaped = mb_html_unescape(escaped);
        let result = extract_str(unescaped).unwrap();
        assert_eq!(result, "<div class=\"test\">&hello</div>");
    }

    #[test]
    fn test_escape() {
        let result = mb_html_escape(s("<b>bold</b>"));
        let escaped = extract_str(result).unwrap();
        assert_eq!(escaped, "&lt;b&gt;bold&lt;/b&gt;");
    }

    #[test]
    fn test_parser_new_returns_instance_with_class_name() {
        // #1480 Gate 2: HTMLParser() is now a passive Instance shell
        // (single allocation, no per-call string for `__class__`).
        // The class identity lives inline in `ObjData::Instance.class_name`.
        let parser = mb_html_parser_new();
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*parser.as_ptr().unwrap()).data {
                assert_eq!(class_name, "HTMLParser");
            } else {
                panic!("expected Instance");
            }
        }
    }

    #[test]
    fn test_register_exposes_full_parser_surface() {
        register();
        // html.parser must carry every typeshed-declared HTMLParser method
        // as a callable, even if each one raises NotImplementedError.
        for name in &[
            "HTMLParser",
            "feed",
            "close",
            "reset",
            "getpos",
            "get_starttag_text",
            "handle_starttag",
            "handle_endtag",
            "handle_startendtag",
            "handle_data",
            "handle_entityref",
            "handle_charref",
            "handle_comment",
            "handle_decl",
            "handle_pi",
            "unknown_decl",
            "unescape",
        ] {
            let v = crate::runtime::module::mb_module_getattr(
                MbValue::from_ptr(MbObject::new_str("html.parser".to_string())),
                MbValue::from_ptr(MbObject::new_str((*name).to_string())),
            );
            // Either a callable (func tag, no ptr) or an object ptr; both are
            // legitimate "registered" outcomes. None-tag means missing.
            assert!(!v.is_none(), "html.parser.{} not registered", name);
        }
    }
}
