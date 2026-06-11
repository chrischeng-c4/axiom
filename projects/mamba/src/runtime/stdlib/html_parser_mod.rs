/// html.parser module for Mamba (#449, #1480).
///
/// Provides:
///   - `html.escape` / `html.unescape` — fully functional string transforms
///     (`unescape` implements CPython's `html.unescape` charref semantics:
///     numeric dec/hex references with the WHATWG invalid-charref remapping,
///     plus the common named entities with and without trailing `;`).
///   - `html.parser.HTMLParser` — a real tokenizer engine: a Rust port of
///     the state machine in CPython's `Lib/html/parser.py` (and the pieces
///     of `Lib/_markupbase.py` it uses). `HTMLParser` is registered as a
///     runtime class so user subclasses dispatch `feed`/`close`/`reset`/
///     `getpos`/`get_starttag_text` through the standard MRO, and the engine
///     dispatches every `handle_*` callback through
///     `class::mb_call_method`, which resolves subclass overrides first and
///     falls back to the registered no-op defaults.
///
/// Engine state lives on the parser instance as plain fields, mirroring
/// CPython's attribute names: `rawdata`, `lineno`, `offset`,
/// `convert_charrefs`, `cdata_elem`, `lasttag`, and the name-mangled
/// `_HTMLParser__starttag_text`.
///
/// The CPython regexes (`tagfind_tolerant`, `attrfind_tolerant`,
/// `locatestarttagend_tolerant`, `endtagfind`, `charref`, `entityref`,
/// `commentclose`, marked-section closers) are hand-ported as char-level
/// matchers with the same tolerant/backtracking behaviour, operating on a
/// `Vec<char>` so indices match Python string (code point) indices.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};
use super::super::dict_ops::DictKey;

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

// Module-level dispatchers for the typeshed-shaped `html.parser` surface.
// CPython exposes these only as HTMLParser *methods*; the module-level
// names are a Mamba-historical extra kept for shape compatibility. The
// real implementations are the class methods registered on the
// `HTMLParser` runtime class below — these module-level callables keep
// raising NotImplementedError because they have no `self` to operate on.
macro_rules! dispatch_variadic_stub {
    ($name:ident, $label:literal) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            raise_not_implemented($label)
        }
    };
}

dispatch_variadic_stub!(dispatch_feed,              "HTMLParser.feed");
dispatch_variadic_stub!(dispatch_close,             "HTMLParser.close");
dispatch_variadic_stub!(dispatch_reset,             "HTMLParser.reset");
dispatch_variadic_stub!(dispatch_getpos,            "HTMLParser.getpos");
dispatch_variadic_stub!(dispatch_get_starttag_text, "HTMLParser.get_starttag_text");
dispatch_variadic_stub!(dispatch_handle_starttag,   "HTMLParser.handle_starttag");
dispatch_variadic_stub!(dispatch_handle_endtag,     "HTMLParser.handle_endtag");
dispatch_variadic_stub!(dispatch_handle_startendtag,"HTMLParser.handle_startendtag");
dispatch_variadic_stub!(dispatch_handle_data,       "HTMLParser.handle_data");
dispatch_variadic_stub!(dispatch_handle_entityref,  "HTMLParser.handle_entityref");
dispatch_variadic_stub!(dispatch_handle_charref,    "HTMLParser.handle_charref");
dispatch_variadic_stub!(dispatch_handle_comment,    "HTMLParser.handle_comment");
dispatch_variadic_stub!(dispatch_handle_decl,       "HTMLParser.handle_decl");
dispatch_variadic_stub!(dispatch_handle_pi,         "HTMLParser.handle_pi");
dispatch_variadic_stub!(dispatch_unknown_decl,      "HTMLParser.unknown_decl");

dispatch_unary!(dispatch_escape, mb_html_escape);
dispatch_unary!(dispatch_unescape, mb_html_unescape);

/// `html.parser.HTMLParser(*, convert_charrefs=True)` constructor.
///
/// Keyword arguments arrive either as a trailing kwargs dict (method-call
/// lowering) or — for the bare-Ident constructor shape outside the
/// trailing-kwargs allowlist — flattened to a positional bool; both are
/// accepted, since `convert_charrefs` is the only constructor parameter.
#[allow(non_snake_case)]
unsafe extern "C" fn dispatch_HTMLParser(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args: &[MbValue] = if nargs == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let mut convert = true;
    for a in args {
        if let Some(b) = a.as_bool() {
            convert = b;
        } else if let Some(ptr) = a.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    if let Ok(g) = lock.read() {
                        if let Some(v) = g.get(&DictKey::Str("convert_charrefs".to_string())) {
                            convert = v
                                .as_bool()
                                .or_else(|| v.as_int().map(|x| x != 0))
                                .unwrap_or(true);
                        }
                    }
                }
            }
        }
    }
    let inst = mb_html_parser_new();
    init_state(inst, convert);
    inst
}

/// Register the html and html.parser modules.
///
/// `html` exposes the escape/unescape pair plus the `HTMLParser`
/// constructor (Mamba's historical placement — divergent from CPython,
/// but kept for backwards compat with #1261's fixtures). `html.parser`
/// mirrors the typeshed-declared submodule surface, and `HTMLParser` is
/// additionally registered as a runtime class so user subclasses inherit
/// the real tokenizer engine through normal MRO dispatch.
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
    parser_attrs.insert("__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str("html.parser".to_string())));
    parser_attrs.insert("__package__".to_string(),
        MbValue::from_ptr(MbObject::new_str("html".to_string())));

    let parser_dispatchers: Vec<(&str, usize)> = vec![
        ("HTMLParser",            dispatch_HTMLParser as usize),
        ("feed",                  dispatch_feed as usize),
        ("close",                 dispatch_close as usize),
        ("reset",                 dispatch_reset as usize),
        ("getpos",                dispatch_getpos as usize),
        ("get_starttag_text",     dispatch_get_starttag_text as usize),
        ("handle_starttag",       dispatch_handle_starttag as usize),
        ("handle_endtag",         dispatch_handle_endtag as usize),
        ("handle_startendtag",    dispatch_handle_startendtag as usize),
        ("handle_data",           dispatch_handle_data as usize),
        ("handle_entityref",      dispatch_handle_entityref as usize),
        ("handle_charref",        dispatch_handle_charref as usize),
        ("handle_comment",        dispatch_handle_comment as usize),
        ("handle_decl",           dispatch_handle_decl as usize),
        ("handle_pi",             dispatch_handle_pi as usize),
        ("unknown_decl",          dispatch_unknown_decl as usize),
        // `unescape` is deprecated on HTMLParser (since 3.5) but still
        // present in the public surface — point it at the real impl.
        ("unescape",              dispatch_unescape as usize),
    ];
    for (name, addr) in &parser_dispatchers {
        parser_attrs.insert(name.to_string(), MbValue::from_func(*addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*addr as u64);
        });
    }
    super::register_module("html.parser", parser_attrs);

    // `HTMLParser` as a runtime class: `class Rec(HTMLParser)` resolves the
    // textual base name against CLASS_REGISTRY, so subclass instances
    // inherit feed/close/reset/getpos/get_starttag_text and the no-op
    // handle_* defaults through standard MRO lookup (the unittest.TestCase
    // pattern).
    register_parser_class();

    // isinstance(x, HTMLParser) resolves the constructor func through
    // NATIVE_TYPE_NAMES into the registered class (MRO-aware).
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        m.borrow_mut().insert(dispatch_HTMLParser as usize as u64, "HTMLParser".to_string());
    });

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
        if let (Some(v), Some(html_mod)) = (
            parser_val,
            mods.borrow_mut().get_mut("html"),
        ) {
            html_mod.attrs.insert("parser".to_string(), v);
        }
    });
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

/// NotImplementedError helper for the module-level method-shaped stubs
/// (CPython has no module-level `feed` etc.; the class methods are real).
fn raise_not_implemented(name: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("NotImplementedError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            format!("html.parser.{name} is not implemented in Mamba (#1480)"),
        )),
    );
    MbValue::none()
}

/// html.parser.HTMLParser() -> HTMLParser Instance shell.
///
/// **Hot path (#1480 Gate 2).** Keep this body minimal — the class tag is
/// stored inline in `ObjData::Instance.class_name` with no per-call string
/// allocation. State-field initialization (`init_state`) is done by the
/// constructor dispatcher in a single fields-lock acquisition, mirroring
/// the ~5 attribute writes CPython's pure-Python `__init__` performs.
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

/// html.unescape(s) -> unescaped HTML string (CPython charref semantics).
pub fn mb_html_unescape(val: MbValue) -> MbValue {
    let s = extract_str(val).unwrap_or_default();
    MbValue::from_ptr(MbObject::new_str(unescape_str(&s)))
}

// ════════════════════════════════════════════════════════════════════════
//  html.unescape — charref replacement (port of html/__init__.py)
// ════════════════════════════════════════════════════════════════════════

/// Windows-1252 remapping for numeric charrefs (html._invalid_charrefs).
fn invalid_charref_char(num: u32) -> Option<&'static str> {
    Some(match num {
        0x00 => "\u{fffd}", 0x0d => "\r",       0x80 => "\u{20ac}", 0x81 => "\u{81}",
        0x82 => "\u{201a}", 0x83 => "\u{192}",  0x84 => "\u{201e}", 0x85 => "\u{2026}",
        0x86 => "\u{2020}", 0x87 => "\u{2021}", 0x88 => "\u{2c6}",  0x89 => "\u{2030}",
        0x8a => "\u{160}",  0x8b => "\u{2039}", 0x8c => "\u{152}",  0x8d => "\u{8d}",
        0x8e => "\u{17d}",  0x8f => "\u{8f}",   0x90 => "\u{90}",   0x91 => "\u{2018}",
        0x92 => "\u{2019}", 0x93 => "\u{201c}", 0x94 => "\u{201d}", 0x95 => "\u{2022}",
        0x96 => "\u{2013}", 0x97 => "\u{2014}", 0x98 => "\u{2dc}",  0x99 => "\u{2122}",
        0x9a => "\u{161}",  0x9b => "\u{203a}", 0x9c => "\u{153}",  0x9d => "\u{9d}",
        0x9e => "\u{17e}",  0x9f => "\u{178}",
        _ => return None,
    })
}

/// Codepoints numeric charrefs map to the empty string
/// (html._invalid_codepoints: controls + noncharacters).
fn is_invalid_codepoint(num: u32) -> bool {
    matches!(num,
        0x1..=0x8 | 0xb | 0xe..=0x1f | 0x7f..=0x9f | 0xfdd0..=0xfdef)
        || (num & 0xffff) == 0xfffe
        || (num & 0xffff) == 0xffff
}

fn numeric_charref_string(num_str: &str, hex: bool) -> String {
    let num = u32::from_str_radix(num_str, if hex { 16 } else { 10 })
        .unwrap_or(0x0011_0000); // overflow → out of range → U+FFFD
    if let Some(s) = invalid_charref_char(num) {
        return s.to_string();
    }
    if (0xd800..=0xdfff).contains(&num) || num > 0x0010_ffff {
        return "\u{fffd}".to_string();
    }
    if is_invalid_codepoint(num) {
        return String::new();
    }
    char::from_u32(num)
        .map(|c| c.to_string())
        .unwrap_or_else(|| "\u{fffd}".to_string())
}

/// Named-entity table: the HTML4 Latin-1 set (which the html5 dict carries
/// both with and without a trailing `;`) plus the common symbol entities
/// (`;`-terminated only, matching the html5 dict).
fn html5_lookup(s: &str) -> Option<&'static str> {
    Some(match s {
        "amp;" | "amp" | "AMP;" | "AMP" => "&",
        "lt;" | "lt" | "LT;" | "LT" => "<",
        "gt;" | "gt" | "GT;" | "GT" => ">",
        "quot;" | "quot" | "QUOT;" | "QUOT" => "\"",
        "apos;" => "'",
        "nbsp;" | "nbsp" => "\u{a0}",
        "iexcl;" | "iexcl" => "\u{a1}",
        "cent;" | "cent" => "\u{a2}",
        "pound;" | "pound" => "\u{a3}",
        "curren;" | "curren" => "\u{a4}",
        "yen;" | "yen" => "\u{a5}",
        "brvbar;" | "brvbar" => "\u{a6}",
        "sect;" | "sect" => "\u{a7}",
        "uml;" | "uml" => "\u{a8}",
        "copy;" | "copy" | "COPY;" | "COPY" => "\u{a9}",
        "ordf;" | "ordf" => "\u{aa}",
        "laquo;" | "laquo" => "\u{ab}",
        "not;" | "not" => "\u{ac}",
        "shy;" | "shy" => "\u{ad}",
        "reg;" | "reg" | "REG;" | "REG" => "\u{ae}",
        "macr;" | "macr" => "\u{af}",
        "deg;" | "deg" => "\u{b0}",
        "plusmn;" | "plusmn" => "\u{b1}",
        "sup2;" | "sup2" => "\u{b2}",
        "sup3;" | "sup3" => "\u{b3}",
        "acute;" | "acute" => "\u{b4}",
        "micro;" | "micro" => "\u{b5}",
        "para;" | "para" => "\u{b6}",
        "middot;" | "middot" => "\u{b7}",
        "cedil;" | "cedil" => "\u{b8}",
        "sup1;" | "sup1" => "\u{b9}",
        "ordm;" | "ordm" => "\u{ba}",
        "raquo;" | "raquo" => "\u{bb}",
        "frac14;" | "frac14" => "\u{bc}",
        "frac12;" | "frac12" => "\u{bd}",
        "frac34;" | "frac34" => "\u{be}",
        "iquest;" | "iquest" => "\u{bf}",
        "times;" | "times" => "\u{d7}",
        "divide;" | "divide" => "\u{f7}",
        "szlig;" | "szlig" => "\u{df}",
        "agrave;" | "agrave" => "\u{e0}",
        "aacute;" | "aacute" => "\u{e1}",
        "acirc;" | "acirc" => "\u{e2}",
        "atilde;" | "atilde" => "\u{e3}",
        "auml;" | "auml" => "\u{e4}",
        "aring;" | "aring" => "\u{e5}",
        "aelig;" | "aelig" => "\u{e6}",
        "ccedil;" | "ccedil" => "\u{e7}",
        "egrave;" | "egrave" => "\u{e8}",
        "eacute;" | "eacute" => "\u{e9}",
        "ecirc;" | "ecirc" => "\u{ea}",
        "euml;" | "euml" => "\u{eb}",
        "igrave;" | "igrave" => "\u{ec}",
        "iacute;" | "iacute" => "\u{ed}",
        "icirc;" | "icirc" => "\u{ee}",
        "iuml;" | "iuml" => "\u{ef}",
        "ntilde;" | "ntilde" => "\u{f1}",
        "ograve;" | "ograve" => "\u{f2}",
        "oacute;" | "oacute" => "\u{f3}",
        "ocirc;" | "ocirc" => "\u{f4}",
        "otilde;" | "otilde" => "\u{f5}",
        "ouml;" | "ouml" => "\u{f6}",
        "ugrave;" | "ugrave" => "\u{f9}",
        "uacute;" | "uacute" => "\u{fa}",
        "ucirc;" | "ucirc" => "\u{fb}",
        "uuml;" | "uuml" => "\u{fc}",
        "yacute;" | "yacute" => "\u{fd}",
        "yuml;" | "yuml" => "\u{ff}",
        "OElig;" => "\u{152}",
        "oelig;" => "\u{153}",
        "Scaron;" => "\u{160}",
        "scaron;" => "\u{161}",
        "Yuml;" => "\u{178}",
        "fnof;" => "\u{192}",
        "circ;" => "\u{2c6}",
        "tilde;" => "\u{2dc}",
        "ensp;" => "\u{2002}",
        "emsp;" => "\u{2003}",
        "thinsp;" => "\u{2009}",
        "zwnj;" => "\u{200c}",
        "zwj;" => "\u{200d}",
        "ndash;" => "\u{2013}",
        "mdash;" => "\u{2014}",
        "lsquo;" => "\u{2018}",
        "rsquo;" => "\u{2019}",
        "sbquo;" => "\u{201a}",
        "ldquo;" => "\u{201c}",
        "rdquo;" => "\u{201d}",
        "bdquo;" => "\u{201e}",
        "dagger;" => "\u{2020}",
        "Dagger;" => "\u{2021}",
        "bull;" => "\u{2022}",
        "hellip;" => "\u{2026}",
        "permil;" => "\u{2030}",
        "prime;" => "\u{2032}",
        "Prime;" => "\u{2033}",
        "lsaquo;" => "\u{2039}",
        "rsaquo;" => "\u{203a}",
        "oline;" => "\u{203e}",
        "frasl;" => "\u{2044}",
        "euro;" => "\u{20ac}",
        "trade;" | "TRADE;" => "\u{2122}",
        "larr;" => "\u{2190}",
        "uarr;" => "\u{2191}",
        "rarr;" => "\u{2192}",
        "darr;" => "\u{2193}",
        "harr;" => "\u{2194}",
        "forall;" => "\u{2200}",
        "part;" => "\u{2202}",
        "exist;" => "\u{2203}",
        "empty;" => "\u{2205}",
        "nabla;" => "\u{2207}",
        "isin;" => "\u{2208}",
        "notin;" => "\u{2209}",
        "ni;" => "\u{220b}",
        "prod;" => "\u{220f}",
        "sum;" => "\u{2211}",
        "minus;" => "\u{2212}",
        "lowast;" => "\u{2217}",
        "radic;" => "\u{221a}",
        "prop;" => "\u{221d}",
        "infin;" => "\u{221e}",
        "ang;" => "\u{2220}",
        "and;" => "\u{2227}",
        "or;" => "\u{2228}",
        "cap;" => "\u{2229}",
        "cup;" => "\u{222a}",
        "int;" => "\u{222b}",
        "there4;" => "\u{2234}",
        "sim;" => "\u{223c}",
        "cong;" => "\u{2245}",
        "asymp;" => "\u{2248}",
        "ne;" => "\u{2260}",
        "equiv;" => "\u{2261}",
        "le;" => "\u{2264}",
        "ge;" => "\u{2265}",
        "sub;" => "\u{2282}",
        "sup;" => "\u{2283}",
        "sube;" => "\u{2286}",
        "supe;" => "\u{2287}",
        "oplus;" => "\u{2295}",
        "otimes;" => "\u{2297}",
        "perp;" => "\u{22a5}",
        "sdot;" => "\u{22c5}",
        _ => return None,
    })
}

/// Port of `html.unescape`: replace `&(#[0-9]+;?|#[xX]hex+;?|name{1,32};?)`
/// left to right; named references fall back to the longest table prefix
/// (down to length 2), and total misses pass through unchanged.
pub fn unescape_str(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(s.len());
    let mut i = 0usize;
    while i < n {
        let c = chars[i];
        if c != '&' {
            out.push(c);
            i += 1;
            continue;
        }
        // numeric: &#[0-9]+;? or &#[xX][0-9a-fA-F]+;?
        if i + 1 < n && chars[i + 1] == '#' {
            let mut p = i + 2;
            let hex = p < n && matches!(chars[p], 'x' | 'X');
            if hex { p += 1; }
            let dstart = p;
            if hex {
                while p < n && chars[p].is_ascii_hexdigit() { p += 1; }
            } else {
                while p < n && chars[p].is_ascii_digit() { p += 1; }
            }
            if p > dstart {
                let num_str: String = chars[dstart..p].iter().collect();
                if p < n && chars[p] == ';' { p += 1; }
                out.push_str(&numeric_charref_string(&num_str, hex));
                i = p;
                continue;
            }
            out.push('&');
            i += 1;
            continue;
        }
        // named: [^\t\n\f <&#;]{1,32};?
        let mut p = i + 1;
        while p < n
            && p - (i + 1) < 32
            && !matches!(chars[p], '\t' | '\n' | '\x0c' | ' ' | '<' | '&' | '#' | ';')
        {
            p += 1;
        }
        if p == i + 1 {
            out.push('&');
            i += 1;
            continue;
        }
        let mut name: String = chars[i + 1..p].iter().collect();
        if p < n && chars[p] == ';' {
            name.push(';');
            p += 1;
        }
        if let Some(rep) = html5_lookup(&name) {
            out.push_str(rep);
            i = p;
            continue;
        }
        // longest-prefix fallback (html5 dict semantics)
        let nchars: Vec<char> = name.chars().collect();
        let mut matched = false;
        for x in (2..nchars.len()).rev() {
            let prefix: String = nchars[..x].iter().collect();
            if let Some(rep) = html5_lookup(&prefix) {
                out.push_str(rep);
                let rest: String = nchars[x..].iter().collect();
                out.push_str(&rest);
                i = p;
                matched = true;
                break;
            }
        }
        if !matched {
            out.push('&');
            out.push_str(&name);
            i = p;
        }
    }
    out
}

// ════════════════════════════════════════════════════════════════════════
//  Instance-state helpers
// ════════════════════════════════════════════════════════════════════════

fn name_val(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn inst_set(inst: MbValue, name: &str, value: MbValue) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.write().unwrap().insert(name.to_string(), value);
            }
        }
    }
}

fn inst_get(inst: MbValue, name: &str) -> Option<MbValue> {
    inst.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(name).copied()
        } else {
            None
        }
    })
}

fn str_field(inst: MbValue, name: &str) -> String {
    inst_get(inst, name).and_then(extract_str).unwrap_or_default()
}

fn opt_str_field(inst: MbValue, name: &str) -> Option<String> {
    inst_get(inst, name).and_then(|v| {
        if v.is_none() { None } else { extract_str(v) }
    })
}

fn int_field(inst: MbValue, name: &str, default: i64) -> i64 {
    inst_get(inst, name).and_then(|v| v.as_int()).unwrap_or(default)
}

fn bool_field(inst: MbValue, name: &str, default: bool) -> bool {
    inst_get(inst, name)
        .and_then(|v| v.as_bool().or_else(|| v.as_int().map(|i| i != 0)))
        .unwrap_or(default)
}

/// `HTMLParser.__init__` body: convert_charrefs + reset() state, written
/// under a single fields-lock acquisition (#1480 Gate 2).
fn init_state(inst: MbValue, convert: bool) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                f.insert("convert_charrefs".to_string(), MbValue::from_bool(convert));
                f.insert("_HTMLParser__starttag_text".to_string(), MbValue::none());
                f.insert("rawdata".to_string(), name_val(""));
                f.insert("lasttag".to_string(), name_val("???"));
                f.insert("cdata_elem".to_string(), MbValue::none());
                f.insert("lineno".to_string(), MbValue::from_int(1));
                f.insert("offset".to_string(), MbValue::from_int(0));
            }
        }
    }
}

/// `HTMLParser.reset` + `ParserBase.reset` (does not touch convert_charrefs).
fn reset_state(inst: MbValue) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                f.insert("rawdata".to_string(), name_val(""));
                f.insert("lasttag".to_string(), name_val("???"));
                f.insert("cdata_elem".to_string(), MbValue::none());
                f.insert("lineno".to_string(), MbValue::from_int(1));
                f.insert("offset".to_string(), MbValue::from_int(0));
            }
        }
    }
}

/// Lazy-init guard: if a subclass `__init__` never chained to the base
/// `__init__`, initialize the engine state on first use with defaults.
fn ensure_state(inst: MbValue) {
    if inst_get(inst, "rawdata").is_none() {
        let convert = bool_field(inst, "convert_charrefs", true);
        init_state(inst, convert);
    }
}

fn set_cdata_mode(inst: MbValue, elem: &str) {
    inst_set(inst, "cdata_elem", name_val(&elem.to_lowercase()));
}

fn clear_cdata_mode(inst: MbValue) {
    inst_set(inst, "cdata_elem", MbValue::none());
}

/// Dispatch `self.<name>(*args)` through the shared instance dispatcher so
/// subclass overrides + MRO are honoured (falls back to the registered
/// no-op defaults on the HTMLParser base class).
fn dispatch_handler(inst: MbValue, name: &str, args: Vec<MbValue>) -> MbValue {
    let arg_list = MbValue::from_ptr(MbObject::new_list(args));
    super::super::class::mb_call_method(inst, name_val(name), arg_list)
}

fn emit_data(inst: MbValue, s: &str) {
    dispatch_handler(inst, "handle_data", vec![name_val(s)]);
}

// ════════════════════════════════════════════════════════════════════════
//  Char-level matchers (ports of the parser.py / _markupbase.py regexes)
// ════════════════════════════════════════════════════════════════════════

fn is_space(c: char) -> bool {
    c.is_whitespace()
}

/// Tag-name continuation class: `[^\t\n\r\f />\x00]`.
fn tagname_char(c: char) -> bool {
    !matches!(c, '\t' | '\n' | '\r' | '\x0c' | ' ' | '/' | '>' | '\0')
}

fn cs(chars: &[char], a: usize, b: usize) -> String {
    let n = chars.len();
    let a2 = a.min(n);
    let b2 = b.min(n);
    if a2 >= b2 {
        return String::new();
    }
    chars[a2..b2].iter().collect()
}

fn starts_with(chars: &[char], i: usize, pat: &str) -> bool {
    let mut p = i;
    for c in pat.chars() {
        if p >= chars.len() || chars[p] != c {
            return false;
        }
        p += 1;
    }
    true
}

fn find_ch(chars: &[char], c: char, from: usize) -> Option<usize> {
    (from.min(chars.len())..chars.len()).find(|&p| chars[p] == c)
}

fn rfind_ch(chars: &[char], c: char, from: usize, to: usize) -> Option<usize> {
    (from..to.min(chars.len())).rev().find(|&p| chars[p] == c)
}

/// `tagfind_tolerant`: `([a-zA-Z][^\t\n\r\f />\x00]*)(?:\s|/(?!>))*`
/// Returns (group-1 tag name, match end).
fn match_tagfind(chars: &[char], i: usize) -> Option<(String, usize)> {
    let n = chars.len();
    if i >= n || !chars[i].is_ascii_alphabetic() {
        return None;
    }
    let mut p = i + 1;
    while p < n && tagname_char(chars[p]) {
        p += 1;
    }
    let name: String = chars[i..p].iter().collect();
    loop {
        if p < n && is_space(chars[p]) {
            p += 1;
        } else if p < n && chars[p] == '/' && !(p + 1 < n && chars[p + 1] == '>') {
            p += 1;
        } else {
            break;
        }
    }
    Some((name, p))
}

struct AttrMatch {
    name: String,
    /// `None` when there was no `=` value part (valueless attribute);
    /// `Some(raw)` carries the raw group-3 text (quotes included for the
    /// quoted forms, possibly empty for the bare form).
    value: Option<String>,
    end: usize,
}

/// `attrfind_tolerant`:
/// `((?<=['"\s/])[^\s/>][^\s/=>]*)(\s*=+\s*('[^']*'|"[^"]*"|(?!['"])[^>\s]*))?(?:\s|/(?!>))*`
fn match_attrfind(chars: &[char], pos: usize) -> Option<AttrMatch> {
    let n = chars.len();
    if pos == 0 || pos >= n {
        return None;
    }
    let prev = chars[pos - 1];
    if !(prev == '\'' || prev == '"' || prev == '/' || is_space(prev)) {
        return None; // lookbehind failed
    }
    let c0 = chars[pos];
    if is_space(c0) || c0 == '/' || c0 == '>' {
        return None;
    }
    let mut p = pos + 1;
    while p < n && !(is_space(chars[p]) || matches!(chars[p], '/' | '=' | '>')) {
        p += 1;
    }
    let name: String = chars[pos..p].iter().collect();
    let mut value: Option<String> = None;

    // optional value group: \s*=+\s*('...'|"..."|bare)
    {
        let mut q = p;
        while q < n && is_space(chars[q]) {
            q += 1;
        }
        let eq_start = q;
        while q < n && chars[q] == '=' {
            q += 1;
        }
        if q > eq_start {
            let after_eq = q;
            while q < n && is_space(chars[q]) {
                q += 1;
            }
            if q < n && (chars[q] == '\'' || chars[q] == '"') {
                let quote = chars[q];
                match (q + 1..n).find(|&r| chars[r] == quote) {
                    Some(r) => {
                        value = Some(chars[q..=r].iter().collect());
                        p = r + 1;
                    }
                    None => {
                        // regex backtrack: the second `\s*` gives everything
                        // back and the bare alternative matches empty right
                        // after the `=+`.
                        value = Some(String::new());
                        p = after_eq;
                    }
                }
            } else {
                let vstart = q;
                while q < n && chars[q] != '>' && !is_space(chars[q]) {
                    q += 1;
                }
                value = Some(chars[vstart..q].iter().collect());
                p = q;
            }
        }
    }
    // trailing (?:\s|/(?!>))*
    loop {
        if p < n && is_space(chars[p]) {
            p += 1;
        } else if p < n && chars[p] == '/' && !(p + 1 < n && chars[p + 1] == '>') {
            p += 1;
        } else {
            break;
        }
    }
    Some(AttrMatch { name, value, end: p })
}

/// `locatestarttagend_tolerant` (the verbose whole-start-tag matcher).
fn match_locatestarttagend(chars: &[char], i: usize) -> Option<usize> {
    let n = chars.len();
    if i + 1 >= n || chars[i] != '<' || !chars[i + 1].is_ascii_alphabetic() {
        return None;
    }
    let mut p = i + 2;
    while p < n && tagname_char(chars[p]) {
        p += 1;
    }
    // optional group: [\s/]* then attribute*
    let mut q = p;
    while q < n && (is_space(chars[q]) || chars[q] == '/') {
        q += 1;
    }
    loop {
        match match_attrfind(chars, q) {
            Some(m) if m.end > q => q = m.end,
            _ => break,
        }
    }
    p = q;
    // trailing \s*
    while p < n && is_space(chars[p]) {
        p += 1;
    }
    Some(p)
}

/// `endtagfind`: `</\s*([a-zA-Z][-.a-zA-Z0-9:_]*)\s*>`
fn match_endtagfind(chars: &[char], i: usize) -> Option<(String, usize)> {
    let n = chars.len();
    if i + 1 >= n || chars[i] != '<' || chars[i + 1] != '/' {
        return None;
    }
    let mut p = i + 2;
    while p < n && is_space(chars[p]) {
        p += 1;
    }
    if p >= n || !chars[p].is_ascii_alphabetic() {
        return None;
    }
    let start = p;
    p += 1;
    while p < n && (chars[p].is_ascii_alphanumeric() || matches!(chars[p], '-' | '.' | ':' | '_')) {
        p += 1;
    }
    let name: String = chars[start..p].iter().collect();
    while p < n && is_space(chars[p]) {
        p += 1;
    }
    if p < n && chars[p] == '>' {
        Some((name, p + 1))
    } else {
        None
    }
}

/// `charref`: `&#(?:[0-9]+|[xX][0-9a-fA-F]+)[^0-9a-fA-F]`
/// Returns (group()[2:-1], match end).
fn match_charref(chars: &[char], i: usize) -> Option<(String, usize)> {
    let n = chars.len();
    if i + 1 >= n || chars[i] != '&' || chars[i + 1] != '#' {
        return None;
    }
    let mut p = i + 2;
    let hex = p < n && matches!(chars[p], 'x' | 'X');
    if hex {
        p += 1;
    }
    let dstart = p;
    if hex {
        while p < n && chars[p].is_ascii_hexdigit() {
            p += 1;
        }
    } else {
        while p < n && chars[p].is_ascii_digit() {
            p += 1;
        }
    }
    if p == dstart || p >= n {
        return None;
    }
    // the terminator class is [^0-9a-fA-F] for BOTH alternatives, so a
    // decimal run followed by a hex letter cannot match at all.
    if chars[p].is_ascii_hexdigit() {
        return None;
    }
    let name: String = chars[i + 2..p].iter().collect();
    Some((name, p + 1))
}

/// `entityref`: `&([a-zA-Z][-.a-zA-Z0-9]*)[^a-zA-Z0-9]`
/// Returns (group 1, match end), honouring the one-char EOF backtrack.
fn match_entityref(chars: &[char], i: usize) -> Option<(String, usize)> {
    let n = chars.len();
    if i + 1 >= n || chars[i] != '&' || !chars[i + 1].is_ascii_alphabetic() {
        return None;
    }
    let mut p = i + 2;
    while p < n && (chars[p].is_ascii_alphanumeric() || matches!(chars[p], '-' | '.')) {
        p += 1;
    }
    if p < n {
        // chars[p] is outside the name class, hence non-alnum: terminator.
        let name: String = chars[i + 1..p].iter().collect();
        return Some((name, p + 1));
    }
    // EOF: the greedy `*` gives one char back — the terminator must still
    // be non-alnum, i.e. '-' or '.', and the name must keep >= 1 char.
    if p - (i + 1) >= 2 && matches!(chars[p - 1], '-' | '.') {
        let name: String = chars[i + 1..p - 1].iter().collect();
        return Some((name, p));
    }
    None
}

/// `incomplete`: `&[a-zA-Z#]` — caller guarantees chars[i] == '&'.
fn match_incomplete(chars: &[char], i: usize) -> bool {
    i + 1 < chars.len() && (chars[i + 1].is_ascii_alphabetic() || chars[i + 1] == '#')
}

/// `commentclose`: search `--\s*>`; returns (match start, match end).
fn search_comment_close(chars: &[char], from: usize) -> Option<(usize, usize)> {
    let n = chars.len();
    let mut j = from;
    while j + 1 < n {
        if chars[j] == '-' && chars[j + 1] == '-' {
            let mut k = j + 2;
            while k < n && is_space(chars[k]) {
                k += 1;
            }
            if k < n && chars[k] == '>' {
                return Some((j, k + 1));
            }
        }
        j += 1;
    }
    None
}

/// `_markedsectionclose` (`]\s*]\s*>`) / `_msmarkedsectionclose` (`]\s*>`).
fn search_marked_close(chars: &[char], from: usize, ms_style: bool) -> Option<(usize, usize)> {
    let n = chars.len();
    for j in from..n {
        if chars[j] != ']' {
            continue;
        }
        let mut k = j + 1;
        while k < n && is_space(chars[k]) {
            k += 1;
        }
        if !ms_style {
            if !(k < n && chars[k] == ']') {
                continue;
            }
            k += 1;
            while k < n && is_space(chars[k]) {
                k += 1;
            }
        }
        if k < n && chars[k] == '>' {
            return Some((j, k + 1));
        }
    }
    None
}

/// CDATA-mode `interesting`: search `</\s*<elem>\s*>` case-insensitively;
/// returns the match start.
fn search_cdata_close(chars: &[char], from: usize, elem: &str) -> Option<usize> {
    let n = chars.len();
    let el: Vec<char> = elem.chars().collect();
    for j in from..n {
        if chars[j] != '<' || j + 1 >= n || chars[j + 1] != '/' {
            continue;
        }
        let mut k = j + 2;
        while k < n && is_space(chars[k]) {
            k += 1;
        }
        if k + el.len() > n {
            continue;
        }
        let mut ok = true;
        for (t, &ec) in el.iter().enumerate() {
            if chars[k + t].to_ascii_lowercase() != ec.to_ascii_lowercase() {
                ok = false;
                break;
            }
        }
        if !ok {
            continue;
        }
        k += el.len();
        while k < n && is_space(chars[k]) {
            k += 1;
        }
        if k < n && chars[k] == '>' {
            return Some(j);
        }
    }
    None
}

/// `_markupbase._scan_name` (`[a-zA-Z][-_.a-zA-Z0-9]*\s*`): returns
/// (lowercased stripped name, match end); `None` for buffer-boundary
/// incompleteness (and for the malformed-input AssertionError path,
/// surfaced via `mb_raise`).
fn scan_decl_name(chars: &[char], i: usize) -> Option<(String, usize)> {
    let n = chars.len();
    if i >= n {
        return None;
    }
    if !chars[i].is_ascii_alphabetic() {
        // CPython: `self.error("expected name token at %r" % ...)` — and
        // ParserBase.error (not overridden since 3.5) raises
        // NotImplementedError with this fixed message.
        super::super::exception::mb_raise(
            name_val("NotImplementedError"),
            name_val("subclasses of ParserBase must override error()"),
        );
        return None;
    }
    let mut p = i + 1;
    while p < n && (chars[p].is_ascii_alphanumeric() || matches!(chars[p], '-' | '_' | '.')) {
        p += 1;
    }
    let name: String = chars[i..p].iter().collect();
    while p < n && is_space(chars[p]) {
        p += 1;
    }
    if p == n {
        return None; // match runs to end of buffer: incomplete
    }
    Some((name.to_lowercase(), p))
}

// ════════════════════════════════════════════════════════════════════════
//  Parser engine (port of HTMLParser/goahead + the parse_* helpers)
// ════════════════════════════════════════════════════════════════════════

/// `ParserBase.updatepos`: advance lineno/offset over chars[i..j].
fn updatepos(inst: MbValue, chars: &[char], i: usize, j: usize) -> usize {
    if i >= j {
        return j;
    }
    let nlines = chars[i..j].iter().filter(|&&c| c == '\n').count();
    if nlines > 0 {
        let lineno = int_field(inst, "lineno", 1) + nlines as i64;
        inst_set(inst, "lineno", MbValue::from_int(lineno));
        let pos = (i..j).rev().find(|&p| chars[p] == '\n').unwrap();
        inst_set(inst, "offset", MbValue::from_int((j - (pos + 1)) as i64));
    } else {
        let offset = int_field(inst, "offset", 0) + (j - i) as i64;
        inst_set(inst, "offset", MbValue::from_int(offset));
    }
    j
}

/// `HTMLParser.goahead` — handle data as far as reasonable; leaves
/// unconsumed input in `rawdata` for the next feed unless `end` forces
/// EOF semantics.
fn goahead(inst: MbValue, end: bool) {
    let rawdata = str_field(inst, "rawdata");
    let chars: Vec<char> = rawdata.chars().collect();
    let n = chars.len();
    let mut i: usize = 0;
    while i < n {
        let convert = bool_field(inst, "convert_charrefs", true);
        let cdata = opt_str_field(inst, "cdata_elem");
        let j: usize;
        if convert && cdata.is_none() {
            match find_ch(&chars, '<', i) {
                Some(p) => j = p,
                None => {
                    // No next '<': flush, unless a charref may be cut in
                    // half at the buffer end.
                    let from = i.max(n.saturating_sub(34));
                    if let Some(amppos) = rfind_ch(&chars, '&', from, n) {
                        if !chars[amppos..n].iter().any(|&c| is_space(c) || c == ';') {
                            break; // wait till we get all the text
                        }
                    }
                    j = n;
                }
            }
        } else if let Some(ref elem) = cdata {
            match search_cdata_close(&chars, i, elem) {
                Some(s) => j = s,
                None => break,
            }
        } else {
            match (i..n).find(|&p| chars[p] == '&' || chars[p] == '<') {
                Some(p) => j = p,
                None => j = n,
            }
        }
        if i < j {
            if convert && cdata.is_none() {
                emit_data(inst, &unescape_str(&cs(&chars, i, j)));
            } else {
                emit_data(inst, &cs(&chars, i, j));
            }
        }
        i = updatepos(inst, &chars, i, j);
        if i == n {
            break;
        }
        if chars[i] == '<' {
            let mut k: i64;
            if i + 1 < n && chars[i + 1].is_ascii_alphabetic() {
                k = parse_starttag(inst, &chars, i);
            } else if starts_with(&chars, i, "</") {
                k = parse_endtag(inst, &chars, i);
            } else if starts_with(&chars, i, "<!--") {
                k = parse_comment(inst, &chars, i, true);
            } else if starts_with(&chars, i, "<?") {
                k = parse_pi(inst, &chars, i);
            } else if starts_with(&chars, i, "<!") {
                k = parse_html_declaration(inst, &chars, i);
            } else if i + 1 < n {
                emit_data(inst, "<");
                k = (i + 1) as i64;
            } else {
                break;
            }
            if k < 0 {
                if !end {
                    break;
                }
                let k2 = match find_ch(&chars, '>', i + 1) {
                    Some(p) => p + 1,
                    None => match find_ch(&chars, '<', i + 1) {
                        Some(p) => p,
                        None => i + 1,
                    },
                };
                if convert && cdata.is_none() {
                    emit_data(inst, &unescape_str(&cs(&chars, i, k2)));
                } else {
                    emit_data(inst, &cs(&chars, i, k2));
                }
                k = k2 as i64;
            }
            i = updatepos(inst, &chars, i, k as usize);
        } else if starts_with(&chars, i, "&#") {
            if let Some((name, endm)) = match_charref(&chars, i) {
                dispatch_handler(inst, "handle_charref", vec![name_val(&name)]);
                let mut k = endm;
                if !(k >= 1 && chars[k - 1] == ';') {
                    k -= 1;
                }
                i = updatepos(inst, &chars, i, k);
                continue;
            } else {
                if chars[i..n].iter().any(|&c| c == ';') {
                    // bail by consuming &#
                    emit_data(inst, &cs(&chars, i, i + 2));
                    i = updatepos(inst, &chars, i, (i + 2).min(n));
                }
                break;
            }
        } else if chars[i] == '&' {
            if let Some((name, endm)) = match_entityref(&chars, i) {
                dispatch_handler(inst, "handle_entityref", vec![name_val(&name)]);
                let mut k = endm;
                if !(k >= 1 && chars[k - 1] == ';') {
                    k -= 1;
                }
                i = updatepos(inst, &chars, i, k);
                continue;
            }
            if match_incomplete(&chars, i) {
                if end && i + 2 == n {
                    // incomplete entity is the entire remaining input
                    i = updatepos(inst, &chars, i, i + 1);
                }
                break;
            } else if i + 1 < n {
                // not the end of the buffer, and can't be confused with
                // some other construct
                emit_data(inst, "&");
                i = updatepos(inst, &chars, i, i + 1);
            } else {
                break;
            }
        } else {
            // "interesting.search() lied" — defensive (unreachable)
            break;
        }
    }
    if end && i < n && opt_str_field(inst, "cdata_elem").is_none() {
        let convert = bool_field(inst, "convert_charrefs", true);
        if convert {
            emit_data(inst, &unescape_str(&cs(&chars, i, n)));
        } else {
            emit_data(inst, &cs(&chars, i, n));
        }
        i = updatepos(inst, &chars, i, n);
    }
    inst_set(inst, "rawdata", name_val(&cs(&chars, i, n)));
}

/// `check_for_whole_start_tag`: end position or -1 if incomplete.
fn check_for_whole_start_tag(chars: &[char], i: usize) -> i64 {
    let n = chars.len();
    let j = match match_locatestarttagend(chars, i) {
        Some(j) => j,
        None => return -1, // unreachable per the goahead starttagopen guard
    };
    if j >= n {
        return -1; // end of input
    }
    let next = chars[j];
    if next == '>' {
        return (j + 1) as i64;
    }
    if next == '/' {
        if j + 1 < n && chars[j + 1] == '>' {
            return (j + 2) as i64;
        }
        return -1; // buffer boundary
    }
    if next.is_ascii_alphabetic() || next == '=' {
        // end of input in or before attribute value
        return -1;
    }
    if j > i { j as i64 } else { (i + 1) as i64 }
}

/// `parse_starttag`: returns end position or -1 if not terminated.
fn parse_starttag(inst: MbValue, chars: &[char], i: usize) -> i64 {
    inst_set(inst, "_HTMLParser__starttag_text", MbValue::none());
    let endpos_i = check_for_whole_start_tag(chars, i);
    if endpos_i < 0 {
        return endpos_i;
    }
    let endpos = endpos_i as usize;
    inst_set(inst, "_HTMLParser__starttag_text", name_val(&cs(chars, i, endpos)));

    let mut attrs: Vec<MbValue> = Vec::new();
    let (tag_raw, mut k) = match match_tagfind(chars, i + 1) {
        Some(m) => m,
        None => {
            // unreachable per goahead's starttagopen guard; consume as
            // data to stay tolerant rather than asserting.
            emit_data(inst, &cs(chars, i, endpos));
            return endpos as i64;
        }
    };
    let tag = tag_raw.to_lowercase();
    inst_set(inst, "lasttag", name_val(&tag));

    while k < endpos {
        let m = match match_attrfind(chars, k) {
            Some(m) => m,
            None => break,
        };
        let mut attrvalue: Option<String> = match m.value {
            None => None,
            Some(raw) => {
                let vc: Vec<char> = raw.chars().collect();
                if vc.len() >= 2
                    && ((vc[0] == '\'' && vc[vc.len() - 1] == '\'')
                        || (vc[0] == '"' && vc[vc.len() - 1] == '"'))
                {
                    Some(vc[1..vc.len() - 1].iter().collect())
                } else {
                    Some(raw)
                }
            }
        };
        if let Some(ref v) = attrvalue {
            if !v.is_empty() {
                attrvalue = Some(unescape_str(v));
            }
        }
        let val_mb = match attrvalue {
            Some(v) => name_val(&v),
            None => MbValue::none(),
        };
        attrs.push(MbValue::from_ptr(MbObject::new_tuple(vec![
            name_val(&m.name.to_lowercase()),
            val_mb,
        ])));
        k = m.end;
    }

    let end_str = cs(chars, k.min(endpos), endpos);
    let end_trim = end_str.trim();
    if end_trim != ">" && end_trim != "/>" {
        emit_data(inst, &cs(chars, i, endpos));
        return endpos as i64;
    }
    let attrs_list = MbValue::from_ptr(MbObject::new_list(attrs));
    if end_trim == "/>" {
        // XHTML-style empty tag: <span attr="value" />
        dispatch_handler(inst, "handle_startendtag", vec![name_val(&tag), attrs_list]);
    } else {
        dispatch_handler(inst, "handle_starttag", vec![name_val(&tag), attrs_list]);
        if tag == "script" || tag == "style" {
            set_cdata_mode(inst, &tag);
        }
    }
    endpos as i64
}

/// `parse_endtag`: returns end position or -1 if incomplete.
fn parse_endtag(inst: MbValue, chars: &[char], i: usize) -> i64 {
    // endendtag: first '>' after `</`
    let gtpos = match find_ch(chars, '>', i + 1) {
        None => return -1,
        Some(p) => p + 1, // match.end()
    };
    if let Some((elem_raw, _end)) = match_endtagfind(chars, i) {
        let elem = elem_raw.to_lowercase();
        if let Some(ref ce) = opt_str_field(inst, "cdata_elem") {
            if &elem != ce {
                emit_data(inst, &cs(chars, i, gtpos));
                return gtpos as i64;
            }
        }
        dispatch_handler(inst, "handle_endtag", vec![name_val(&elem)]);
        clear_cdata_mode(inst);
        return gtpos as i64;
    }
    if opt_str_field(inst, "cdata_elem").is_some() {
        emit_data(inst, &cs(chars, i, gtpos));
        return gtpos as i64;
    }
    match match_tagfind(chars, i + 2) {
        None => {
            // w3.org/TR/html5/tokenization.html#end-tag-open-state
            if starts_with(chars, i, "</>") {
                (i + 3) as i64
            } else {
                parse_bogus_comment(inst, chars, i)
            }
        }
        Some((name_raw, nm_end)) => {
            let tagname = name_raw.to_lowercase();
            // consume and ignore other stuff between the name and the '>'
            let g = find_ch(chars, '>', nm_end);
            dispatch_handler(inst, "handle_endtag", vec![name_val(&tagname)]);
            match g {
                Some(p) => (p + 1) as i64,
                None => 0, // CPython: find() == -1 → return -1 + 1
            }
        }
    }
}

/// `_markupbase.parse_comment`.
fn parse_comment(inst: MbValue, chars: &[char], i: usize, report: bool) -> i64 {
    match search_comment_close(chars, i + 4) {
        None => -1,
        Some((start, endm)) => {
            if report {
                dispatch_handler(inst, "handle_comment", vec![name_val(&cs(chars, i + 4, start))]);
            }
            endm as i64
        }
    }
}

/// `parse_bogus_comment` (report=1 form).
fn parse_bogus_comment(inst: MbValue, chars: &[char], i: usize) -> i64 {
    match find_ch(chars, '>', i + 2) {
        None => -1,
        Some(pos) => {
            dispatch_handler(inst, "handle_comment", vec![name_val(&cs(chars, i + 2, pos))]);
            (pos + 1) as i64
        }
    }
}

/// `parse_pi`.
fn parse_pi(inst: MbValue, chars: &[char], i: usize) -> i64 {
    match find_ch(chars, '>', i + 2) {
        None => -1,
        Some(j) => {
            dispatch_handler(inst, "handle_pi", vec![name_val(&cs(chars, i + 2, j))]);
            (j + 1) as i64
        }
    }
}

/// `parse_html_declaration` (HTML5 markup-declaration-open state).
fn parse_html_declaration(inst: MbValue, chars: &[char], i: usize) -> i64 {
    let n = chars.len();
    if starts_with(chars, i, "<!--") {
        return parse_comment(inst, chars, i, true);
    }
    if starts_with(chars, i, "<![") {
        return parse_marked_section(inst, chars, i, true);
    }
    if i + 9 <= n {
        let head: String = chars[i..i + 9].iter().collect::<String>().to_lowercase();
        if head == "<!doctype" {
            return match find_ch(chars, '>', i + 9) {
                None => -1,
                Some(gtpos) => {
                    dispatch_handler(inst, "handle_decl", vec![name_val(&cs(chars, i + 2, gtpos))]);
                    (gtpos + 1) as i64
                }
            };
        }
    }
    parse_bogus_comment(inst, chars, i)
}

/// `_markupbase.parse_marked_section` (`<![CDATA[...]]>` etc.).
fn parse_marked_section(inst: MbValue, chars: &[char], i: usize, report: bool) -> i64 {
    let (sect_name, j) = match scan_decl_name(chars, i + 3) {
        Some(x) => x,
        None => return -1,
    };
    let close = match sect_name.as_str() {
        "temp" | "cdata" | "ignore" | "include" | "rcdata" => {
            search_marked_close(chars, i + 3, false)
        }
        "if" | "else" | "endif" => search_marked_close(chars, i + 3, true),
        _ => {
            // CPython: `self.error('unknown status keyword %r ...')` — and
            // ParserBase.error (not overridden since 3.5) raises
            // NotImplementedError with this fixed message.
            let _ = j;
            super::super::exception::mb_raise(
                name_val("NotImplementedError"),
                name_val("subclasses of ParserBase must override error()"),
            );
            return chars.len() as i64; // consume buffer after raising
        }
    };
    match close {
        None => -1,
        Some((start, endm)) => {
            if report {
                dispatch_handler(inst, "unknown_decl", vec![name_val(&cs(chars, i + 3, start))]);
            }
            endm as i64
        }
    }
}

// ════════════════════════════════════════════════════════════════════════
//  HTMLParser class methods (registered in the runtime class table)
// ════════════════════════════════════════════════════════════════════════

/// `HTMLParser.__init__(self, *, convert_charrefs=True)`. The second arg
/// carries either the trailing kwargs dict from `super().__init__(**kw)`
/// lowering or a positional bool (keyword flattening).
extern "C" fn hp_init(self_obj: MbValue, kw: MbValue) -> MbValue {
    let mut convert = true;
    if let Some(b) = kw.as_bool() {
        convert = b;
    } else if let Some(ptr) = kw.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                if let Ok(g) = lock.read() {
                    if let Some(v) = g.get(&DictKey::Str("convert_charrefs".to_string())) {
                        convert = v
                            .as_bool()
                            .or_else(|| v.as_int().map(|x| x != 0))
                            .unwrap_or(true);
                    }
                }
            }
        }
    }
    init_state(self_obj, convert);
    MbValue::none()
}

/// `HTMLParser.reset(self)` — loses all unprocessed data.
extern "C" fn hp_reset(self_obj: MbValue) -> MbValue {
    reset_state(self_obj);
    MbValue::none()
}

/// `HTMLParser.feed(self, data)` — buffer + tokenize incrementally.
extern "C" fn hp_feed(self_obj: MbValue, data: MbValue) -> MbValue {
    ensure_state(self_obj);
    let d = extract_str(data).unwrap_or_default();
    let buf = str_field(self_obj, "rawdata") + &d;
    inst_set(self_obj, "rawdata", name_val(&buf));
    goahead(self_obj, false);
    MbValue::none()
}

/// `HTMLParser.close(self)` — handle any buffered data as if at EOF.
extern "C" fn hp_close(self_obj: MbValue) -> MbValue {
    ensure_state(self_obj);
    goahead(self_obj, true);
    MbValue::none()
}

/// `ParserBase.getpos(self)` -> (lineno, offset) tuple.
extern "C" fn hp_getpos(self_obj: MbValue) -> MbValue {
    let lineno = int_field(self_obj, "lineno", 1);
    let offset = int_field(self_obj, "offset", 0);
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(lineno),
        MbValue::from_int(offset),
    ]))
}

/// `HTMLParser.get_starttag_text(self)` -> full source of the most recent
/// start tag, or None.
extern "C" fn hp_get_starttag_text(self_obj: MbValue) -> MbValue {
    inst_get(self_obj, "_HTMLParser__starttag_text").unwrap_or_else(MbValue::none)
}

/// Deprecated `HTMLParser.unescape(self, s)`.
extern "C" fn hp_unescape_method(_self_obj: MbValue, s: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(unescape_str(
        &extract_str(s).unwrap_or_default(),
    )))
}

// Overridable handler defaults (CPython: all no-ops except startendtag).
extern "C" fn hp_def_starttag(_self_obj: MbValue, _tag: MbValue, _attrs: MbValue) -> MbValue {
    MbValue::none()
}

extern "C" fn hp_def_endtag(_self_obj: MbValue, _tag: MbValue) -> MbValue {
    MbValue::none()
}

/// Default `handle_startendtag(tag, attrs)` decomposes into
/// starttag + endtag through the instance dispatcher so a subclass that
/// overrides only those two still sees `<br/>`.
extern "C" fn hp_def_startendtag(self_obj: MbValue, tag: MbValue, attrs: MbValue) -> MbValue {
    dispatch_handler(self_obj, "handle_starttag", vec![tag, attrs]);
    dispatch_handler(self_obj, "handle_endtag", vec![tag]);
    MbValue::none()
}

/// Shared no-op default for the unary handlers (data/comment/decl/pi/
/// entityref/charref/unknown_decl).
extern "C" fn hp_def_unary(_self_obj: MbValue, _arg: MbValue) -> MbValue {
    MbValue::none()
}

/// Register `HTMLParser` as a runtime class (the unittest.TestCase
/// pattern) so `class Rec(HTMLParser)` dispatches the engine through
/// the standard MRO and the engine reaches subclass overrides.
fn register_parser_class() {
    let mut methods: HashMap<String, MbValue> = HashMap::new();
    let m: &[(&str, usize)] = &[
        ("__init__",           hp_init as *const () as usize),
        ("feed",               hp_feed as *const () as usize),
        ("close",              hp_close as *const () as usize),
        ("reset",              hp_reset as *const () as usize),
        ("getpos",             hp_getpos as *const () as usize),
        ("get_starttag_text",  hp_get_starttag_text as *const () as usize),
        ("handle_starttag",    hp_def_starttag as *const () as usize),
        ("handle_startendtag", hp_def_startendtag as *const () as usize),
        ("handle_endtag",      hp_def_endtag as *const () as usize),
        ("handle_charref",     hp_def_unary as *const () as usize),
        ("handle_entityref",   hp_def_unary as *const () as usize),
        ("handle_data",        hp_def_unary as *const () as usize),
        ("handle_comment",     hp_def_unary as *const () as usize),
        ("handle_decl",        hp_def_unary as *const () as usize),
        ("handle_pi",          hp_def_unary as *const () as usize),
        ("unknown_decl",       hp_def_unary as *const () as usize),
        ("unescape",           hp_unescape_method as *const () as usize),
    ];
    for (name, addr) in m {
        methods.insert(name.to_string(), MbValue::from_func(*addr));
    }
    super::super::class::mb_class_register("HTMLParser", Vec::new(), methods);
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
    fn test_unescape_named_numeric_and_prefix() {
        assert_eq!(unescape_str("&amp; &lt; &gt; &quot; &apos;"), "& < > \" '");
        assert_eq!(unescape_str("&#65;&#66;&#x43;"), "ABC");
        // legacy semicolon-less + longest-prefix fallback
        assert_eq!(unescape_str("&ampx;"), "&x;");
        assert_eq!(unescape_str("a & b &"), "a & b &");
    }

    #[test]
    fn test_parser_new_returns_instance_with_class_name() {
        // #1480 Gate 2: HTMLParser() is a passive Instance shell
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
    fn test_engine_tokenizes_starttag_and_tracks_pos() {
        register();
        let p = mb_html_parser_new();
        init_state(p, true);
        hp_feed(p, s("<div class='x'>hello"));
        assert_eq!(str_field(p, "lasttag"), "div");
        assert_eq!(
            extract_str(hp_get_starttag_text(p)).as_deref(),
            Some("<div class='x'>")
        );
        assert_eq!(int_field(p, "lineno", 0), 1);
        assert_eq!(int_field(p, "offset", 0), 20);
        // buffer fully consumed
        assert_eq!(str_field(p, "rawdata"), "");
    }

    #[test]
    fn test_register_exposes_full_parser_surface() {
        register();
        // html.parser must carry every typeshed-declared HTMLParser method
        // as a callable.
        for name in &[
            "HTMLParser", "feed", "close", "reset", "getpos",
            "get_starttag_text", "handle_starttag", "handle_endtag",
            "handle_startendtag", "handle_data", "handle_entityref",
            "handle_charref", "handle_comment", "handle_decl",
            "handle_pi", "unknown_decl", "unescape",
        ] {
            let v = crate::runtime::module::mb_module_getattr(
                MbValue::from_ptr(MbObject::new_str("html.parser".to_string())),
                MbValue::from_ptr(MbObject::new_str((*name).to_string())),
            );
            // Either a callable (func tag, no ptr) or an object ptr; both are
            // legitimate "registered" outcomes. None-tag means missing.
            assert!(!v.is_none(),
                "html.parser.{} not registered", name);
        }
    }
}
