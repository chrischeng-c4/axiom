use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// argparse module for Mamba (#399).
///
/// Provides: ArgumentParser (stub for CLI argument parsing).
/// Functions: mb_argparse_new, mb_argparse_add_argument, mb_argparse_parse_args
use std::collections::HashMap;

unsafe extern "C" fn dispatch_argument_parser(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // ArgumentParser(description=...) — keyword arrives as trailing dict.
    let mut desc = a.get(0).copied().unwrap_or_else(MbValue::none);
    for v in a.iter() {
        if let Some(ptr) = v.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let g = lock.read().unwrap();
                    let key = super::super::dict_ops::DictKey::Str("description".to_string());
                    if let Some(found) = g.get(&key) {
                        desc = *found;
                        break;
                    }
                }
            }
        }
    }
    mb_argparse_new(desc)
}

/// Register the argparse module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr = dispatch_argument_parser as *const () as usize;
    attrs.insert("ArgumentParser".into(), MbValue::from_func(addr));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });

    super::register_module("argparse", attrs);
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

/// argparse.ArgumentParser(description) -> parser instance dict
pub fn mb_argparse_new(desc: MbValue) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            let d = extract_str(desc).unwrap_or_default();
            map.insert(
                "description".into(),
                MbValue::from_ptr(MbObject::new_str(d)),
            );
            map.insert(
                "_args".into(),
                MbValue::from_ptr(MbObject::new_list(vec![])),
            );
        }
    }
    MbValue::from_ptr(dict)
}

/// parser.add_argument(name, ...) -> None
pub fn mb_argparse_add_argument(parser: MbValue, name: MbValue) -> MbValue {
    if let Some(ptr) = parser.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(args_val) = map.get("_args").copied() {
                    if let Some(args_ptr) = args_val.as_ptr() {
                        if let ObjData::List(ref lock2) = (*args_ptr).data {
                            let mut items = lock2.write().unwrap();
                            items.push(name);
                        }
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// parser.parse_args() -> namespace dict with arg values from env
pub fn mb_argparse_parse_args(parser: MbValue) -> MbValue {
    let result = MbObject::new_dict();
    // Parse from std::env::args, skip program name
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Get registered argument names
    let mut names = Vec::new();
    if let Some(ptr) = parser.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(args_val) = map.get("_args").copied() {
                    if let Some(args_ptr) = args_val.as_ptr() {
                        if let ObjData::List(ref lock2) = (*args_ptr).data {
                            let items = lock2.read().unwrap();
                            for item in items.iter() {
                                if let Some(s) = extract_str(*item) {
                                    names.push(s);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Simple positional parsing: assign args to names in order
    unsafe {
        if let ObjData::Dict(ref lock) = (*result).data {
            let mut map = lock.write().unwrap();
            let mut arg_idx = 0;
            for name in &names {
                let clean = name.trim_start_matches('-');
                if arg_idx < args.len() {
                    map.insert(
                        clean.into(),
                        MbValue::from_ptr(MbObject::new_str(args[arg_idx].clone())),
                    );
                    arg_idx += 1;
                } else {
                    map.insert(clean.into(), MbValue::none());
                }
            }
        }
    }
    MbValue::from_ptr(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_str_field(dict: MbValue, key: &str) -> Option<String> {
        if let Some(ptr) = dict.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if let Some(val) = map.get(key).copied() {
                        return extract_str(val);
                    }
                }
            }
        }
        None
    }

    fn get_list_len(dict: MbValue, key: &str) -> usize {
        if let Some(ptr) = dict.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if let Some(val) = map.get(key).copied() {
                        if let Some(list_ptr) = val.as_ptr() {
                            if let ObjData::List(ref l) = (*list_ptr).data {
                                return l.read().unwrap().len();
                            }
                        }
                    }
                }
            }
        }
        0
    }

    #[test]
    fn test_new_parser() {
        let desc = MbValue::from_ptr(MbObject::new_str("test".to_string()));
        let parser = mb_argparse_new(desc);
        assert!(parser.as_ptr().is_some());
    }

    #[test]
    fn test_new_parser_with_str_desc() {
        let desc = MbValue::from_ptr(MbObject::new_str("my description".to_string()));
        let parser = mb_argparse_new(desc);
        assert_eq!(
            get_str_field(parser, "description"),
            Some("my description".to_string())
        );
    }

    #[test]
    fn test_new_parser_with_non_str_desc() {
        let desc = MbValue::from_int(0);
        let parser = mb_argparse_new(desc);
        assert_eq!(get_str_field(parser, "description"), Some(String::new()));
    }

    #[test]
    fn test_extract_str_str_value() {
        let s = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        assert_eq!(extract_str(s), Some("hello".to_string()));
    }

    #[test]
    fn test_extract_str_non_str_value() {
        assert_eq!(extract_str(MbValue::from_int(42)), None);
    }

    #[test]
    fn test_extract_str_null_ptr() {
        assert_eq!(extract_str(MbValue::none()), None);
    }

    #[test]
    fn test_add_argument_valid_parser() {
        let desc = MbValue::from_ptr(MbObject::new_str("desc".to_string()));
        let parser = mb_argparse_new(desc);
        let name = MbValue::from_ptr(MbObject::new_str("--foo".to_string()));
        mb_argparse_add_argument(parser, name);
        assert_eq!(get_list_len(parser, "_args"), 1);
    }

    #[test]
    fn test_add_argument_null_parser() {
        // Should not panic
        let name = MbValue::from_ptr(MbObject::new_str("--foo".to_string()));
        mb_argparse_add_argument(MbValue::none(), name);
    }

    #[test]
    fn test_add_argument_non_dict_parser() {
        // Non-dict ptr: list is not a dict, so no-op
        let list_val = MbValue::from_ptr(MbObject::new_list(vec![]));
        let name = MbValue::from_ptr(MbObject::new_str("--foo".to_string()));
        mb_argparse_add_argument(list_val, name);
    }

    #[test]
    fn test_parse_args_no_names() {
        let desc = MbValue::from_ptr(MbObject::new_str("".to_string()));
        let parser = mb_argparse_new(desc);
        let ns = mb_argparse_parse_args(parser);
        assert!(ns.as_ptr().is_some());
    }

    #[test]
    fn test_parse_args_null_parser() {
        let ns = mb_argparse_parse_args(MbValue::none());
        assert!(ns.as_ptr().is_some());
    }

    #[test]
    fn test_parse_args_fewer_env_args_than_names() {
        let desc = MbValue::from_ptr(MbObject::new_str("".to_string()));
        let parser = mb_argparse_new(desc);
        // Register 2 args but env will have fewer (test process won't pass them)
        let n1 = MbValue::from_ptr(MbObject::new_str("--alpha".to_string()));
        let n2 = MbValue::from_ptr(MbObject::new_str("--beta".to_string()));
        mb_argparse_add_argument(parser, n1);
        mb_argparse_add_argument(parser, n2);
        let ns = mb_argparse_parse_args(parser);
        // Both keys should be present (None for unmatched)
        assert!(ns.as_ptr().is_some());
    }
}
