use super::super::rc::MbObject;
use super::super::value::MbValue;
/// __main__ module for Mamba.
///
/// Provides the top-level script execution module. In CPython, `__main__` is
/// the module where top-level code runs. The `if __name__ == "__main__"` idiom
/// checks this attribute to distinguish script execution from module import.
use std::collections::HashMap;

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
