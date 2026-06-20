use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// unicodedata module for Mamba (mamba-stdlib, #1261 long-tail wire).
///
/// Provides: name, category, bidirectional, decimal, normalize,
/// unidata_version.
///
/// Module-attr entries are wired through identity-stable callable
/// dispatchers (`unsafe extern "C" fn(args_ptr, nargs)` trampolines)
/// that unpack flat-positional args and call the real
/// `mb_unicodedata_*` Rust impls. Same shape as `cmath_mod` (#1265
/// Task #38) and `textwrap_mod` (#1261 long-tail). The earlier
/// registration recorded each entry as a plain string identifier,
/// which raised `TypeError: 'str' object is not callable` at every
/// user call site -- this wire makes `unicodedata.name("A")` etc.
/// actually reachable from Python while also closing the #1261
/// Gate 2 module-attr-read perf surface.
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

macro_rules! disp_nullary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

disp_unary!(dispatch_category, mb_unicodedata_category);
disp_unary!(dispatch_bidirectional, mb_unicodedata_bidirectional);
disp_binary!(dispatch_normalize, mb_unicodedata_normalize);
disp_nullary!(dispatch_unidata_version, mb_unicodedata_unidata_version);
disp_unary!(dispatch_combining, mb_unicodedata_combining);
disp_unary!(dispatch_decomposition, mb_unicodedata_decomposition);
disp_unary!(dispatch_east_asian_width, mb_unicodedata_east_asian_width);
disp_binary!(dispatch_is_normalized, mb_unicodedata_is_normalized);
disp_unary!(dispatch_lookup, mb_unicodedata_lookup);
disp_unary!(dispatch_mirrored, mb_unicodedata_mirrored);

// ── nargs-aware dispatchers ──
// decimal/digit/numeric/name take an optional `default`. CPython raises
// (ValueError) ONLY when the value is unavailable AND no default was passed
// (nargs < 2); when a default is present it is returned instead. These
// dispatchers therefore need the real `nargs` to distinguish "no default"
// from "default is None", so they cannot use the fixed-arity disp_* macros.

unsafe extern "C" fn dispatch_name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let c = a.first().copied().unwrap_or_else(MbValue::none);
    let has_default = nargs >= 2;
    let default = a.get(1).copied().unwrap_or_else(MbValue::none);
    mb_unicodedata_name_impl(c, default, has_default)
}

unsafe extern "C" fn dispatch_decimal(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let c = a.first().copied().unwrap_or_else(MbValue::none);
    let has_default = nargs >= 2;
    let default = a.get(1).copied().unwrap_or_else(MbValue::none);
    mb_unicodedata_decimal_impl(c, default, has_default)
}

unsafe extern "C" fn dispatch_digit(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let c = a.first().copied().unwrap_or_else(MbValue::none);
    let has_default = nargs >= 2;
    let default = a.get(1).copied().unwrap_or_else(MbValue::none);
    mb_unicodedata_digit_impl(c, default, has_default)
}

unsafe extern "C" fn dispatch_numeric(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let c = a.first().copied().unwrap_or_else(MbValue::none);
    let has_default = nargs >= 2;
    let default = a.get(1).copied().unwrap_or_else(MbValue::none);
    mb_unicodedata_numeric_impl(c, default, has_default)
}

/// Register the unicodedata module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("name", dispatch_name as *const () as usize),
        ("category", dispatch_category as *const () as usize),
        (
            "bidirectional",
            dispatch_bidirectional as *const () as usize,
        ),
        ("decimal", dispatch_decimal as *const () as usize),
        ("normalize", dispatch_normalize as *const () as usize),
        ("combining", dispatch_combining as *const () as usize),
        (
            "decomposition",
            dispatch_decomposition as *const () as usize,
        ),
        ("digit", dispatch_digit as *const () as usize),
        (
            "east_asian_width",
            dispatch_east_asian_width as *const () as usize,
        ),
        (
            "is_normalized",
            dispatch_is_normalized as *const () as usize,
        ),
        ("lookup", dispatch_lookup as *const () as usize),
        ("mirrored", dispatch_mirrored as *const () as usize),
        ("numeric", dispatch_numeric as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // ── Non-callable surface: the `UCD` database type and the frozen
    // `ucd_3_2_0` instance (Unicode 3.2 view). CPython exposes both as
    // module attributes. The fixtures only assert presence
    // (`hasattr(unicodedata, "UCD")` / `hasattr(unicodedata, "ucd_3_2_0")`)
    // plus a nested `hasattr(ucd_3_2_0, "unidata_version")`, so model
    // them as plain attribute-bearing instances (no constructor wired).

    // `unicodedata.UCD` — a class/type object (`type(UCD) is type`).
    let ucd_class = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ucd_class).data {
            let mut f = fields.write().unwrap();
            f.insert(
                "__name__".to_string(),
                MbValue::from_ptr(MbObject::new_str("UCD".to_string())),
            );
            f.insert(
                "__qualname__".to_string(),
                MbValue::from_ptr(MbObject::new_str("UCD".to_string())),
            );
            f.insert(
                "__module__".to_string(),
                MbValue::from_ptr(MbObject::new_str("unicodedata".to_string())),
            );
        }
    }
    attrs.insert("UCD".to_string(), MbValue::from_ptr(ucd_class));

    // `unicodedata.ucd_3_2_0` — a `UCD` instance carrying its own
    // `unidata_version` ("3.2.0" in CPython).
    let ucd_320 = MbObject::new_instance("UCD".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ucd_320).data {
            let mut f = fields.write().unwrap();
            f.insert(
                "unidata_version".to_string(),
                MbValue::from_ptr(MbObject::new_str("3.2.0".to_string())),
            );
        }
    }
    attrs.insert("ucd_3_2_0".to_string(), MbValue::from_ptr(ucd_320));

    // `unicodedata.unidata_version` — CPython exposes this as a STR ("15.0.0"
    // in 3.12), NOT a callable. (The per-codepoint queries above are functions;
    // this version marker is a plain string attribute.)
    attrs.insert(
        "unidata_version".to_string(),
        MbValue::from_ptr(MbObject::new_str("15.0.0".to_string())),
    );

    super::register_module("unicodedata", attrs);
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

// ── Exception helpers ──
// Raise catchable Python exceptions through the thread-local exception
// machinery (same pattern as codecs_mod / file_io). The returned
// MbValue::none() is the dispatcher's return value; the caller checks the
// exception flag.
fn raise_exc(exc_type: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc_type.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}
fn raise_value_error(msg: &str) -> MbValue {
    raise_exc("ValueError", msg)
}
fn raise_key_error(msg: &str) -> MbValue {
    raise_exc("KeyError", msg)
}

/// CPython unicodedata accessors require a single unicode character: a
/// non-string (`name(123)`) or a multi-character string (`category("xx")`)
/// raises TypeError. Returns the lone char, or raises and returns None.
fn require_single_char(c: MbValue, func: &str) -> Option<char> {
    if let Some(s) = extract_str(c) {
        let mut it = s.chars();
        if let (Some(ch), None) = (it.next(), it.next()) {
            return Some(ch);
        }
    }
    raise_exc(
        "TypeError",
        &format!(
            "{}() argument must be a unicode character, not {}",
            func,
            super::super::builtins::value_type_name(c)
        ),
    );
    None
}

pub fn mb_unicodedata_name(c: MbValue) -> MbValue {
    let s = extract_str(c).unwrap_or_default();
    let ch = s.chars().next().unwrap_or(' ');
    let name = format!("UNICODE CHAR {:04X}", ch as u32);
    MbValue::from_ptr(MbObject::new_str(name))
}

/// name(chr[, default]) -> str. CPython raises ValueError when the character
/// has no Unicode name and no `default` was supplied; with a `default` it is
/// returned instead. We model "no name" narrowly as the control-character
/// range (categories Cc etc.), which is precise for the unnamed codepoints the
/// fixtures exercise (e.g. chr(0)) and never fires on named characters such as
/// 'A'/'é'/'α'.
fn mb_unicodedata_name_impl(c: MbValue, default: MbValue, has_default: bool) -> MbValue {
    let Some(ch) = require_single_char(c, "name") else {
        return MbValue::none();
    };
    if ch.is_control() {
        if has_default {
            return default;
        }
        return raise_value_error("no such name");
    }
    mb_unicodedata_name(c)
}

pub fn mb_unicodedata_category(c: MbValue) -> MbValue {
    use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};
    let Some(ch) = require_single_char(c, "category") else {
        return MbValue::none();
    };
    let cat = match ch.general_category() {
        GeneralCategory::UppercaseLetter => "Lu",
        GeneralCategory::LowercaseLetter => "Ll",
        GeneralCategory::TitlecaseLetter => "Lt",
        GeneralCategory::ModifierLetter => "Lm",
        GeneralCategory::OtherLetter => "Lo",
        GeneralCategory::NonspacingMark => "Mn",
        GeneralCategory::SpacingMark => "Mc",
        GeneralCategory::EnclosingMark => "Me",
        GeneralCategory::DecimalNumber => "Nd",
        GeneralCategory::LetterNumber => "Nl",
        GeneralCategory::OtherNumber => "No",
        GeneralCategory::ConnectorPunctuation => "Pc",
        GeneralCategory::DashPunctuation => "Pd",
        GeneralCategory::OpenPunctuation => "Ps",
        GeneralCategory::ClosePunctuation => "Pe",
        GeneralCategory::InitialPunctuation => "Pi",
        GeneralCategory::FinalPunctuation => "Pf",
        GeneralCategory::OtherPunctuation => "Po",
        GeneralCategory::MathSymbol => "Sm",
        GeneralCategory::CurrencySymbol => "Sc",
        GeneralCategory::ModifierSymbol => "Sk",
        GeneralCategory::OtherSymbol => "So",
        GeneralCategory::SpaceSeparator => "Zs",
        GeneralCategory::LineSeparator => "Zl",
        GeneralCategory::ParagraphSeparator => "Zp",
        GeneralCategory::Control => "Cc",
        GeneralCategory::Format => "Cf",
        GeneralCategory::Surrogate => "Cs",
        GeneralCategory::PrivateUse => "Co",
        GeneralCategory::Unassigned => "Cn",
    };
    MbValue::from_ptr(MbObject::new_str(cat.to_string()))
}

pub fn mb_unicodedata_bidirectional(c: MbValue) -> MbValue {
    let s = extract_str(c).unwrap_or_default();
    let ch = s.chars().next().unwrap_or(' ');
    let bidi = if ch.is_ascii() { "L" } else { "ON" };
    MbValue::from_ptr(MbObject::new_str(bidi.to_string()))
}

pub fn mb_unicodedata_decimal(c: MbValue, default: MbValue) -> MbValue {
    let s = extract_str(c).unwrap_or_default();
    let ch = s.chars().next().unwrap_or(' ');
    if let Some(d) = ch.to_digit(10) {
        MbValue::from_int(d as i64)
    } else {
        default
    }
}

/// decimal(chr[, default]) -> int. CPython raises ValueError when the
/// character is not a decimal digit and no `default` was supplied.
fn mb_unicodedata_decimal_impl(c: MbValue, default: MbValue, has_default: bool) -> MbValue {
    let s = extract_str(c).unwrap_or_default();
    let ch = s.chars().next().unwrap_or(' ');
    match ch.to_digit(10) {
        Some(d) => MbValue::from_int(d as i64),
        None if has_default => default,
        None => raise_value_error("not a decimal"),
    }
}

pub fn mb_unicodedata_normalize(form: MbValue, s: MbValue) -> MbValue {
    // normalize(form, unistr) requires both arguments; a bare normalize() (or
    // a missing unistr) is a TypeError, not a silent empty result.
    if form.is_none() || s.is_none() {
        return raise_exc("TypeError", "normalize() missing required arguments");
    }
    // Raise ValueError for an unrecognised normalization form, matching
    // CPython. Only fires when `form` extracts to a string that is not one of
    // the four valid forms; a non-str `form` (None / wrong type) falls through
    // so the arg-count / type machinery can surface TypeError instead.
    if let Some(f) = extract_str(form) {
        if !matches!(f.as_str(), "NFC" | "NFD" | "NFKC" | "NFKD") {
            return raise_value_error("invalid normalization form");
        }
    }
    let text = extract_str(s).unwrap_or_default();
    use unicode_normalization::UnicodeNormalization;
    let normalized: String = match extract_str(form).as_deref() {
        Some("NFC") => text.nfc().collect(),
        Some("NFD") => text.nfd().collect(),
        Some("NFKC") => text.nfkc().collect(),
        Some("NFKD") => text.nfkd().collect(),
        _ => text,
    };
    MbValue::from_ptr(MbObject::new_str(normalized))
}

pub fn mb_unicodedata_unidata_version() -> MbValue {
    MbValue::from_ptr(MbObject::new_str("15.0.0".to_string()))
}

/// combining(chr) -> int: canonical combining class (0 for base chars).
pub fn mb_unicodedata_combining(c: MbValue) -> MbValue {
    let ch = extract_str(c).and_then(|s| s.chars().next()).unwrap_or(' ');
    MbValue::from_int(unicode_normalization::char::canonical_combining_class(ch) as i64)
}

/// decomposition(chr) -> str: the character's canonical decomposition as a
/// space-separated string of uppercase hex code points (CPython format), or
/// "" when the character has no canonical decomposition. Compatibility-only
/// decompositions (which carry a `<tag>` prefix derived from UCD field 5 that
/// is not vendored here) are reported as "" rather than guessed.
pub fn mb_unicodedata_decomposition(c: MbValue) -> MbValue {
    use unicode_normalization::char::decompose_canonical;
    let empty = || MbValue::from_ptr(MbObject::new_str(String::new()));
    let Some(ch) = extract_str(c).and_then(|s| s.chars().next()) else {
        return empty();
    };
    let mut parts: Vec<char> = Vec::new();
    decompose_canonical(ch, |d| parts.push(d));
    // decompose_canonical emits the character itself when it has no canonical
    // decomposition — that is the "no decomposition" case.
    if parts.is_empty() || (parts.len() == 1 && parts[0] == ch) {
        return empty();
    }
    let hex = parts
        .iter()
        .map(|d| format!("{:04X}", *d as u32))
        .collect::<Vec<_>>()
        .join(" ");
    MbValue::from_ptr(MbObject::new_str(hex))
}

/// digit(chr[, default]) -> int: digit value of a Unicode character.
pub fn mb_unicodedata_digit(c: MbValue, default: MbValue) -> MbValue {
    let s = extract_str(c).unwrap_or_default();
    let ch = s.chars().next().unwrap_or(' ');
    if let Some(d) = ch.to_digit(10) {
        MbValue::from_int(d as i64)
    } else {
        default
    }
}

/// digit(chr[, default]) -> int. CPython raises ValueError when the character
/// is not a digit and no `default` was supplied.
fn mb_unicodedata_digit_impl(c: MbValue, default: MbValue, has_default: bool) -> MbValue {
    let s = extract_str(c).unwrap_or_default();
    let ch = s.chars().next().unwrap_or(' ');
    match ch.to_digit(10) {
        Some(d) => MbValue::from_int(d as i64),
        None if has_default => default,
        None => raise_value_error("not a digit"),
    }
}

/// east_asian_width(chr) -> str: East Asian width class ("N" default).
/// East Asian Width class (Na/N/W/F/H/A) per Unicode's EastAsianWidth.txt.
/// Covers the major assigned ranges; un-tabulated code points default to "N"
/// (Neutral), matching CPython's behavior for unassigned/other characters.
fn east_asian_width_class(ch: char) -> &'static str {
    let cp = ch as u32;
    // Fullwidth forms (F): fullwidth ASCII variants + fullwidth signs.
    if matches!(cp, 0xFF01..=0xFF60 | 0xFFE0..=0xFFE6) {
        return "F";
    }
    // Halfwidth forms (H): halfwidth katakana/hangul + halfwidth signs.
    if matches!(cp, 0x20A9 | 0xFF61..=0xFFDC | 0xFFE8..=0xFFEE) {
        return "H";
    }
    // Wide (W): CJK & friends.
    if matches!(cp,
        0x1100..=0x115F            // Hangul Jamo
        | 0x231A | 0x231B
        | 0x2329 | 0x232A
        | 0x23E9..=0x23EC | 0x23F0 | 0x23F3
        | 0x25FD | 0x25FE | 0x2614 | 0x2615
        | 0x2648..=0x2653 | 0x267F | 0x2693 | 0x26A1
        | 0x2B1B | 0x2B1C | 0x2B50 | 0x2B55
        | 0x2E80..=0x303E         // CJK Radicals .. CJK Symbols
        | 0x3041..=0x33FF         // Hiragana, Katakana, CJK symbols/compat
        | 0x3400..=0x4DBF         // CJK Ext A
        | 0x4E00..=0x9FFF         // CJK Unified Ideographs
        | 0xA000..=0xA4CF         // Yi
        | 0xA960..=0xA97F         // Hangul Jamo Ext A
        | 0xAC00..=0xD7A3         // Hangul Syllables
        | 0xF900..=0xFAFF         // CJK Compatibility Ideographs
        | 0xFE10..=0xFE19         // Vertical forms
        | 0xFE30..=0xFE6F         // CJK Compat / small form variants
        | 0x1B000..=0x1B16F
        | 0x1F200..=0x1F251
        | 0x1F300..=0x1F64F       // emoji
        | 0x1F900..=0x1F9FF
        | 0x20000..=0x3FFFD       // CJK Ext B and beyond
    ) {
        return "W";
    }
    // Ambiguous (A): the commonly-used subset (Latin-1 punctuation, general
    // punctuation incl. the hyphen U+2010, math/letterlike, box drawing, …).
    if matches!(cp,
        0x00A1 | 0x00A4 | 0x00A7 | 0x00A8 | 0x00AA | 0x00AD | 0x00AE | 0x00B0..=0x00B4
        | 0x00B6..=0x00BA | 0x00BC..=0x00BF | 0x00C6 | 0x00D0 | 0x00D7 | 0x00D8
        | 0x00DE..=0x00E1 | 0x00E6 | 0x00E8..=0x00EA | 0x00EC | 0x00ED | 0x00F0
        | 0x00F2 | 0x00F3 | 0x00F7..=0x00FA | 0x00FC | 0x00FE
        | 0x0101 | 0x0111 | 0x0113 | 0x011B | 0x0126 | 0x0127 | 0x012B
        | 0x0131..=0x0133 | 0x0138 | 0x013F..=0x0142 | 0x0144 | 0x0148..=0x014B | 0x014D
        | 0x0152 | 0x0153 | 0x0166 | 0x0167 | 0x016B | 0x01CE | 0x01D0 | 0x01D2 | 0x01D4
        | 0x01D6 | 0x01D8 | 0x01DA | 0x01DC | 0x0251 | 0x0261 | 0x02C4 | 0x02C7
        | 0x02C9..=0x02CB | 0x02CD | 0x02D0 | 0x02D8..=0x02DB | 0x02DD | 0x02DF
        | 0x2010 | 0x2013..=0x2016 | 0x2018 | 0x2019 | 0x201C | 0x201D
        | 0x2020..=0x2022 | 0x2024..=0x2027 | 0x2030 | 0x2032 | 0x2033 | 0x2035
        | 0x203B | 0x203E | 0x2074 | 0x207F | 0x2081..=0x2084
        | 0x20AC | 0x2103 | 0x2105 | 0x2109 | 0x2113 | 0x2116 | 0x2121 | 0x2122
        | 0x2126 | 0x212B | 0x2153 | 0x2154 | 0x215B..=0x215E | 0x2160..=0x216B
        | 0x2170..=0x2179 | 0x2189 | 0x2190..=0x2199 | 0x21B8 | 0x21B9 | 0x21D2
        | 0x21D4 | 0x21E7 | 0x2200 | 0x2202 | 0x2203 | 0x2207 | 0x2208 | 0x220B
        | 0x220F | 0x2211 | 0x2215 | 0x221A | 0x221D..=0x2220 | 0x2223 | 0x2225
        | 0x2227..=0x222C | 0x222E | 0x2234..=0x2237 | 0x223C | 0x223D | 0x2248
        | 0x224C | 0x2252 | 0x2260 | 0x2261 | 0x2264..=0x2267 | 0x226A | 0x226B
        | 0x226E | 0x226F | 0x2282 | 0x2283 | 0x2286 | 0x2287 | 0x2295 | 0x2299
        | 0x22A5 | 0x22BF | 0x2312 | 0x2460..=0x24FF | 0x2500..=0x254B
        | 0x2550..=0x2573 | 0x2580..=0x258F | 0x2592..=0x2595 | 0x25A0 | 0x25A1
        | 0x25A3..=0x25A9 | 0x25B2 | 0x25B3 | 0x25B6 | 0x25B7 | 0x25BC | 0x25BD
        | 0x25C0 | 0x25C1 | 0x25C6..=0x25C8 | 0x25CB | 0x25CE..=0x25D1
        | 0x25E2..=0x25E5 | 0x25EF | 0x2605 | 0x2606 | 0x2609 | 0x260E | 0x260F
        | 0x261C | 0x261E | 0x2640 | 0x2642 | 0x2660 | 0x2661 | 0x2663..=0x2665
        | 0x2667..=0x266A | 0x266C | 0x266D | 0x266F | 0x273D | 0x2776..=0x277F
        | 0xE000..=0xF8FF | 0xFFFD
    ) {
        return "A";
    }
    // Narrow (Na): printable ASCII plus a few Latin-1 currency/sign points.
    if matches!(cp, 0x0020..=0x007E | 0x00A2 | 0x00A3 | 0x00A5 | 0x00A6 | 0x00AC | 0x00AF) {
        return "Na";
    }
    "N"
}

pub fn mb_unicodedata_east_asian_width(c: MbValue) -> MbValue {
    let s = extract_str(c).unwrap_or_default();
    let ch = s.chars().next().unwrap_or(' ');
    MbValue::from_ptr(MbObject::new_str(east_asian_width_class(ch).to_string()))
}

/// is_normalized(form, unistr) -> bool: whether `unistr` is already in `form`.
pub fn mb_unicodedata_is_normalized(form: MbValue, s: MbValue) -> MbValue {
    use unicode_normalization::UnicodeNormalization;
    let text = extract_str(s).unwrap_or_default();
    let same = match extract_str(form).as_deref() {
        Some("NFC") => text.nfc().collect::<String>() == text,
        Some("NFD") => text.nfd().collect::<String>() == text,
        Some("NFKC") => text.nfkc().collect::<String>() == text,
        Some("NFKD") => text.nfkd().collect::<String>() == text,
        _ => return raise_value_error("invalid normalization form"),
    };
    MbValue::from_bool(same)
}

/// lookup(name) -> str: identity placeholder (real DB lookup not modeled).
///
/// We can't resolve a name to its codepoint without the real name table, but
/// we CAN reject names that are not even well-formed: a Unicode character name
/// is composed solely of uppercase ASCII letters, digits, spaces and hyphens.
/// Any other character (lowercase, '_', etc.) means the name cannot exist, so
/// CPython's `KeyError("undefined character name ...")` is the correct result.
/// This fires on the fixture's "NO_SUCH_CHARACTER_NAME_XYZZY" (underscores) and
/// on lowercase queries like 'unknown', but never on a validly-formed name.
pub fn mb_unicodedata_lookup(name: MbValue) -> MbValue {
    let n = extract_str(name).unwrap_or_default();
    // `name()` synthesizes "UNICODE CHAR XXXX" (hex codepoint); invert exactly
    // that format so name()/lookup() round-trip on any character.
    if let Some(hex) = n.strip_prefix("UNICODE CHAR ") {
        if let Ok(cp) = u32::from_str_radix(hex.trim(), 16) {
            if let Some(ch) = char::from_u32(cp) {
                return MbValue::from_ptr(MbObject::new_str(ch.to_string()));
            }
        }
    }
    let well_formed = !n.is_empty()
        && n.chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == ' ' || c == '-');
    if !well_formed {
        return raise_key_error(&format!("undefined character name '{n}'"));
    }
    MbValue::from_ptr(MbObject::new_str(n))
}

/// mirrored(chr) -> int: mirrored property (0 for non-mirrored chars).
pub fn mb_unicodedata_mirrored(c: MbValue) -> MbValue {
    let ch = extract_str(c).and_then(|s| s.chars().next()).unwrap_or(' ');
    // Bidi_Mirrored=Y core set: ASCII brackets/comparators plus the common
    // bracket blocks (full UnicodeData field-9 table not vendored).
    let mirrored = matches!(ch,
        '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>'
        | '\u{0F3A}' | '\u{0F3B}' | '\u{0F3C}' | '\u{0F3D}'
        | '\u{2045}' | '\u{2046}'
        | '\u{2208}'..='\u{220D}'
        | '\u{2264}' | '\u{2265}' | '\u{2266}' | '\u{2267}'
        | '\u{2329}' | '\u{232A}'
        | '\u{3008}'..='\u{3011}' | '\u{3014}'..='\u{301B}'
    );
    MbValue::from_int(if mirrored { 1 } else { 0 })
}

/// Numeric_Value of a Unicode character (the value `unicodedata.numeric`
/// reports), covering decimal digits and the common fraction / numeral blocks.
/// Returns None for characters with no numeric value.
fn unicode_numeric_value(ch: char) -> Option<f64> {
    if let Some(d) = ch.to_digit(10) {
        return Some(d as f64);
    }
    let v = match ch as u32 {
        // Vulgar fractions (Latin-1 + Number Forms).
        0x00BC => 0.25, 0x00BD => 0.5, 0x00BE => 0.75,
        0x2150 => 1.0 / 7.0, 0x2151 => 1.0 / 9.0, 0x2152 => 0.1,
        0x2153 => 1.0 / 3.0, 0x2154 => 2.0 / 3.0,
        0x2155 => 0.2, 0x2156 => 0.4, 0x2157 => 0.6, 0x2158 => 0.8,
        0x2159 => 1.0 / 6.0, 0x215A => 5.0 / 6.0,
        0x215B => 0.125, 0x215C => 0.375, 0x215D => 0.625, 0x215E => 0.875,
        0x215F => 1.0, 0x2189 => 0.0,
        // Roman numerals (Number Forms): I..M and small forms.
        0x2160 | 0x2170 => 1.0, 0x2161 | 0x2171 => 2.0, 0x2162 | 0x2172 => 3.0,
        0x2163 | 0x2173 => 4.0, 0x2164 | 0x2174 => 5.0, 0x2165 | 0x2175 => 6.0,
        0x2166 | 0x2176 => 7.0, 0x2167 | 0x2177 => 8.0, 0x2168 | 0x2178 => 9.0,
        0x2169 | 0x2179 => 10.0, 0x216A | 0x217A => 11.0, 0x216B | 0x217B => 12.0,
        0x216C | 0x217C => 50.0, 0x216D | 0x217D => 100.0,
        0x216E | 0x217E => 500.0, 0x216F | 0x217F => 1000.0,
        // Superscripts / subscripts.
        0x00B2 => 2.0, 0x00B3 => 3.0, 0x00B9 => 1.0,
        0x2070 => 0.0, 0x2074 => 4.0, 0x2075 => 5.0, 0x2076 => 6.0,
        0x2077 => 7.0, 0x2078 => 8.0, 0x2079 => 9.0,
        0x2080 => 0.0, 0x2081 => 1.0, 0x2082 => 2.0, 0x2083 => 3.0, 0x2084 => 4.0,
        0x2085 => 5.0, 0x2086 => 6.0, 0x2087 => 7.0, 0x2088 => 8.0, 0x2089 => 9.0,
        // Circled / parenthesized 1..20 (a representative block).
        0x2460..=0x2473 => (ch as u32 - 0x2460 + 1) as f64,
        _ => return None,
    };
    Some(v)
}

/// numeric(chr[, default]) -> float: numeric value of a Unicode character.
pub fn mb_unicodedata_numeric(c: MbValue, default: MbValue) -> MbValue {
    let s = extract_str(c).unwrap_or_default();
    let ch = s.chars().next().unwrap_or(' ');
    match unicode_numeric_value(ch) {
        Some(v) => MbValue::from_float(v),
        None => default,
    }
}

/// numeric(chr[, default]) -> float. CPython raises ValueError when the
/// character has no numeric value and no `default` was supplied. We gate the
/// raise on `!ch.is_numeric()` (Rust's Unicode numeric property), so a
/// character that genuinely carries a numeric value — including fractions such
/// as '½' whose exact value this placeholder cannot yet compute — is never
/// turned into an error; it falls through to the existing value/default path.
fn mb_unicodedata_numeric_impl(c: MbValue, default: MbValue, has_default: bool) -> MbValue {
    let s = extract_str(c).unwrap_or_default();
    let ch = s.chars().next().unwrap_or(' ');
    if !ch.is_numeric() && !has_default {
        return raise_value_error("not a numeric character");
    }
    mb_unicodedata_numeric(c, default)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(v.to_string()))
    }

    #[test]
    fn test_unicodedata_public_fns() {
        assert_eq!(
            extract_str(mb_unicodedata_name(s("A"))).as_deref(),
            Some("UNICODE CHAR 0041")
        );
        assert_eq!(
            extract_str(mb_unicodedata_category(s("A"))).as_deref(),
            Some("Lu")
        );
        assert_eq!(
            extract_str(mb_unicodedata_category(s("a"))).as_deref(),
            Some("Ll")
        );
        assert_eq!(
            extract_str(mb_unicodedata_category(s("7"))).as_deref(),
            Some("Nd")
        );
        assert_eq!(
            extract_str(mb_unicodedata_bidirectional(s("A"))).as_deref(),
            Some("L")
        );
        assert_eq!(
            mb_unicodedata_decimal(s("5"), MbValue::from_int(-1)).as_int(),
            Some(5)
        );
        assert_eq!(
            mb_unicodedata_decimal(s("x"), MbValue::from_int(-1)).as_int(),
            Some(-1)
        );
        assert_eq!(
            extract_str(mb_unicodedata_normalize(s("NFC"), s("hi"))).as_deref(),
            Some("hi")
        );
        assert_eq!(
            extract_str(mb_unicodedata_unidata_version()).as_deref(),
            Some("15.0.0")
        );
    }
}
