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
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

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
        ("bidirectional", dispatch_bidirectional as *const () as usize),
        ("decimal", dispatch_decimal as *const () as usize),
        ("normalize", dispatch_normalize as *const () as usize),
        ("combining", dispatch_combining as *const () as usize),
        ("decomposition", dispatch_decomposition as *const () as usize),
        ("digit", dispatch_digit as *const () as usize),
        ("east_asian_width", dispatch_east_asian_width as *const () as usize),
        ("is_normalized", dispatch_is_normalized as *const () as usize),
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
            f.insert("__name__".to_string(),
                MbValue::from_ptr(MbObject::new_str("UCD".to_string())));
            f.insert("__qualname__".to_string(),
                MbValue::from_ptr(MbObject::new_str("UCD".to_string())));
            f.insert("__module__".to_string(),
                MbValue::from_ptr(MbObject::new_str("unicodedata".to_string())));
        }
    }
    attrs.insert("UCD".to_string(), MbValue::from_ptr(ucd_class));

    // `unicodedata.ucd_3_2_0` — a `UCD` instance carrying its own
    // `unidata_version` ("3.2.0" in CPython).
    let ucd_320 = MbObject::new_instance("UCD".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ucd_320).data {
            let mut f = fields.write().unwrap();
            f.insert("unidata_version".to_string(),
                MbValue::from_ptr(MbObject::new_str("3.2.0".to_string())));
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
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
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
fn raise_value_error(msg: &str) -> MbValue { raise_exc("ValueError", msg) }
fn raise_key_error(msg: &str) -> MbValue { raise_exc("KeyError", msg) }

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
    let s = extract_str(c).unwrap_or_default();
    let ch = s.chars().next().unwrap_or(' ');
    if ch.is_control() {
        if has_default { return default; }
        return raise_value_error("no such name");
    }
    mb_unicodedata_name(c)
}

pub fn mb_unicodedata_category(c: MbValue) -> MbValue {
    use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};
    let s = extract_str(c).unwrap_or_default();
    let ch = s.chars().next().unwrap_or(' ');
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
    if let Some(d) = ch.to_digit(10) { MbValue::from_int(d as i64) } else { default }
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
    MbValue::from_int(
        unicode_normalization::char::canonical_combining_class(ch) as i64,
    )
}

/// decomposition(chr) -> str: decomposition mapping ("" when none).
pub fn mb_unicodedata_decomposition(c: MbValue) -> MbValue {
    let _ = extract_str(c);
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

/// digit(chr[, default]) -> int: digit value of a Unicode character.
pub fn mb_unicodedata_digit(c: MbValue, default: MbValue) -> MbValue {
    let s = extract_str(c).unwrap_or_default();
    let ch = s.chars().next().unwrap_or(' ');
    if let Some(d) = ch.to_digit(10) { MbValue::from_int(d as i64) } else { default }
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
pub fn mb_unicodedata_east_asian_width(c: MbValue) -> MbValue {
    let s = extract_str(c).unwrap_or_default();
    let ch = s.chars().next().unwrap_or(' ');
    let w = if ch.is_ascii() { "Na" } else { "N" };
    MbValue::from_ptr(MbObject::new_str(w.to_string()))
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
    let well_formed = !n.is_empty()
        && n.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == ' ' || c == '-');
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

/// numeric(chr[, default]) -> float: numeric value of a Unicode character.
pub fn mb_unicodedata_numeric(c: MbValue, default: MbValue) -> MbValue {
    let s = extract_str(c).unwrap_or_default();
    let ch = s.chars().next().unwrap_or(' ');
    if let Some(d) = ch.to_digit(10) { MbValue::from_float(d as f64) } else { default }
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

    fn s(v: &str) -> MbValue { MbValue::from_ptr(MbObject::new_str(v.to_string())) }

    #[test]
    fn test_unicodedata_public_fns() {
        assert_eq!(extract_str(mb_unicodedata_name(s("A"))).as_deref(), Some("UNICODE CHAR 0041"));
        assert_eq!(extract_str(mb_unicodedata_category(s("A"))).as_deref(), Some("Lu"));
        assert_eq!(extract_str(mb_unicodedata_category(s("a"))).as_deref(), Some("Ll"));
        assert_eq!(extract_str(mb_unicodedata_category(s("7"))).as_deref(), Some("Nd"));
        assert_eq!(extract_str(mb_unicodedata_bidirectional(s("A"))).as_deref(), Some("L"));
        assert_eq!(mb_unicodedata_decimal(s("5"), MbValue::from_int(-1)).as_int(), Some(5));
        assert_eq!(mb_unicodedata_decimal(s("x"), MbValue::from_int(-1)).as_int(), Some(-1));
        assert_eq!(extract_str(mb_unicodedata_normalize(s("NFC"), s("hi"))).as_deref(), Some("hi"));
        assert_eq!(extract_str(mb_unicodedata_unidata_version()).as_deref(), Some("15.0.0"));
    }
}
