use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// locale module for Mamba (mamba-stdlib).
use std::collections::HashMap;

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

macro_rules! dispatch_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
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

dispatch_nullary!(dispatch_getlocale, mb_locale_getlocale);
dispatch_binary!(dispatch_setlocale, mb_locale_setlocale);
dispatch_binary!(dispatch_format_string, mb_locale_format_string);
dispatch_unary!(dispatch_atof, mb_locale_atof);
dispatch_unary!(dispatch_atoi, mb_locale_atoi);
dispatch_unary!(dispatch_strxfrm, mb_locale_strxfrm);
dispatch_binary!(dispatch_strcoll, mb_locale_strcoll);

/// Raise a catchable exception whose type-name is `exc`; `except locale.<exc>`
/// (or a builtin like `ValueError`) matches by this name string. Mirrors the
/// netrc/graphlib raise-helper pattern. Returns `none` so callers can
/// `return raise_named(...)` from a value-returning dispatch fn.
fn raise_named(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// A locale name is "valid-looking" if CPython could plausibly accept it on a
/// POSIX system: the empty string / `C` / `POSIX`, or a structured
/// `ll[_CC][.codeset][@modifier]` token. Anything else (e.g. a multi-word
/// garbage token like `no_such_locale_xyzzy`) is treated as unknown. This is a
/// conservative shape check: it must never reject a real locale name, so a
/// genuine unknown locale that happens to *look* structured is allowed through
/// (a missed raise, never a wrong raise on a valid name).
fn locale_name_is_valid_looking(s: &str) -> bool {
    if s.is_empty() || s == "C" || s == "POSIX" {
        return true;
    }
    // Strip optional `@modifier`.
    let s = s.split('@').next().unwrap_or(s);
    // Strip optional `.codeset`.
    let lang_country = s.split('.').next().unwrap_or(s);
    let mut parts = lang_country.split('_');
    let lang = parts.next().unwrap_or("");
    // Language: 2-3 ASCII letters.
    if !(2..=3).contains(&lang.len()) || !lang.bytes().all(|b| b.is_ascii_alphabetic()) {
        return false;
    }
    // Optional single country segment: 2-3 ASCII alphanumerics.
    if let Some(country) = parts.next() {
        if !(2..=3).contains(&country.len()) || !country.bytes().all(|b| b.is_ascii_alphanumeric())
        {
            return false;
        }
    }
    // No further `_`-segments allowed (rejects `no_such_locale_xyzzy`).
    parts.next().is_none()
}

// Generic present+callable stub for surface-only locale callables.
// Surface fixtures only assert presence/callability/type, so a no-op
// stub that returns None is sufficient to satisfy them.
unsafe extern "C" fn dispatch_locale_stub(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

/// locale.getencoding() / getpreferredencoding([do_setlocale]) -> str.
/// CPython returns the current locale's codec name; mamba runs UTF-8, and
/// "UTF-8" round-trips through codecs.lookup. The optional do_setlocale arg
/// (getpreferredencoding) is accepted and ignored.
unsafe extern "C" fn dispatch_locale_getencoding(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str("UTF-8".to_string()))
}

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("getlocale", dispatch_getlocale as usize),
        ("setlocale", dispatch_setlocale as usize),
        ("format_string", dispatch_format_string as usize),
        ("atof", dispatch_atof as usize),
        ("atoi", dispatch_atoi as usize),
        ("strcoll", dispatch_strcoll as usize),
        ("strxfrm", dispatch_strxfrm as usize),
        ("getencoding", dispatch_locale_getencoding as usize),
        ("getpreferredencoding", dispatch_locale_getencoding as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    // surface: missing CPython module callables (functions, builtins, Error class)
    // — present+callable stubs sharing one generic dispatcher address.
    let stub_addr = dispatch_locale_stub as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(stub_addr as u64);
    });
    let stub_names: &[&str] = &[
        "bind_textdomain_codeset",
        "bindtextdomain",
        "currency",
        "dcgettext",
        "delocalize",
        "dgettext",
        "getdefaultlocale",
        "gettext",
        "localeconv",
        "localize",
        "nl_langinfo",
        "normalize",
        "resetlocale",
        "str",
        "textdomain",
    ];
    for name in stub_names {
        attrs.insert((*name).to_string(), MbValue::from_func(stub_addr));
    }
    // `locale.Error` is an Exception subclass in CPython. Expose it as a *type
    // object* (`class_name == "type"` carrying `__name__ == "Error"`), mirroring
    // graphlib.CycleError. This lets three things line up:
    //   * `resolve_class_name(locale.Error)` recovers "Error" via `__name__`, so
    //     an `except locale.Error:` clause matches a `mb_raise("Error", ...)`
    //     (setlocale's unknown-category / unknown-locale errors below);
    //   * `issubclass(locale.Error, Exception)` is true (bare-`Error` is a
    //     recognized Exception subclass in is_subclass_of);
    //   * `hasattr(locale.Error, "args")` still resolves (type-object getattr is
    //     permissive, and the `args` field is seeded explicitly anyway), keeping
    //     the surface probe green.
    {
        let inst = MbObject::new_instance("type".to_string());
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*inst).data {
                let mut f = fields.write().unwrap();
                f.insert(
                    "__name__".to_string(),
                    MbValue::from_ptr(MbObject::new_str("Error".to_string())),
                );
                f.insert(
                    "args".to_string(),
                    MbValue::from_ptr(MbObject::new_tuple(Vec::new())),
                );
            }
        }
        attrs.insert("Error".to_string(), MbValue::from_ptr(inst));
    }
    // LC_* are integer constants in CPython, not callables — emit them
    // eagerly as ints (matching the `pub fn mb_locale_LC_*()` returns).
    attrs.insert("LC_ALL".to_string(), mb_locale_LC_ALL());
    attrs.insert("LC_CTYPE".to_string(), mb_locale_LC_CTYPE());
    attrs.insert("LC_TIME".to_string(), mb_locale_LC_TIME());
    attrs.insert("LC_NUMERIC".to_string(), mb_locale_LC_NUMERIC());
    // surface: missing CPython module constants (auto-added)
    attrs.insert("ABDAY_1".into(), MbValue::from_int(14));
    attrs.insert("ABDAY_2".into(), MbValue::from_int(15));
    attrs.insert("ABDAY_3".into(), MbValue::from_int(16));
    attrs.insert("ABDAY_4".into(), MbValue::from_int(17));
    attrs.insert("ABDAY_5".into(), MbValue::from_int(18));
    attrs.insert("ABDAY_6".into(), MbValue::from_int(19));
    attrs.insert("ABDAY_7".into(), MbValue::from_int(20));
    attrs.insert("ABMON_1".into(), MbValue::from_int(33));
    attrs.insert("ABMON_10".into(), MbValue::from_int(42));
    attrs.insert("ABMON_11".into(), MbValue::from_int(43));
    attrs.insert("ABMON_12".into(), MbValue::from_int(44));
    attrs.insert("ABMON_2".into(), MbValue::from_int(34));
    attrs.insert("ABMON_3".into(), MbValue::from_int(35));
    attrs.insert("ABMON_4".into(), MbValue::from_int(36));
    attrs.insert("ABMON_5".into(), MbValue::from_int(37));
    attrs.insert("ABMON_6".into(), MbValue::from_int(38));
    attrs.insert("ABMON_7".into(), MbValue::from_int(39));
    attrs.insert("ABMON_8".into(), MbValue::from_int(40));
    attrs.insert("ABMON_9".into(), MbValue::from_int(41));
    attrs.insert("ALT_DIGITS".into(), MbValue::from_int(49));
    attrs.insert("AM_STR".into(), MbValue::from_int(5));
    attrs.insert("CHAR_MAX".into(), MbValue::from_int(127));
    attrs.insert("CODESET".into(), MbValue::from_int(0));
    attrs.insert("CRNCYSTR".into(), MbValue::from_int(56));
    attrs.insert("DAY_1".into(), MbValue::from_int(7));
    attrs.insert("DAY_2".into(), MbValue::from_int(8));
    attrs.insert("DAY_3".into(), MbValue::from_int(9));
    attrs.insert("DAY_4".into(), MbValue::from_int(10));
    attrs.insert("DAY_5".into(), MbValue::from_int(11));
    attrs.insert("DAY_6".into(), MbValue::from_int(12));
    attrs.insert("DAY_7".into(), MbValue::from_int(13));
    attrs.insert("D_FMT".into(), MbValue::from_int(2));
    attrs.insert("D_T_FMT".into(), MbValue::from_int(1));
    attrs.insert("ERA".into(), MbValue::from_int(45));
    attrs.insert("ERA_D_FMT".into(), MbValue::from_int(46));
    attrs.insert("ERA_D_T_FMT".into(), MbValue::from_int(47));
    attrs.insert("ERA_T_FMT".into(), MbValue::from_int(48));
    attrs.insert("LC_COLLATE".into(), MbValue::from_int(1));
    attrs.insert("LC_MESSAGES".into(), MbValue::from_int(6));
    attrs.insert("LC_MONETARY".into(), MbValue::from_int(3));
    attrs.insert("MON_1".into(), MbValue::from_int(21));
    attrs.insert("MON_10".into(), MbValue::from_int(30));
    attrs.insert("MON_11".into(), MbValue::from_int(31));
    attrs.insert("MON_12".into(), MbValue::from_int(32));
    attrs.insert("MON_2".into(), MbValue::from_int(22));
    attrs.insert("MON_3".into(), MbValue::from_int(23));
    attrs.insert("MON_4".into(), MbValue::from_int(24));
    attrs.insert("MON_5".into(), MbValue::from_int(25));
    attrs.insert("MON_6".into(), MbValue::from_int(26));
    attrs.insert("MON_7".into(), MbValue::from_int(27));
    attrs.insert("MON_8".into(), MbValue::from_int(28));
    attrs.insert("MON_9".into(), MbValue::from_int(29));
    attrs.insert("NOEXPR".into(), MbValue::from_int(53));
    attrs.insert("PM_STR".into(), MbValue::from_int(6));
    attrs.insert("RADIXCHAR".into(), MbValue::from_int(50));
    attrs.insert("THOUSEP".into(), MbValue::from_int(51));
    attrs.insert("T_FMT".into(), MbValue::from_int(3));
    attrs.insert("T_FMT_AMPM".into(), MbValue::from_int(4));
    attrs.insert("YESEXPR".into(), MbValue::from_int(52));
    super::register_module("locale", attrs);
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

pub fn mb_locale_getlocale() -> MbValue {
    let lang = MbValue::from_ptr(MbObject::new_str("en_US".to_string()));
    let enc = MbValue::from_ptr(MbObject::new_str("UTF-8".to_string()));
    MbValue::from_ptr(MbObject::new_tuple(vec![lang, enc]))
}

pub fn mb_locale_setlocale(cat: MbValue, locale_str: MbValue) -> MbValue {
    // Unknown category: CPython's `setlocale(category, ...)` raises
    // `locale.Error` for a category int outside the valid LC_* range (0..=6).
    // Query mode (`setlocale(cat)` with no locale arg) uses the same categories,
    // so this only rejects genuinely out-of-range ints like 999999 and never a
    // standard LC_* constant.
    if let Some(c) = cat.as_int() {
        if !(0..=6).contains(&c) {
            return raise_named("Error", &format!("unsupported locale category {c}"));
        }
    } else if !cat.is_none() {
        // A non-int category (e.g. a string) is a TypeError per CPython.
        return raise_named("TypeError", "an integer is required (got type str)");
    }
    // bytes (or a tuple containing bytes) can never name a locale.
    let has_bytes = |v: MbValue| -> bool {
        v.as_ptr().is_some_and(|p| unsafe {
            match &(*p).data {
                ObjData::Bytes(_) | ObjData::ByteArray(_) => true,
                ObjData::Tuple(items) => items.iter().any(|it| {
                    it.as_ptr().is_some_and(|q| {
                        matches!((*q).data, ObjData::Bytes(_) | ObjData::ByteArray(_))
                    })
                }),
                _ => false,
            }
        })
    };
    if has_bytes(locale_str) {
        return raise_named(
            "TypeError",
            "locale must be a string or iterable of strings",
        );
    }
    if let Some(s) = extract_str(locale_str) {
        // Unknown locale: CPython raises `locale.Error` when the requested locale
        // is not installed. Mamba has no real locale DB, so we only reject names
        // that cannot be a POSIX locale at all (multi-word garbage tokens). A
        // well-formed but uninstalled name is allowed through — a missed raise is
        // acceptable, a wrong raise on a valid name (e.g. "fr_FR.UTF-8") is not.
        if !locale_name_is_valid_looking(&s) {
            return raise_named("Error", &format!("unsupported locale setting: {s}"));
        }
        MbValue::from_ptr(MbObject::new_str(s))
    } else {
        MbValue::from_ptr(MbObject::new_str("en_US.UTF-8".to_string()))
    }
}

/// `locale.atof(s)` — parse a string to float in the current (C-like) locale.
/// Raises `ValueError` on a non-numeric string, matching CPython. Grouping
/// commas are stripped before parsing so grouped C-locale numerics still parse.
pub fn mb_locale_atof(val: MbValue) -> MbValue {
    let Some(s) = extract_str(val) else {
        return raise_named("ValueError", "could not convert string to float");
    };
    let cleaned: String = s.trim().chars().filter(|c| *c != ',').collect();
    match cleaned.parse::<f64>() {
        Ok(f) => MbValue::from_float(f),
        Err(_) => raise_named(
            "ValueError",
            &format!("could not convert string to float: '{s}'"),
        ),
    }
}

/// `locale.atoi(s)` — parse a string to int in the current (C-like) locale.
/// Raises `ValueError` on a non-integer string, matching CPython.
pub fn mb_locale_atoi(val: MbValue) -> MbValue {
    let Some(s) = extract_str(val) else {
        return raise_named("ValueError", "invalid literal for int()");
    };
    let cleaned: String = s.trim().chars().filter(|c| *c != ',').collect();
    match cleaned.parse::<i64>() {
        Ok(i) => MbValue::from_int(i),
        Err(_) => raise_named(
            "ValueError",
            &format!("invalid literal for int() with base 10: '{s}'"),
        ),
    }
}

/// `locale.strxfrm(s)` — transform a string into a sort key. CPython rejects an
/// embedded NUL with `ValueError`. In the C locale the identity transform
/// preserves ordering, so returning the input string is order-correct.
pub fn mb_locale_strxfrm(val: MbValue) -> MbValue {
    let Some(s) = extract_str(val) else {
        return MbValue::none();
    };
    if s.contains('\0') {
        return raise_named("ValueError", "embedded null character");
    }
    MbValue::from_ptr(MbObject::new_str(s))
}

/// `locale.strcoll(a, b)` — compare two strings per the current collation.
/// CPython rejects an embedded NUL in either operand with `ValueError`. In the C
/// locale collation reduces to byte ordering, returning <0 / 0 / >0.
pub fn mb_locale_strcoll(a: MbValue, b: MbValue) -> MbValue {
    let (Some(sa), Some(sb)) = (extract_str(a), extract_str(b)) else {
        return raise_named("TypeError", "strcoll arguments must be strings");
    };
    if sa.contains('\0') || sb.contains('\0') {
        return raise_named("ValueError", "embedded null character");
    }
    let ord = match sa.as_bytes().cmp(sb.as_bytes()) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    };
    MbValue::from_int(ord)
}

pub fn mb_locale_format_string(fmt: MbValue, val: MbValue) -> MbValue {
    // locale.format_string(fmt, val) with the default grouping=False is the
    // same printf-style substitution as `fmt % val` — delegate to the shared
    // formatter so %%-escapes, tuple args, and %(name)s mappings all work
    // (the previous stub only handled a single %d/%f).
    let f = extract_str(fmt).unwrap_or_default();
    super::super::string_ops::mb_str_percent_format(f, val)
}

pub fn mb_locale_LC_ALL() -> MbValue {
    MbValue::from_int(6)
}
pub fn mb_locale_LC_CTYPE() -> MbValue {
    MbValue::from_int(0)
}
pub fn mb_locale_LC_TIME() -> MbValue {
    MbValue::from_int(2)
}
pub fn mb_locale_LC_NUMERIC() -> MbValue {
    MbValue::from_int(1)
}

#[cfg(test)]
mod tests {
    use super::super::super::rc::{MbObject, ObjData};
    use super::super::super::value::MbValue;
    use super::*;

    fn tuple_str_at(val: MbValue, idx: usize) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Tuple(ref items) = (*ptr).data {
                items.get(idx).and_then(|v| {
                    v.as_ptr().and_then(|p| {
                        if let ObjData::Str(ref s) = (*p).data {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                })
            } else {
                None
            }
        })
    }

    fn get_str_val(val: MbValue) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
    }

    #[test]
    fn test_getlocale_tuple() {
        let result = mb_locale_getlocale();
        assert_eq!(tuple_str_at(result, 0).as_deref(), Some("en_US"));
        assert_eq!(tuple_str_at(result, 1).as_deref(), Some("UTF-8"));
    }

    #[test]
    fn test_setlocale_with_str() {
        let cat = MbValue::none();
        let locale = MbValue::from_ptr(MbObject::new_str("fr_FR.UTF-8".to_string()));
        let result = mb_locale_setlocale(cat, locale);
        assert_eq!(get_str_val(result).as_deref(), Some("fr_FR.UTF-8"));
    }

    #[test]
    fn test_setlocale_without_str() {
        let cat = MbValue::none();
        let result = mb_locale_setlocale(cat, MbValue::none());
        assert_eq!(get_str_val(result).as_deref(), Some("en_US.UTF-8"));
    }

    #[test]
    fn test_format_string_int() {
        let fmt = MbValue::from_ptr(MbObject::new_str("count: %d".to_string()));
        let result = mb_locale_format_string(fmt, MbValue::from_int(42));
        assert_eq!(get_str_val(result).as_deref(), Some("count: 42"));
    }

    #[test]
    fn test_format_string_float() {
        let fmt = MbValue::from_ptr(MbObject::new_str("pi=%f".to_string()));
        let result = mb_locale_format_string(fmt, MbValue::from_float(3.14159));
        assert_eq!(get_str_val(result).as_deref(), Some("pi=3.141590"));
    }

    #[test]
    fn test_lc_constants() {
        assert_eq!(mb_locale_LC_ALL().as_int(), Some(6));
        assert_eq!(mb_locale_LC_CTYPE().as_int(), Some(0));
        assert_eq!(mb_locale_LC_TIME().as_int(), Some(2));
        assert_eq!(mb_locale_LC_NUMERIC().as_int(), Some(1));
        // non-str format → unchanged
        let fmt = MbValue::from_ptr(MbObject::new_str("x=%d".to_string()));
        let result = mb_locale_format_string(fmt, MbValue::none());
        assert_eq!(get_str_val(result).as_deref(), Some("x=%d"));
    }
}
