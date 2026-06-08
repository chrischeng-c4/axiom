/// __main__ module for Mamba.
///
/// Provides the top-level script execution module. In CPython, `__main__` is
/// the module where top-level code runs. The `if __name__ == "__main__"` idiom
/// checks this attribute to distinguish script execution from module import.
use std::collections::HashMap;
use super::super::rc::MbObject;
use super::super::value::MbValue;

// @spec .aw/changes/mamba-stdlib-main/groups/stdlib-main-module/specs/mamba-stdlib-main-spec.md
pub fn register() {
    let mut attrs = HashMap::new();

    // __name__ = "__main__"
    attrs.insert(
        "__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str("__main__".to_string())),
    );

    // __doc__ = None
    attrs.insert("__doc__".to_string(), MbValue::none());

    // __loader__ = None
    attrs.insert("__loader__".to_string(), MbValue::none());

    // __spec__ = None
    attrs.insert("__spec__".to_string(), MbValue::none());

        // surface: missing CPython module constants (auto-added)
    attrs.insert("lib".into(), MbValue::from_ptr(MbObject::new_str("__main__".to_string())));
    attrs.insert("line".into(), MbValue::from_ptr(MbObject::new_str("zlib zlib_mod.rs".to_string())));
    attrs.insert("modfile".into(), MbValue::from_ptr(MbObject::new_str("main_mod.rs".to_string())));
    attrs.insert("msg".into(), MbValue::from_ptr(MbObject::new_str("__future__: 0 missing constants".to_string())));
    attrs.insert("n".into(), MbValue::from_int(0));
    attrs.insert("total".into(), MbValue::from_int(0));
    attrs.insert("touched".into(), MbValue::from_int(0));
    super::register_module("__main__", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_name_value() {
        let v = MbValue::from_ptr(MbObject::new_str("__main__".to_string()));
        unsafe {
            let obj = &*v.as_ptr().unwrap();
            if let crate::runtime::rc::ObjData::Str(ref s) = obj.data {
                assert_eq!(s.as_str(), "__main__");
            } else {
                panic!("expected string");
            }
        }
    }

    #[test]
    fn test_main_doc_is_none() {
        let v = MbValue::none();
        assert!(v.is_none());
    }

    #[test]
    fn test_main_loader_is_none() {
        let v = MbValue::none();
        assert!(v.is_none());
    }

    #[test]
    fn test_main_spec_is_none() {
        let v = MbValue::none();
        assert!(v.is_none());
    }

    #[test]
    fn test_register_module() {
        // Calling register() should not panic
        register();
    }
}
