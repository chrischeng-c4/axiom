/// C type representation from parsed headers (#256).
#[derive(Debug, Clone, PartialEq)]
pub enum CType {
    Void,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float,
    Double,
    Bool,
    /// `const char*` — null-terminated string
    ConstChar,
    /// `char*` — mutable string
    MutChar,
    /// Pointer to another type
    Pointer(Box<CType>),
    /// Const pointer to another type
    ConstPointer(Box<CType>),
    /// Named type (struct/enum/typedef reference)
    Named(String),
}

/// A parsed C function declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct CFunction {
    pub name: String,
    pub params: Vec<CParam>,
    pub return_type: CType,
}

/// A function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct CParam {
    pub name: String,
    pub ty: CType,
}

/// A parsed C struct definition.
#[derive(Debug, Clone, PartialEq)]
pub struct CStruct {
    pub name: String,
    pub fields: Vec<CField>,
}

/// A struct field.
#[derive(Debug, Clone, PartialEq)]
pub struct CField {
    pub name: String,
    pub ty: CType,
}

/// A parsed C enum definition.
#[derive(Debug, Clone, PartialEq)]
pub struct CEnum {
    pub name: String,
    pub variants: Vec<CEnumVariant>,
}

/// An enum variant with optional value.
#[derive(Debug, Clone, PartialEq)]
pub struct CEnumVariant {
    pub name: String,
    pub value: Option<i64>,
}

/// Complete parsed header contents.
#[derive(Debug, Clone, Default)]
pub struct CHeader {
    pub functions: Vec<CFunction>,
    pub structs: Vec<CStruct>,
    pub enums: Vec<CEnum>,
}

impl CType {
    /// Human-readable name for diagnostics.
    pub fn display_name(&self) -> String {
        match self {
            CType::Void => "void".into(),
            CType::Int8 => "int8_t".into(),
            CType::Int16 => "int16_t".into(),
            CType::Int32 => "int32_t".into(),
            CType::Int64 => "int64_t".into(),
            CType::UInt8 => "uint8_t".into(),
            CType::UInt16 => "uint16_t".into(),
            CType::UInt32 => "uint32_t".into(),
            CType::UInt64 => "uint64_t".into(),
            CType::Float => "float".into(),
            CType::Double => "double".into(),
            CType::Bool => "bool".into(),
            CType::ConstChar => "const char*".into(),
            CType::MutChar => "char*".into(),
            CType::Pointer(inner) => format!("{}*", inner.display_name()),
            CType::ConstPointer(inner) => format!("const {}*", inner.display_name()),
            CType::Named(name) => name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- CType::display_name for all 17 variants ---
    #[test]
    fn test_display_name_void() {
        assert_eq!(CType::Void.display_name(), "void");
    }

    #[test]
    fn test_display_name_int8() {
        assert_eq!(CType::Int8.display_name(), "int8_t");
    }

    #[test]
    fn test_display_name_int16() {
        assert_eq!(CType::Int16.display_name(), "int16_t");
    }

    #[test]
    fn test_display_name_int32() {
        assert_eq!(CType::Int32.display_name(), "int32_t");
    }

    #[test]
    fn test_display_name_int64() {
        assert_eq!(CType::Int64.display_name(), "int64_t");
    }

    #[test]
    fn test_display_name_uint8() {
        assert_eq!(CType::UInt8.display_name(), "uint8_t");
    }

    #[test]
    fn test_display_name_uint16() {
        assert_eq!(CType::UInt16.display_name(), "uint16_t");
    }

    #[test]
    fn test_display_name_uint32() {
        assert_eq!(CType::UInt32.display_name(), "uint32_t");
    }

    #[test]
    fn test_display_name_uint64() {
        assert_eq!(CType::UInt64.display_name(), "uint64_t");
    }

    #[test]
    fn test_display_name_float() {
        assert_eq!(CType::Float.display_name(), "float");
    }

    #[test]
    fn test_display_name_double() {
        assert_eq!(CType::Double.display_name(), "double");
    }

    #[test]
    fn test_display_name_bool() {
        assert_eq!(CType::Bool.display_name(), "bool");
    }

    #[test]
    fn test_display_name_const_char() {
        assert_eq!(CType::ConstChar.display_name(), "const char*");
    }

    #[test]
    fn test_display_name_mut_char() {
        assert_eq!(CType::MutChar.display_name(), "char*");
    }

    #[test]
    fn test_display_name_pointer() {
        let t = CType::Pointer(Box::new(CType::Int32));
        assert_eq!(t.display_name(), "int32_t*");
    }

    #[test]
    fn test_display_name_const_pointer() {
        let t = CType::ConstPointer(Box::new(CType::UInt8));
        assert_eq!(t.display_name(), "const uint8_t*");
    }

    #[test]
    fn test_display_name_named() {
        let t = CType::Named("MyStruct".to_string());
        assert_eq!(t.display_name(), "MyStruct");
    }

    // --- Debug ---
    #[test]
    fn test_debug_pointer_variant() {
        let t = CType::Pointer(Box::new(CType::Int32));
        let s = format!("{t:?}");
        assert!(s.contains("Pointer"));
    }

    // --- Clone / PartialEq ---
    #[test]
    fn test_ctype_eq_int32_int32() {
        assert_eq!(CType::Int32, CType::Int32);
    }

    #[test]
    fn test_ctype_neq_int32_int64() {
        assert_ne!(CType::Int32, CType::Int64);
    }

    #[test]
    fn test_cenum_clone() {
        let e = CEnum {
            name: "Color".to_string(),
            variants: vec![
                CEnumVariant {
                    name: "Red".to_string(),
                    value: Some(0),
                },
                CEnumVariant {
                    name: "Green".to_string(),
                    value: Some(1),
                },
            ],
        };
        let e2 = e.clone();
        assert_eq!(e, e2);
    }

    // --- CFunction ---
    #[test]
    fn test_cfunction_construct_clone_eq() {
        let f = CFunction {
            name: "add".to_string(),
            params: vec![
                CParam {
                    name: "a".to_string(),
                    ty: CType::Int32,
                },
                CParam {
                    name: "b".to_string(),
                    ty: CType::Int32,
                },
            ],
            return_type: CType::Int32,
        };
        let f2 = f.clone();
        assert_eq!(f, f2);
    }

    // --- CParam ---
    #[test]
    fn test_cparam_construct_clone_eq() {
        let p = CParam {
            name: "x".to_string(),
            ty: CType::Double,
        };
        let p2 = p.clone();
        assert_eq!(p, p2);
    }

    // --- CStruct ---
    #[test]
    fn test_cstruct_construct_clone_eq() {
        let s = CStruct {
            name: "Point".to_string(),
            fields: vec![
                CField {
                    name: "x".to_string(),
                    ty: CType::Float,
                },
                CField {
                    name: "y".to_string(),
                    ty: CType::Float,
                },
            ],
        };
        let s2 = s.clone();
        assert_eq!(s, s2);
    }

    // --- CField ---
    #[test]
    fn test_cfield_construct_clone() {
        let f = CField {
            name: "val".to_string(),
            ty: CType::Int64,
        };
        let f2 = f.clone();
        assert_eq!(f.name, f2.name);
    }

    // --- CEnumVariant ---
    #[test]
    fn test_cenumvariant_value_some() {
        let v = CEnumVariant {
            name: "A".to_string(),
            value: Some(42),
        };
        assert_eq!(v.value, Some(42));
    }

    #[test]
    fn test_cenumvariant_value_none() {
        let v = CEnumVariant {
            name: "B".to_string(),
            value: None,
        };
        assert_eq!(v.value, None);
    }

    // --- CHeader ---
    #[test]
    fn test_cheader_default_and_push() {
        let mut h = CHeader::default();
        assert!(h.functions.is_empty());
        h.functions.push(CFunction {
            name: "f".to_string(),
            params: vec![],
            return_type: CType::Void,
        });
        assert_eq!(h.functions.len(), 1);
        h.structs.push(CStruct {
            name: "S".to_string(),
            fields: vec![],
        });
        assert_eq!(h.structs.len(), 1);
        h.enums.push(CEnum {
            name: "E".to_string(),
            variants: vec![],
        });
        assert_eq!(h.enums.len(), 1);
    }
}
