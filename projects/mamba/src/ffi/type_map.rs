use super::c_types::*;
use crate::types::ty::{Ty, TypeId};
use crate::types::context::TypeContext;

/// Map a C type to a Mamba type (#257).
pub fn c_type_to_mamba(ct: &CType, tcx: &mut TypeContext) -> TypeId {
    match ct {
        CType::Void => tcx.none(),
        CType::Int8 | CType::Int16 | CType::Int32 | CType::Int64
        | CType::UInt8 | CType::UInt16 | CType::UInt32 | CType::UInt64 => tcx.int(),
        CType::Float | CType::Double => tcx.float(),
        CType::Bool => tcx.bool(),
        CType::ConstChar | CType::MutChar => tcx.str(),
        CType::Pointer(inner) | CType::ConstPointer(inner) => {
            // Pointers → optional type (T?)
            let inner_ty = c_type_to_mamba(inner, tcx);
            let none_ty = tcx.none();
            tcx.intern(Ty::Union(vec![inner_ty, none_ty]))
        }
        CType::Named(name) => {
            // Named types resolve to class/enum references
            tcx.intern(Ty::Class {
                name: name.clone(),
                fields: vec![],
                match_args: None,
            })
        }
    }
}

/// Generate a Mamba class from a C struct (#258).
pub fn struct_to_class(s: &CStruct, tcx: &mut TypeContext) -> TypeId {
    let fields: Vec<(String, TypeId)> = s.fields.iter()
        .map(|f| (f.name.clone(), c_type_to_mamba(&f.ty, tcx)))
        .collect();
    tcx.intern(Ty::Class {
        name: s.name.clone(),
        fields,
        match_args: None,
    })
}

/// Generate a Mamba enum-like class from a C enum (#259).
pub fn enum_to_class(e: &CEnum, tcx: &mut TypeContext) -> TypeId {
    let int_ty = tcx.int();
    let fields: Vec<(String, TypeId)> = e.variants.iter()
        .map(|v| {
            let clean_name = clean_variant_name(&v.name, &e.name);
            (clean_name, int_ty)
        })
        .collect();
    tcx.intern(Ty::Class {
        name: e.name.clone(),
        fields,
        match_args: None,
    })
}

/// Clean up cbindgen enum variant names (#259).
/// E.g., "ColorRed" → "RED", "Color_Red" → "RED"
fn clean_variant_name(variant: &str, enum_name: &str) -> String {
    let stripped = if variant.starts_with(enum_name) {
        &variant[enum_name.len()..]
    } else if variant.starts_with(&format!("{}_", enum_name)) {
        &variant[enum_name.len() + 1..]
    } else {
        variant
    };
    let stripped = stripped.trim_start_matches('_');
    // Convert to UPPER_SNAKE_CASE
    let mut result = String::new();
    for (i, c) in stripped.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_uppercase());
    }
    if result.is_empty() {
        variant.to_uppercase()
    } else {
        result
    }
}

/// Map a complete C header to Mamba types (#257, #258, #259).
pub fn map_header_types(header: &CHeader, tcx: &mut TypeContext) -> MappedTypes {
    let mut mapped = MappedTypes::default();

    for s in &header.structs {
        let ty = struct_to_class(s, tcx);
        mapped.classes.push((s.name.clone(), ty));
    }

    for e in &header.enums {
        let ty = enum_to_class(e, tcx);
        mapped.classes.push((e.name.clone(), ty));
    }

    for f in &header.functions {
        let param_types: Vec<TypeId> = f.params.iter()
            .map(|p| c_type_to_mamba(&p.ty, tcx))
            .collect();
        let ret = c_type_to_mamba(&f.return_type, tcx);
        let fn_ty = tcx.intern(Ty::Fn { params: param_types, ret, variadic: false });
        mapped.functions.push((f.name.clone(), fn_ty));
    }

    mapped
}

/// Result of mapping a C header to Mamba types.
#[derive(Debug, Default)]
pub struct MappedTypes {
    pub classes: Vec<(String, TypeId)>,
    pub functions: Vec<(String, TypeId)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_type_mapping() {
        let mut tcx = TypeContext::new();
        assert_eq!(c_type_to_mamba(&CType::Int32, &mut tcx), tcx.int());
        assert_eq!(c_type_to_mamba(&CType::Double, &mut tcx), tcx.float());
        assert_eq!(c_type_to_mamba(&CType::Bool, &mut tcx), tcx.bool());
        assert_eq!(c_type_to_mamba(&CType::Void, &mut tcx), tcx.none());
        assert_eq!(c_type_to_mamba(&CType::ConstChar, &mut tcx), tcx.str());
    }

    #[test]
    fn test_struct_to_class() {
        let mut tcx = TypeContext::new();
        let s = CStruct {
            name: "Point".into(),
            fields: vec![
                CField { name: "x".into(), ty: CType::Int32 },
                CField { name: "y".into(), ty: CType::Int32 },
            ],
        };
        let ty = struct_to_class(&s, &mut tcx);
        match tcx.get(ty) {
            Ty::Class { name, fields, .. } => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
            }
            _ => panic!("expected Class type"),
        }
    }

    #[test]
    fn test_enum_to_class() {
        let mut tcx = TypeContext::new();
        let e = CEnum {
            name: "Color".into(),
            variants: vec![
                CEnumVariant { name: "ColorRed".into(), value: Some(0) },
                CEnumVariant { name: "ColorGreen".into(), value: Some(1) },
            ],
        };
        let ty = enum_to_class(&e, &mut tcx);
        match tcx.get(ty) {
            Ty::Class { name, fields, .. } => {
                assert_eq!(name, "Color");
                assert_eq!(fields[0].0, "RED");
                assert_eq!(fields[1].0, "GREEN");
            }
            _ => panic!("expected Class type"),
        }
    }

    #[test]
    fn test_clean_variant_name() {
        assert_eq!(clean_variant_name("ColorRed", "Color"), "RED");
        assert_eq!(clean_variant_name("Color_Green", "Color"), "GREEN");
        assert_eq!(clean_variant_name("Blue", "Shade"), "BLUE");
    }

    // --- Additional tests ---

    #[test]
    fn test_all_int_types_map_to_int() {
        let mut tcx = TypeContext::new();
        let int_id = tcx.int();
        assert_eq!(c_type_to_mamba(&CType::Int8, &mut tcx), int_id);
        assert_eq!(c_type_to_mamba(&CType::Int16, &mut tcx), int_id);
        assert_eq!(c_type_to_mamba(&CType::Int32, &mut tcx), int_id);
        assert_eq!(c_type_to_mamba(&CType::Int64, &mut tcx), int_id);
        assert_eq!(c_type_to_mamba(&CType::UInt8, &mut tcx), int_id);
        assert_eq!(c_type_to_mamba(&CType::UInt16, &mut tcx), int_id);
        assert_eq!(c_type_to_mamba(&CType::UInt32, &mut tcx), int_id);
        assert_eq!(c_type_to_mamba(&CType::UInt64, &mut tcx), int_id);
    }

    #[test]
    fn test_float_and_double_map_to_float() {
        let mut tcx = TypeContext::new();
        let float_id = tcx.float();
        assert_eq!(c_type_to_mamba(&CType::Float, &mut tcx), float_id);
        assert_eq!(c_type_to_mamba(&CType::Double, &mut tcx), float_id);
    }

    #[test]
    fn test_char_types_map_to_str() {
        let mut tcx = TypeContext::new();
        let str_id = tcx.str();
        assert_eq!(c_type_to_mamba(&CType::ConstChar, &mut tcx), str_id);
        assert_eq!(c_type_to_mamba(&CType::MutChar, &mut tcx), str_id);
    }

    #[test]
    fn test_pointer_maps_to_optional() {
        let mut tcx = TypeContext::new();
        let ptr_ty = CType::Pointer(Box::new(CType::Int32));
        let mapped = c_type_to_mamba(&ptr_ty, &mut tcx);
        match tcx.get(mapped) {
            Ty::Union(variants) => {
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0], tcx.int());
                assert_eq!(variants[1], tcx.none());
            }
            _ => panic!("expected Union type for pointer"),
        }
    }

    #[test]
    fn test_const_pointer_maps_to_optional() {
        let mut tcx = TypeContext::new();
        let ptr_ty = CType::ConstPointer(Box::new(CType::Double));
        let mapped = c_type_to_mamba(&ptr_ty, &mut tcx);
        match tcx.get(mapped) {
            Ty::Union(variants) => {
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0], tcx.float());
                assert_eq!(variants[1], tcx.none());
            }
            _ => panic!("expected Union type for const pointer"),
        }
    }

    #[test]
    fn test_named_type_maps_to_class() {
        let mut tcx = TypeContext::new();
        let named = CType::Named("MyStruct".into());
        let mapped = c_type_to_mamba(&named, &mut tcx);
        match tcx.get(mapped) {
            Ty::Class { name, fields, .. } => {
                assert_eq!(name, "MyStruct");
                assert!(fields.is_empty());
            }
            _ => panic!("expected Class type for Named"),
        }
    }

    #[test]
    fn test_struct_to_class_field_types() {
        let mut tcx = TypeContext::new();
        let s = CStruct {
            name: "Data".into(),
            fields: vec![
                CField { name: "count".into(), ty: CType::Int32 },
                CField { name: "value".into(), ty: CType::Double },
                CField { name: "name".into(), ty: CType::ConstChar },
                CField { name: "active".into(), ty: CType::Bool },
            ],
        };
        let ty = struct_to_class(&s, &mut tcx);
        match tcx.get(ty) {
            Ty::Class { name, fields, .. } => {
                assert_eq!(name, "Data");
                assert_eq!(fields.len(), 4);
                assert_eq!(fields[0].1, tcx.int());
                assert_eq!(fields[1].1, tcx.float());
                assert_eq!(fields[2].1, tcx.str());
                assert_eq!(fields[3].1, tcx.bool());
            }
            _ => panic!("expected Class type"),
        }
    }

    #[test]
    fn test_struct_to_class_empty_fields() {
        let mut tcx = TypeContext::new();
        let s = CStruct {
            name: "Empty".into(),
            fields: vec![],
        };
        let ty = struct_to_class(&s, &mut tcx);
        match tcx.get(ty) {
            Ty::Class { name, fields, .. } => {
                assert_eq!(name, "Empty");
                assert!(fields.is_empty());
            }
            _ => panic!("expected Class type"),
        }
    }

    #[test]
    fn test_enum_to_class_all_int_fields() {
        let mut tcx = TypeContext::new();
        let e = CEnum {
            name: "Direction".into(),
            variants: vec![
                CEnumVariant { name: "Up".into(), value: Some(0) },
                CEnumVariant { name: "Down".into(), value: Some(1) },
                CEnumVariant { name: "Left".into(), value: Some(2) },
                CEnumVariant { name: "Right".into(), value: Some(3) },
            ],
        };
        let ty = enum_to_class(&e, &mut tcx);
        let int_id = tcx.int();
        match tcx.get(ty) {
            Ty::Class { name, fields, .. } => {
                assert_eq!(name, "Direction");
                assert_eq!(fields.len(), 4);
                for (_, field_ty) in fields {
                    assert_eq!(*field_ty, int_id);
                }
            }
            _ => panic!("expected Class type"),
        }
    }

    #[test]
    fn test_clean_variant_name_empty_after_strip() {
        // variant == enum_name => stripped is empty => use full variant
        let result = clean_variant_name("Color", "Color");
        assert_eq!(result, "COLOR");
    }

    #[test]
    fn test_clean_variant_name_underscore_separator() {
        assert_eq!(clean_variant_name("Color_Blue", "Color"), "BLUE");
    }

    #[test]
    fn test_clean_variant_name_camel_case_split() {
        assert_eq!(clean_variant_name("StatusNotFound", "Status"), "NOT_FOUND");
    }

    #[test]
    fn test_map_header_types_full() {
        let mut tcx = TypeContext::new();
        let header = CHeader {
            structs: vec![CStruct {
                name: "Vec2".into(),
                fields: vec![
                    CField { name: "x".into(), ty: CType::Float },
                    CField { name: "y".into(), ty: CType::Float },
                ],
            }],
            enums: vec![CEnum {
                name: "Axis".into(),
                variants: vec![
                    CEnumVariant { name: "X".into(), value: Some(0) },
                    CEnumVariant { name: "Y".into(), value: Some(1) },
                ],
            }],
            functions: vec![CFunction {
                name: "dot".into(),
                params: vec![
                    CParam { name: "a".into(), ty: CType::Float },
                    CParam { name: "b".into(), ty: CType::Float },
                ],
                return_type: CType::Float,
            }],
        };
        let mapped = map_header_types(&header, &mut tcx);
        assert_eq!(mapped.classes.len(), 2);
        assert_eq!(mapped.classes[0].0, "Vec2");
        assert_eq!(mapped.classes[1].0, "Axis");
        assert_eq!(mapped.functions.len(), 1);
        assert_eq!(mapped.functions[0].0, "dot");
        // dot function should be a Fn type
        match tcx.get(mapped.functions[0].1) {
            Ty::Fn { params, ret, .. } => {
                assert_eq!(params.len(), 2);
                assert_eq!(*ret, tcx.float());
            }
            _ => panic!("expected Fn type"),
        }
    }

    #[test]
    fn test_map_header_types_empty() {
        let mut tcx = TypeContext::new();
        let header = CHeader::default();
        let mapped = map_header_types(&header, &mut tcx);
        assert!(mapped.classes.is_empty());
        assert!(mapped.functions.is_empty());
    }
}
