use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// configparser module for Mamba (#mamba-stdlib).
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

macro_rules! dispatch_ternary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
                a.get(2).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

macro_rules! dispatch_quaternary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
                a.get(2).copied().unwrap_or_else(MbValue::none),
                a.get(3).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

dispatch_nullary!(dispatch_ConfigParser, mb_configparser_ConfigParser);
dispatch_binary!(dispatch_read_string, mb_configparser_read_string);
dispatch_ternary!(dispatch_get, mb_configparser_get);
dispatch_quaternary!(dispatch_set, mb_configparser_set);
dispatch_unary!(dispatch_sections, mb_configparser_sections);
dispatch_binary!(dispatch_options, mb_configparser_options);

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("ConfigParser", dispatch_ConfigParser as usize),
        ("read_string", dispatch_read_string as usize),
        ("get", dispatch_get as usize),
        ("set", dispatch_set as usize),
        ("sections", dispatch_sections as usize),
        ("options", dispatch_options as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("configparser", attrs);
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

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

pub fn mb_configparser_ConfigParser() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut m = lock.write().unwrap();
            m.insert(
                "__class__".into(),
                MbValue::from_ptr(MbObject::new_str("ConfigParser".to_string())),
            );
            m.insert("_data".into(), MbValue::from_ptr(MbObject::new_dict()));
        }
    }
    MbValue::from_ptr(dict)
}

pub fn mb_configparser_read_string(parser: MbValue, text: MbValue) -> MbValue {
    let text_str = match extract_str(text) {
        Some(s) => s,
        None => return raise_type_error("read_string() argument must be str"),
    };
    let data_ptr = parser.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let map = lock.read().unwrap();
            map.get("_data").and_then(|v| v.as_ptr())
        } else {
            None
        }
    });
    let dp = match data_ptr {
        Some(p) => p,
        None => return MbValue::none(),
    };
    let mut cur_sec = String::new();
    for line in text_str.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            cur_sec = line[1..line.len() - 1].trim().to_string();
            unsafe {
                if let ObjData::Dict(ref lock) = (*dp).data {
                    let mut map = lock.write().unwrap();
                    if !map.contains_key(&cur_sec) {
                        map.insert(
                            cur_sec.clone().into(),
                            MbValue::from_ptr(MbObject::new_dict()),
                        );
                    }
                }
            }
        } else if let Some(eq_pos) = line.find('=') {
            let k = line[..eq_pos].trim().to_string();
            let v = line[eq_pos + 1..].trim().to_string();
            if !cur_sec.is_empty() {
                unsafe {
                    if let ObjData::Dict(ref lock) = (*dp).data {
                        let map = lock.read().unwrap();
                        if let Some(sv) = map.get(&cur_sec) {
                            if let Some(sp) = sv.as_ptr() {
                                if let ObjData::Dict(ref sl) = (*sp).data {
                                    sl.write()
                                        .unwrap()
                                        .insert(k.into(), MbValue::from_ptr(MbObject::new_str(v)));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    MbValue::none()
}

pub fn mb_configparser_get(parser: MbValue, section: MbValue, key: MbValue) -> MbValue {
    let sec = extract_str(section).unwrap_or_default();
    let k = extract_str(key).unwrap_or_default();
    if let Some(ptr) = parser.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(dv) = map.get("_data") {
                    if let Some(dp) = dv.as_ptr() {
                        if let ObjData::Dict(ref dl) = (*dp).data {
                            let dm = dl.read().unwrap();
                            if let Some(sv) = dm.get(&sec) {
                                if let Some(sp) = sv.as_ptr() {
                                    if let ObjData::Dict(ref sl) = (*sp).data {
                                        if let Some(v) = sl.read().unwrap().get(&k) {
                                            return *v;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    MbValue::none()
}

pub fn mb_configparser_set(
    parser: MbValue,
    section: MbValue,
    key: MbValue,
    value: MbValue,
) -> MbValue {
    let sec = extract_str(section).unwrap_or_default();
    let k = extract_str(key).unwrap_or_default();
    let v = extract_str(value).unwrap_or_default();
    if let Some(ptr) = parser.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(dv) = map.get("_data") {
                    if let Some(dp) = dv.as_ptr() {
                        if let ObjData::Dict(ref dl) = (*dp).data {
                            let mut dm = dl.write().unwrap();
                            let sv = dm
                                .entry(sec.into())
                                .or_insert_with(|| MbValue::from_ptr(MbObject::new_dict()));
                            if let Some(sp) = sv.as_ptr() {
                                if let ObjData::Dict(ref sl) = (*sp).data {
                                    sl.write()
                                        .unwrap()
                                        .insert(k.into(), MbValue::from_ptr(MbObject::new_str(v)));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    MbValue::none()
}

pub fn mb_configparser_sections(parser: MbValue) -> MbValue {
    let mut names = Vec::new();
    if let Some(ptr) = parser.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(dv) = map.get("_data") {
                    if let Some(dp) = dv.as_ptr() {
                        if let ObjData::Dict(ref dl) = (*dp).data {
                            for k in dl.read().unwrap().keys() {
                                names.push(MbValue::from_ptr(MbObject::new_str(k.to_string())));
                            }
                        }
                    }
                }
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(names))
}

pub fn mb_configparser_options(parser: MbValue, section: MbValue) -> MbValue {
    let sec = extract_str(section).unwrap_or_default();
    let mut keys = Vec::new();
    if let Some(ptr) = parser.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(dv) = map.get("_data") {
                    if let Some(dp) = dv.as_ptr() {
                        if let ObjData::Dict(ref dl) = (*dp).data {
                            let dm = dl.read().unwrap();
                            if let Some(sv) = dm.get(&sec) {
                                if let Some(sp) = sv.as_ptr() {
                                    if let ObjData::Dict(ref sl) = (*sp).data {
                                        for k in sl.read().unwrap().keys() {
                                            keys.push(MbValue::from_ptr(MbObject::new_str(
                                                k.to_string(),
                                            )));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(keys))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn get_str(val: MbValue) -> String {
        extract_str(val).unwrap_or_default()
    }

    fn list_strs(val: MbValue) -> Vec<String> {
        val.as_ptr()
            .map(|ptr| unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    lock.read()
                        .unwrap()
                        .iter()
                        .filter_map(|v| extract_str(*v))
                        .collect()
                } else {
                    vec![]
                }
            })
            .unwrap_or_default()
    }

    #[test]
    fn test_create_parser() {
        let p = mb_configparser_ConfigParser();
        assert!(p.as_ptr().is_some());
        // Should have __class__ = "ConfigParser"
        let ptr = p.as_ptr().unwrap();
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let m = lock.read().unwrap();
                assert_eq!(
                    extract_str(*m.get("__class__").unwrap()),
                    Some("ConfigParser".to_string())
                );
            } else {
                panic!("expected dict");
            }
        }
    }

    #[test]
    fn test_read_string_basic() {
        let p = mb_configparser_ConfigParser();
        let ini = "[section1]\nkey1 = value1\nkey2 = value2\n";
        mb_configparser_read_string(p, s(ini));
        let v = mb_configparser_get(p, s("section1"), s("key1"));
        assert_eq!(get_str(v), "value1");
        let v2 = mb_configparser_get(p, s("section1"), s("key2"));
        assert_eq!(get_str(v2), "value2");
    }

    #[test]
    fn test_read_string_multiple_sections() {
        let p = mb_configparser_ConfigParser();
        let ini = "[db]\nhost = localhost\nport = 5432\n\n[app]\ndebug = true\n";
        mb_configparser_read_string(p, s(ini));
        assert_eq!(
            get_str(mb_configparser_get(p, s("db"), s("host"))),
            "localhost"
        );
        assert_eq!(get_str(mb_configparser_get(p, s("db"), s("port"))), "5432");
        assert_eq!(
            get_str(mb_configparser_get(p, s("app"), s("debug"))),
            "true"
        );
    }

    #[test]
    fn test_read_string_comments_and_blank_lines() {
        let p = mb_configparser_ConfigParser();
        let ini = "# comment line\n; another comment\n\n[sec]\nk = v\n";
        mb_configparser_read_string(p, s(ini));
        assert_eq!(get_str(mb_configparser_get(p, s("sec"), s("k"))), "v");
    }

    #[test]
    fn test_get_missing_section() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_read_string(p, s("[s]\nk = v\n"));
        let v = mb_configparser_get(p, s("nonexist"), s("k"));
        assert!(v.is_none());
    }

    #[test]
    fn test_get_missing_key() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_read_string(p, s("[s]\nk = v\n"));
        let v = mb_configparser_get(p, s("s"), s("missing"));
        assert!(v.is_none());
    }

    #[test]
    fn test_set_new_section_and_key() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_set(p, s("new_sec"), s("key"), s("val"));
        assert_eq!(
            get_str(mb_configparser_get(p, s("new_sec"), s("key"))),
            "val"
        );
    }

    #[test]
    fn test_set_overwrite() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_read_string(p, s("[s]\nk = old\n"));
        mb_configparser_set(p, s("s"), s("k"), s("new"));
        assert_eq!(get_str(mb_configparser_get(p, s("s"), s("k"))), "new");
    }

    #[test]
    fn test_sections_empty() {
        let p = mb_configparser_ConfigParser();
        let secs = list_strs(mb_configparser_sections(p));
        assert!(secs.is_empty());
    }

    #[test]
    fn test_sections_after_read() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_read_string(p, s("[alpha]\nx = 1\n[beta]\ny = 2\n"));
        let mut secs = list_strs(mb_configparser_sections(p));
        secs.sort();
        assert_eq!(secs, vec!["alpha", "beta"]);
    }

    #[test]
    fn test_options() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_read_string(p, s("[s]\na = 1\nb = 2\nc = 3\n"));
        let mut opts = list_strs(mb_configparser_options(p, s("s")));
        opts.sort();
        assert_eq!(opts, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_options_missing_section() {
        let p = mb_configparser_ConfigParser();
        let opts = list_strs(mb_configparser_options(p, s("nope")));
        assert!(opts.is_empty());
    }

    #[test]
    fn test_value_with_spaces_around_equals() {
        let p = mb_configparser_ConfigParser();
        mb_configparser_read_string(p, s("[s]\n  key  =  val  \n"));
        // key and value should be trimmed
        assert_eq!(get_str(mb_configparser_get(p, s("s"), s("key"))), "val");
    }
}
