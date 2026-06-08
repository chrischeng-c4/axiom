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

dispatch_nullary!(dispatch_getlocale, mb_locale_getlocale);
dispatch_binary!(dispatch_setlocale, mb_locale_setlocale);
dispatch_binary!(dispatch_format_string, mb_locale_format_string);

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("getlocale", dispatch_getlocale as usize),
        ("setlocale", dispatch_setlocale as usize),
        ("format_string", dispatch_format_string as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    // LC_* are integer constants in CPython, not callables — emit them
    // eagerly as ints (matching the `pub fn mb_locale_LC_*()` returns).
    attrs.insert("LC_ALL".to_string(), mb_locale_LC_ALL());
    attrs.insert("LC_CTYPE".to_string(), mb_locale_LC_CTYPE());
    attrs.insert("LC_TIME".to_string(), mb_locale_LC_TIME());
    attrs.insert("LC_NUMERIC".to_string(), mb_locale_LC_NUMERIC());
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

pub fn mb_locale_setlocale(_cat: MbValue, locale_str: MbValue) -> MbValue {
    if let Some(s) = extract_str(locale_str) {
        MbValue::from_ptr(MbObject::new_str(s))
    } else {
        MbValue::from_ptr(MbObject::new_str("en_US.UTF-8".to_string()))
    }
}

pub fn mb_locale_format_string(fmt: MbValue, val: MbValue) -> MbValue {
    let f = extract_str(fmt).unwrap_or_default();
    let result = if let Some(i) = val.as_int() {
        f.replacen("%d", &i.to_string(), 1)
    } else if let Some(fl) = val.as_float() {
        f.replacen("%f", &format!("{:.6}", fl), 1)
    } else {
        f
    };
    MbValue::from_ptr(MbObject::new_str(result))
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
