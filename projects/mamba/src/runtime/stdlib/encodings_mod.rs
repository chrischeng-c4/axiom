/// encodings package for Mamba (#1261 long-tail).
///
/// Replaces the long_tail `encodings` stub (which left
/// `normalize_encoding` returning `""` and `encodings.aliases.aliases`
/// as an empty dict) with the CPython algorithm and a populated
/// alias map.
///
/// CPython's `normalize_encoding`:
///   Runs of non-alphanumeric, non-`.` characters collapse to a single
///   `_`; leading punctuation is dropped (matches CPython's
///   `punct = False` initial state). The result is *not* lowercased —
///   `lookup()` does the lowercase pass on top.
///
/// We keep `encodings.utf_8.getregentry` and the `encodings.idna`
/// surface as no-ops because they need a real codec registry that
/// Mamba doesn't expose. The bits legacy probes actually read
/// (`normalize_encoding`, `aliases.aliases`) are real.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};
use super::super::dict_ops::DictKey;

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

/// Pure helper so we can unit-test the algorithm directly.
fn normalize_encoding_impl(s: &str) -> String {
    let mut chars: Vec<char> = Vec::with_capacity(s.len());
    let mut punct = false;
    for c in s.chars() {
        if c.is_alphanumeric() || c == '.' {
            if punct && !chars.is_empty() {
                chars.push('_');
            }
            chars.push(c);
            punct = false;
        } else {
            punct = true;
        }
    }
    chars.into_iter().collect()
}

unsafe extern "C" fn dispatch_normalize_encoding(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let input = args.first().copied().and_then(|v| as_str(v)).unwrap_or_default();
    MbValue::from_ptr(MbObject::new_str(normalize_encoding_impl(&input)))
}

unsafe extern "C" fn dispatch_search_function(_a: *const MbValue, _n: usize) -> MbValue {
    // Without a real codec registry we can't honor this. CPython returns None
    // for unknown encodings, so None is a reasonable shim.
    MbValue::none()
}

unsafe extern "C" fn dispatch_class_shell(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_empty_str(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

/// Common-subset of CPython's `encodings.aliases.aliases` map. The full
/// table has ~400 entries; we cover the families that legacy probe code
/// actually checks (UTF, ASCII, latin, common CJK, common 8859, MS code
/// pages most likely to surface in CPython library imports).
fn common_aliases() -> &'static [(&'static str, &'static str)] {
    &[
        // UTF family
        ("utf8",        "utf_8"),
        ("u8",          "utf_8"),
        ("utf",         "utf_8"),
        ("utf_8",       "utf_8"),
        ("utf8_ucs2",   "utf_8"),
        ("utf8_ucs4",   "utf_8"),
        ("utf16",       "utf_16"),
        ("u16",         "utf_16"),
        ("utf_16",      "utf_16"),
        ("utf_16be",    "utf_16_be"),
        ("utf_16le",    "utf_16_le"),
        ("utf32",       "utf_32"),
        ("u32",         "utf_32"),
        ("utf_32",      "utf_32"),
        ("utf_32be",    "utf_32_be"),
        ("utf_32le",    "utf_32_le"),
        ("utf_7",       "utf_7"),
        ("u7",          "utf_7"),
        ("unicode_1_1_utf_7", "utf_7"),
        // ASCII / 646
        ("ascii",       "ascii"),
        ("646",         "ascii"),
        ("ansi_x3.4_1968", "ascii"),
        ("ansi_x3_4_1968", "ascii"),
        ("ansi_x3.4_1986", "ascii"),
        ("cp367",       "ascii"),
        ("csascii",     "ascii"),
        ("ibm367",      "ascii"),
        ("iso646_us",   "ascii"),
        ("iso_646.irv_1991", "ascii"),
        ("iso_ir_6",    "ascii"),
        ("us",          "ascii"),
        ("us_ascii",    "ascii"),
        // Latin 1
        ("latin",       "latin_1"),
        ("latin1",      "latin_1"),
        ("latin_1",     "latin_1"),
        ("iso8859",     "latin_1"),
        ("iso8859_1",   "latin_1"),
        ("iso_8859_1",  "latin_1"),
        ("8859",        "latin_1"),
        ("cp819",       "latin_1"),
        ("csisolatin1", "latin_1"),
        ("ibm819",      "latin_1"),
        // ISO 8859 family
        ("iso8859_2",   "iso8859_2"),
        ("iso8859_3",   "iso8859_3"),
        ("iso8859_4",   "iso8859_4"),
        ("iso8859_5",   "iso8859_5"),
        ("iso8859_6",   "iso8859_6"),
        ("iso8859_7",   "iso8859_7"),
        ("iso8859_8",   "iso8859_8"),
        ("iso8859_9",   "iso8859_9"),
        ("iso8859_10",  "iso8859_10"),
        ("iso8859_13",  "iso8859_13"),
        ("iso8859_14",  "iso8859_14"),
        ("iso8859_15",  "iso8859_15"),
        ("iso8859_16",  "iso8859_16"),
        ("latin2",      "iso8859_2"),
        ("latin3",      "iso8859_3"),
        ("latin4",      "iso8859_4"),
        ("latin5",      "iso8859_9"),
        ("latin6",      "iso8859_10"),
        ("latin7",      "iso8859_13"),
        ("latin8",      "iso8859_14"),
        ("latin9",      "iso8859_15"),
        ("latin10",     "iso8859_16"),
        // CJK
        ("euc_jp",      "euc_jp"),
        ("eucjp",       "euc_jp"),
        ("u_jis",       "euc_jp"),
        ("ujis",        "euc_jp"),
        ("euc_kr",      "euc_kr"),
        ("euckr",       "euc_kr"),
        ("korean",      "euc_kr"),
        ("euc_cn",      "gb2312"),
        ("gb2312",      "gb2312"),
        ("chinese",     "gb2312"),
        ("csiso58gb231280", "gb2312"),
        ("gb18030",     "gb18030"),
        ("gbk",         "gbk"),
        ("936",         "gbk"),
        ("cp936",       "gbk"),
        ("ms936",       "gbk"),
        ("big5",        "big5"),
        ("csbig5",      "big5"),
        ("shift_jis",   "shift_jis"),
        ("shiftjis",    "shift_jis"),
        ("sjis",        "shift_jis"),
        ("s_jis",       "shift_jis"),
        ("csshiftjis",  "shift_jis"),
        // Windows / MS
        ("cp1250",      "cp1250"),
        ("windows_1250","cp1250"),
        ("cp1251",      "cp1251"),
        ("windows_1251","cp1251"),
        ("cp1252",      "cp1252"),
        ("windows_1252","cp1252"),
        ("cp1253",      "cp1253"),
        ("windows_1253","cp1253"),
        ("cp1254",      "cp1254"),
        ("windows_1254","cp1254"),
        ("cp1255",      "cp1255"),
        ("windows_1255","cp1255"),
        ("cp1256",      "cp1256"),
        ("windows_1256","cp1256"),
        ("cp1257",      "cp1257"),
        ("windows_1257","cp1257"),
        ("cp1258",      "cp1258"),
        ("windows_1258","cp1258"),
        // KOI8
        ("koi8_r",      "koi8_r"),
        ("koi8_t",      "koi8_t"),
        ("koi8_u",      "koi8_u"),
        // Mac
        ("mac_arabic",  "mac_arabic"),
        ("mac_croatian","mac_croatian"),
        ("mac_cyrillic","mac_cyrillic"),
        ("mac_farsi",   "mac_farsi"),
        ("mac_greek",   "mac_greek"),
        ("mac_iceland", "mac_iceland"),
        ("mac_latin2",  "mac_latin2"),
        ("mac_roman",   "mac_roman"),
        ("mac_turkish", "mac_turkish"),
    ]
}

pub fn register() {
    // encodings (top-level package)
    let mut attrs: HashMap<String, MbValue> = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("normalize_encoding", dispatch_normalize_encoding as *const () as usize),
        ("search_function",    dispatch_search_function    as *const () as usize),
    ];
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in dispatchers {
            set.insert(*addr as u64);
        }
    });
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    super::register_module("encodings", attrs);

    // encodings.aliases — real aliases dict
    let mut alias_attrs: HashMap<String, MbValue> = HashMap::new();
    let aliases_dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(rw) = &(*aliases_dict).data {
            let mut map = rw.write().unwrap();
            for (k, v) in common_aliases() {
                map.insert(
                    DictKey::Str((*k).into()),
                    MbValue::from_ptr(MbObject::new_str((*v).into())),
                );
            }
        }
    }
    alias_attrs.insert("aliases".into(), MbValue::from_ptr(aliases_dict));
    super::register_module("encodings.aliases", alias_attrs);

    // encodings.utf_8 — getregentry stub (no real codec registry).
    let mut utf8_attrs: HashMap<String, MbValue> = HashMap::new();
    let shell_addr = dispatch_class_shell as *const () as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| { s.borrow_mut().insert(shell_addr as u64); });
    utf8_attrs.insert("getregentry".into(), MbValue::from_func(shell_addr));
    super::register_module("encodings.utf_8", utf8_attrs);

    // encodings.idna — ToASCII/ToUnicode/nameprep stubs (return ""; the real
    // idna_mod also doesn't do anything useful yet).
    let mut idna_attrs: HashMap<String, MbValue> = HashMap::new();
    let empty_addr = dispatch_empty_str as *const () as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| { s.borrow_mut().insert(empty_addr as u64); });
    idna_attrs.insert("ToASCII".into(),  MbValue::from_func(empty_addr));
    idna_attrs.insert("ToUnicode".into(), MbValue::from_func(empty_addr));
    idna_attrs.insert("nameprep".into(),  MbValue::from_func(empty_addr));
    super::register_module("encodings.idna", idna_attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_simple_lowercase_unchanged() {
        assert_eq!(normalize_encoding_impl("utf_8"), "utf_8");
        assert_eq!(normalize_encoding_impl("ascii"), "ascii");
    }

    #[test]
    fn normalize_collapses_punctuation_to_underscore() {
        assert_eq!(normalize_encoding_impl("utf-8"), "utf_8");
        assert_eq!(normalize_encoding_impl("ISO-8859-1"), "ISO_8859_1");
        assert_eq!(normalize_encoding_impl("us-ascii"), "us_ascii");
    }

    #[test]
    fn normalize_runs_of_punctuation_collapse_to_one_underscore() {
        // CPython behaviour: runs of non-alnum collapse to a single `_`.
        assert_eq!(normalize_encoding_impl("utf--8"), "utf_8");
        assert_eq!(normalize_encoding_impl("utf  8"), "utf_8");
    }

    #[test]
    fn normalize_leading_punctuation_dropped() {
        // `punct = False` is initial, so `chars` is empty when the first
        // run of punctuation ends; the `if punct && !chars.is_empty()`
        // guard skips the leading `_`.
        assert_eq!(normalize_encoding_impl("--utf-8"), "utf_8");
    }

    #[test]
    fn normalize_preserves_dots() {
        // `.` is treated as alnum-equivalent by CPython, not punctuation.
        assert_eq!(normalize_encoding_impl("iso.8859.1"), "iso.8859.1");
    }

    #[test]
    fn normalize_empty_input() {
        assert_eq!(normalize_encoding_impl(""), "");
    }

    #[test]
    fn dispatch_handles_str_arg() {
        unsafe {
            let s = MbValue::from_ptr(MbObject::new_str("UTF-8".into()));
            let result = dispatch_normalize_encoding(&s as *const _, 1);
            let got = result.as_ptr().and_then(|p| match &(*p).data {
                ObjData::Str(s) => Some(s.clone()),
                _ => None,
            });
            assert_eq!(got.as_deref(), Some("UTF_8"));
        }
    }

    #[test]
    fn dispatch_handles_missing_arg() {
        unsafe {
            let result = dispatch_normalize_encoding(std::ptr::null(), 0);
            let got = result.as_ptr().and_then(|p| match &(*p).data {
                ObjData::Str(s) => Some(s.clone()),
                _ => None,
            });
            assert_eq!(got.as_deref(), Some(""));
        }
    }
}
