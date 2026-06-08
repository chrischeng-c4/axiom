/// __future__ module for Mamba.
///
/// Exposes CPython 3.12 future feature flags and compiler flag constants.
/// This is a constants-only module for compatibility — Mamba enables all
/// modern Python features by default, so these flags are informational only.
use super::super::rc::{MbObject, ObjData, IMMORTAL_REFCOUNT};
use super::super::value::MbValue;
use std::collections::HashMap;
use std::sync::atomic::Ordering;

// Compiler flag constants (matching CPython 3.12 code object flags)
const CO_NESTED: i64 = 0x0010;
const CO_GENERATOR_ALLOWED: i64 = 0;
const CO_FUTURE_DIVISION: i64 = 0x2_0000;
const CO_FUTURE_ABSOLUTE_IMPORT: i64 = 0x4_0000;
const CO_FUTURE_WITH_STATEMENT: i64 = 0x8_0000;
const CO_FUTURE_PRINT_FUNCTION: i64 = 0x10_0000;
const CO_FUTURE_UNICODE_LITERALS: i64 = 0x20_0000;
const CO_FUTURE_BARRY_AS_BDFL: i64 = 0x40_0000;
const CO_FUTURE_GENERATOR_STOP: i64 = 0x80_0000;
const CO_FUTURE_ANNOTATIONS: i64 = 0x100_0000;

// @spec .aw/tech-design/projects/mamba/stdlib/future.md
pub fn register() {
    register_feature_class();

    let mut attrs = HashMap::new();

    // CO_* compiler flag constants
    attrs.insert("CO_NESTED".to_string(), MbValue::from_int(CO_NESTED));
    attrs.insert(
        "CO_GENERATOR_ALLOWED".to_string(),
        MbValue::from_int(CO_GENERATOR_ALLOWED),
    );
    attrs.insert(
        "CO_FUTURE_DIVISION".to_string(),
        MbValue::from_int(CO_FUTURE_DIVISION),
    );
    attrs.insert(
        "CO_FUTURE_ABSOLUTE_IMPORT".to_string(),
        MbValue::from_int(CO_FUTURE_ABSOLUTE_IMPORT),
    );
    attrs.insert(
        "CO_FUTURE_WITH_STATEMENT".to_string(),
        MbValue::from_int(CO_FUTURE_WITH_STATEMENT),
    );
    attrs.insert(
        "CO_FUTURE_PRINT_FUNCTION".to_string(),
        MbValue::from_int(CO_FUTURE_PRINT_FUNCTION),
    );
    attrs.insert(
        "CO_FUTURE_UNICODE_LITERALS".to_string(),
        MbValue::from_int(CO_FUTURE_UNICODE_LITERALS),
    );
    attrs.insert(
        "CO_FUTURE_BARRY_AS_BDFL".to_string(),
        MbValue::from_int(CO_FUTURE_BARRY_AS_BDFL),
    );
    attrs.insert(
        "CO_FUTURE_GENERATOR_STOP".to_string(),
        MbValue::from_int(CO_FUTURE_GENERATOR_STOP),
    );
    attrs.insert(
        "CO_FUTURE_ANNOTATIONS".to_string(),
        MbValue::from_int(CO_FUTURE_ANNOTATIONS),
    );

    attrs.insert("_Feature".to_string(), str_value("__future__._Feature"));
    let feature_names = feature_specs()
        .iter()
        .map(|spec| str_value(spec.name))
        .collect();
    attrs.insert(
        "all_feature_names".to_string(),
        MbValue::from_ptr(MbObject::new_list(feature_names)),
    );

    for spec in feature_specs() {
        attrs.insert(spec.name.to_string(), make_feature(*spec));
    }

    super::register_module("__future__", attrs);
}

#[derive(Clone, Copy)]
struct FeatureSpec {
    name: &'static str,
    optional: (i64, i64, i64, &'static str, i64),
    mandatory: Option<(i64, i64, i64, &'static str, i64)>,
    flag: i64,
}

fn feature_specs() -> &'static [FeatureSpec] {
    &[
        FeatureSpec {
            name: "nested_scopes",
            optional: (2, 1, 0, "beta", 1),
            mandatory: Some((2, 2, 0, "final", 0)),
            flag: CO_NESTED,
        },
        FeatureSpec {
            name: "generators",
            optional: (2, 2, 0, "alpha", 1),
            mandatory: Some((2, 3, 0, "final", 0)),
            flag: CO_GENERATOR_ALLOWED,
        },
        FeatureSpec {
            name: "division",
            optional: (2, 2, 0, "alpha", 2),
            mandatory: Some((3, 0, 0, "alpha", 0)),
            flag: CO_FUTURE_DIVISION,
        },
        FeatureSpec {
            name: "absolute_import",
            optional: (2, 5, 0, "alpha", 1),
            mandatory: Some((3, 0, 0, "alpha", 0)),
            flag: CO_FUTURE_ABSOLUTE_IMPORT,
        },
        FeatureSpec {
            name: "with_statement",
            optional: (2, 5, 0, "alpha", 1),
            mandatory: Some((2, 6, 0, "alpha", 0)),
            flag: CO_FUTURE_WITH_STATEMENT,
        },
        FeatureSpec {
            name: "print_function",
            optional: (2, 6, 0, "alpha", 2),
            mandatory: Some((3, 0, 0, "alpha", 0)),
            flag: CO_FUTURE_PRINT_FUNCTION,
        },
        FeatureSpec {
            name: "unicode_literals",
            optional: (2, 6, 0, "alpha", 2),
            mandatory: Some((3, 0, 0, "alpha", 0)),
            flag: CO_FUTURE_UNICODE_LITERALS,
        },
        FeatureSpec {
            name: "barry_as_FLUFL",
            optional: (3, 1, 0, "alpha", 2),
            mandatory: Some((4, 0, 0, "alpha", 0)),
            flag: CO_FUTURE_BARRY_AS_BDFL,
        },
        FeatureSpec {
            name: "generator_stop",
            optional: (3, 5, 0, "beta", 1),
            mandatory: Some((3, 7, 0, "alpha", 0)),
            flag: CO_FUTURE_GENERATOR_STOP,
        },
        FeatureSpec {
            name: "annotations",
            optional: (3, 7, 0, "beta", 1),
            mandatory: None,
            flag: CO_FUTURE_ANNOTATIONS,
        },
    ]
}

fn register_feature_class() {
    let mut methods = HashMap::new();
    methods.insert(
        "getOptionalRelease".to_string(),
        MbValue::from_func(mb_feature_get_optional as usize),
    );
    methods.insert(
        "getMandatoryRelease".to_string(),
        MbValue::from_func(mb_feature_get_mandatory as usize),
    );
    super::super::class::mb_class_register("__future__._Feature", vec![], methods);
}

extern "C" fn mb_feature_get_optional(self_obj: MbValue) -> MbValue {
    get_feature_field(self_obj, "_optional")
}

extern "C" fn mb_feature_get_mandatory(self_obj: MbValue) -> MbValue {
    get_feature_field(self_obj, "_mandatory")
}

fn get_feature_field(self_obj: MbValue, field_name: &str) -> MbValue {
    let value = self_obj.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(field_name).copied()
        } else {
            None
        }
    });
    let value = value.unwrap_or_else(MbValue::none);
    unsafe {
        super::super::rc::retain_if_ptr(value);
    }
    value
}

fn make_feature(spec: FeatureSpec) -> MbValue {
    let obj = MbObject::new_instance("__future__._Feature".to_string());
    unsafe {
        (*obj).header.rc.store(IMMORTAL_REFCOUNT, Ordering::Relaxed);
        if let ObjData::Instance { ref fields, .. } = (*obj).data {
            let mut fields = fields.write().unwrap();
            fields.insert("_optional".to_string(), release_tuple(spec.optional));
            fields.insert(
                "_mandatory".to_string(),
                spec.mandatory
                    .map(release_tuple)
                    .unwrap_or_else(MbValue::none),
            );
            fields.insert("compiler_flag".to_string(), MbValue::from_int(spec.flag));
        }
    }
    MbValue::from_ptr(obj)
}

fn release_tuple(value: (i64, i64, i64, &'static str, i64)) -> MbValue {
    let (major, minor, micro, level, serial) = value;
    let tuple = MbObject::new_tuple(vec![
        MbValue::from_int(major),
        MbValue::from_int(minor),
        MbValue::from_int(micro),
        str_value(level),
        MbValue::from_int(serial),
    ]);
    unsafe {
        (*tuple)
            .header
            .rc
            .store(IMMORTAL_REFCOUNT, Ordering::Relaxed);
    }
    MbValue::from_ptr(tuple)
}

fn str_value(value: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str_immortal(value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_co_future_annotations_value() {
        assert_eq!(CO_FUTURE_ANNOTATIONS, 0x100_0000);
        let v = MbValue::from_int(CO_FUTURE_ANNOTATIONS);
        assert_eq!(v.as_int(), Some(0x100_0000));
    }

    #[test]
    fn test_co_future_division_value() {
        assert_eq!(CO_FUTURE_DIVISION, 0x2_0000);
        let v = MbValue::from_int(CO_FUTURE_DIVISION);
        assert_eq!(v.as_int(), Some(0x2_0000));
    }

    #[test]
    fn test_co_future_absolute_import_value() {
        assert_eq!(CO_FUTURE_ABSOLUTE_IMPORT, 0x4_0000);
        let v = MbValue::from_int(CO_FUTURE_ABSOLUTE_IMPORT);
        assert_eq!(v.as_int(), Some(0x4_0000));
    }

    #[test]
    fn test_co_future_with_statement_value() {
        assert_eq!(CO_FUTURE_WITH_STATEMENT, 0x8_0000);
        let v = MbValue::from_int(CO_FUTURE_WITH_STATEMENT);
        assert_eq!(v.as_int(), Some(0x8_0000));
    }

    #[test]
    fn test_co_future_print_function_value() {
        assert_eq!(CO_FUTURE_PRINT_FUNCTION, 0x10_0000);
        let v = MbValue::from_int(CO_FUTURE_PRINT_FUNCTION);
        assert_eq!(v.as_int(), Some(0x10_0000));
    }

    #[test]
    fn test_co_future_unicode_literals_value() {
        assert_eq!(CO_FUTURE_UNICODE_LITERALS, 0x20_0000);
        let v = MbValue::from_int(CO_FUTURE_UNICODE_LITERALS);
        assert_eq!(v.as_int(), Some(0x20_0000));
    }

    #[test]
    fn test_co_nested_value() {
        assert_eq!(CO_NESTED, 0x0010);
        let v = MbValue::from_int(CO_NESTED);
        assert_eq!(v.as_int(), Some(0x0010));
    }

    #[test]
    fn test_co_generator_allowed_value() {
        assert_eq!(CO_GENERATOR_ALLOWED, 0);
        let v = MbValue::from_int(CO_GENERATOR_ALLOWED);
        assert_eq!(v.as_int(), Some(0));
    }

    #[test]
    fn test_feature_objects_expose_compiler_flags() {
        let cases: &[(&str, i64)] = &[
            ("annotations", CO_FUTURE_ANNOTATIONS),
            ("division", CO_FUTURE_DIVISION),
            ("print_function", CO_FUTURE_PRINT_FUNCTION),
            ("unicode_literals", CO_FUTURE_UNICODE_LITERALS),
            ("with_statement", CO_FUTURE_WITH_STATEMENT),
            ("absolute_import", CO_FUTURE_ABSOLUTE_IMPORT),
        ];
        for (name, expected) in cases {
            let feature = make_feature(
                *feature_specs()
                    .iter()
                    .find(|spec| spec.name == *name)
                    .unwrap(),
            );
            let flag = unsafe {
                if let ObjData::Instance { ref fields, .. } = (*feature.as_ptr().unwrap()).data {
                    fields
                        .read()
                        .unwrap()
                        .get("compiler_flag")
                        .and_then(|v| v.as_int())
                } else {
                    None
                }
            };
            assert_eq!(
                flag,
                Some(*expected),
                "feature flag '{}' did not roundtrip correctly",
                name
            );
        }
    }

    #[test]
    fn test_feature_release_methods_return_tuples() {
        register_feature_class();
        let division = make_feature(
            feature_specs()
                .iter()
                .find(|spec| spec.name == "division")
                .copied()
                .unwrap(),
        );
        let optional = mb_feature_get_optional(division);
        let mandatory = mb_feature_get_mandatory(division);
        unsafe {
            assert!(matches!(
                (*optional.as_ptr().unwrap()).data,
                ObjData::Tuple(_)
            ));
            assert!(matches!(
                (*mandatory.as_ptr().unwrap()).data,
                ObjData::Tuple(_)
            ));
        }
    }

    #[test]
    fn test_register_module() {
        // Calling register() should not panic
        register();
    }
}
