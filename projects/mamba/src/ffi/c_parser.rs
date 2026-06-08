use super::c_types::*;

/// Parse a cbindgen-generated C header into structured types (#256).
pub fn parse_c_header(source: &str) -> CHeader {
    let mut header = CHeader::default();
    let mut lines = source.lines().peekable();

    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        // Skip preprocessor directives and comments
        if trimmed.starts_with('#') || trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }

        // Typedef struct
        if trimmed.starts_with("typedef struct") {
            if let Some(s) = parse_typedef_struct(trimmed, &mut lines) {
                header.structs.push(s);
            }
            continue;
        }

        // Typedef enum
        if trimmed.starts_with("typedef enum") {
            if let Some(e) = parse_typedef_enum(trimmed, &mut lines) {
                header.enums.push(e);
            }
            continue;
        }

        // Function declaration (has parentheses and semicolon)
        if trimmed.contains('(') && trimmed.ends_with(';') && !trimmed.starts_with("typedef") {
            if let Some(f) = parse_function_decl(trimmed) {
                header.functions.push(f);
            }
        }
    }

    header
}

fn parse_function_decl(line: &str) -> Option<CFunction> {
    let line = line.trim_end_matches(';').trim();
    let paren_start = line.find('(')?;
    let paren_end = line.rfind(')')?;

    // Parse return type and name
    let before_paren = &line[..paren_start].trim();
    let (ret_type_str, name) = split_type_and_name(before_paren)?;
    let return_type = parse_c_type(ret_type_str.trim());

    // Parse parameters
    let params_str = &line[paren_start + 1..paren_end].trim();
    let params = if params_str.is_empty() || *params_str == "void" {
        Vec::new()
    } else {
        params_str
            .split(',')
            .filter_map(|p| {
                let p = p.trim();
                let (ty_str, name) = split_type_and_name(p)?;
                Some(CParam {
                    name: name.to_string(),
                    ty: parse_c_type(ty_str.trim()),
                })
            })
            .collect()
    };

    Some(CFunction {
        name: name.to_string(),
        params,
        return_type,
    })
}

fn parse_typedef_struct(
    first_line: &str,
    lines: &mut std::iter::Peekable<std::str::Lines>,
) -> Option<CStruct> {
    let mut fields = Vec::new();
    let mut name = String::new();

    // Check for single-line typedef struct
    if first_line.contains('}') {
        return parse_single_line_struct(first_line);
    }

    // Multi-line struct
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed.starts_with('}') {
            // Extract name after closing brace
            let after = trimmed.trim_start_matches('}').trim().trim_end_matches(';').trim();
            if !after.is_empty() {
                name = after.to_string();
            }
            break;
        }
        if let Some(field) = parse_struct_field(trimmed) {
            fields.push(field);
        }
    }

    if name.is_empty() { return None; }
    Some(CStruct { name, fields })
}

fn parse_single_line_struct(line: &str) -> Option<CStruct> {
    let open = line.find('{')?;
    let close = line.find('}')?;
    let after = line[close + 1..].trim().trim_end_matches(';').trim();
    let name = after.to_string();
    let fields_str = &line[open + 1..close];
    let fields: Vec<CField> = fields_str
        .split(';')
        .filter_map(|f| parse_struct_field(f.trim()))
        .collect();
    if name.is_empty() { return None; }
    Some(CStruct { name, fields })
}

fn parse_struct_field(field_str: &str) -> Option<CField> {
    let field_str = field_str.trim().trim_end_matches(';').trim();
    if field_str.is_empty() { return None; }
    let (ty_str, name) = split_type_and_name(field_str)?;
    Some(CField {
        name: name.to_string(),
        ty: parse_c_type(ty_str.trim()),
    })
}

fn parse_typedef_enum(
    first_line: &str,
    lines: &mut std::iter::Peekable<std::str::Lines>,
) -> Option<CEnum> {
    let mut variants = Vec::new();
    let mut name = String::new();

    if first_line.contains('}') {
        return parse_single_line_enum(first_line);
    }

    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed.starts_with('}') {
            let after = trimmed.trim_start_matches('}').trim().trim_end_matches(';').trim();
            if !after.is_empty() {
                name = after.to_string();
            }
            break;
        }
        let trimmed = trimmed.trim_end_matches(',');
        if let Some(variant) = parse_enum_variant(trimmed) {
            variants.push(variant);
        }
    }

    if name.is_empty() { return None; }
    Some(CEnum { name, variants })
}

fn parse_single_line_enum(line: &str) -> Option<CEnum> {
    let open = line.find('{')?;
    let close = line.find('}')?;
    let after = line[close + 1..].trim().trim_end_matches(';').trim();
    let name = after.to_string();
    let variants_str = &line[open + 1..close];
    let variants: Vec<CEnumVariant> = variants_str
        .split(',')
        .filter_map(|v| parse_enum_variant(v.trim()))
        .collect();
    if name.is_empty() { return None; }
    Some(CEnum { name, variants })
}

fn parse_enum_variant(s: &str) -> Option<CEnumVariant> {
    let s = s.trim();
    if s.is_empty() { return None; }
    if let Some(eq_pos) = s.find('=') {
        let name = s[..eq_pos].trim().to_string();
        let value = s[eq_pos + 1..].trim().parse::<i64>().ok();
        Some(CEnumVariant { name, value })
    } else {
        Some(CEnumVariant { name: s.to_string(), value: None })
    }
}

/// Split a C declaration like "int32_t x" into ("int32_t", "x").
fn split_type_and_name(decl: &str) -> Option<(&str, &str)> {
    let decl = decl.trim();
    if decl.is_empty() { return None; }

    // Handle pointer: "type *name" or "type* name"
    if let Some(star_pos) = decl.rfind('*') {
        let after_star = decl[star_pos + 1..].trim();
        if !after_star.is_empty() && after_star.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Some((&decl[..=star_pos], after_star));
        }
    }

    // Standard: last word is the name
    if let Some(space_pos) = decl.rfind(|c: char| c.is_whitespace()) {
        let ty = &decl[..space_pos];
        let name = &decl[space_pos + 1..];
        if !name.is_empty() {
            return Some((ty, name));
        }
    }

    None
}

/// Parse a C type string into a CType.
pub fn parse_c_type(s: &str) -> CType {
    let s = s.trim();
    match s {
        "void" => CType::Void,
        "bool" | "_Bool" => CType::Bool,
        "int8_t" | "char" => CType::Int8,
        "int16_t" | "short" => CType::Int16,
        "int32_t" | "int" => CType::Int32,
        "int64_t" | "long" | "long long" | "intptr_t" | "ssize_t" => CType::Int64,
        "uint8_t" | "unsigned char" => CType::UInt8,
        "uint16_t" | "unsigned short" => CType::UInt16,
        "uint32_t" | "unsigned int" | "unsigned" => CType::UInt32,
        "uint64_t" | "unsigned long" | "size_t" | "uintptr_t" => CType::UInt64,
        "float" => CType::Float,
        "double" => CType::Double,
        "const char*" | "const char *" => CType::ConstChar,
        "char*" | "char *" => CType::MutChar,
        _ => {
            // Handle pointer types
            if s.ends_with('*') {
                let inner = s.trim_end_matches('*').trim();
                if inner.starts_with("const ") {
                    let base = inner.trim_start_matches("const ").trim();
                    CType::ConstPointer(Box::new(parse_c_type(base)))
                } else {
                    CType::Pointer(Box::new(parse_c_type(inner)))
                }
            } else {
                CType::Named(s.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_function() {
        let header = parse_c_header("int32_t add(int32_t a, int32_t b);\n");
        assert_eq!(header.functions.len(), 1);
        let f = &header.functions[0];
        assert_eq!(f.name, "add");
        assert_eq!(f.params.len(), 2);
        assert_eq!(f.return_type, CType::Int32);
    }

    #[test]
    fn test_parse_struct() {
        let header = parse_c_header(
            "typedef struct {\n  int32_t x;\n  int32_t y;\n} Point;\n"
        );
        assert_eq!(header.structs.len(), 1);
        assert_eq!(header.structs[0].name, "Point");
        assert_eq!(header.structs[0].fields.len(), 2);
    }

    #[test]
    fn test_parse_enum() {
        let header = parse_c_header(
            "typedef enum {\n  Red = 0,\n  Green = 1,\n  Blue = 2,\n} Color;\n"
        );
        assert_eq!(header.enums.len(), 1);
        assert_eq!(header.enums[0].name, "Color");
        assert_eq!(header.enums[0].variants.len(), 3);
        assert_eq!(header.enums[0].variants[0].value, Some(0));
    }

    #[test]
    fn test_parse_void_function() {
        let header = parse_c_header("void print_msg(const char* msg);\n");
        assert_eq!(header.functions.len(), 1);
        let f = &header.functions[0];
        assert_eq!(f.name, "print_msg");
        assert_eq!(f.return_type, CType::Void);
        assert_eq!(f.params[0].ty, CType::ConstChar);
    }

    #[test]
    fn test_parse_c_type() {
        assert_eq!(parse_c_type("int32_t"), CType::Int32);
        assert_eq!(parse_c_type("double"), CType::Double);
        assert_eq!(parse_c_type("bool"), CType::Bool);
        assert_eq!(parse_c_type("void"), CType::Void);
    }

    // --- Additional tests ---

    #[test]
    fn test_empty_header() {
        let header = parse_c_header("");
        assert!(header.functions.is_empty());
        assert!(header.structs.is_empty());
        assert!(header.enums.is_empty());
    }

    #[test]
    fn test_header_only_comments_and_preprocessor() {
        let src = "// This is a comment\n#include <stdint.h>\n#ifndef GUARD\n// end\n";
        let header = parse_c_header(src);
        assert!(header.functions.is_empty());
        assert!(header.structs.is_empty());
        assert!(header.enums.is_empty());
    }

    #[test]
    fn test_parse_no_param_function() {
        let header = parse_c_header("int32_t get_value(void);\n");
        assert_eq!(header.functions.len(), 1);
        let f = &header.functions[0];
        assert_eq!(f.name, "get_value");
        assert!(f.params.is_empty());
        assert_eq!(f.return_type, CType::Int32);
    }

    #[test]
    fn test_parse_empty_param_function() {
        let header = parse_c_header("void init();\n");
        assert_eq!(header.functions.len(), 1);
        let f = &header.functions[0];
        assert_eq!(f.name, "init");
        assert!(f.params.is_empty());
        assert_eq!(f.return_type, CType::Void);
    }

    #[test]
    fn test_parse_pointer_return() {
        let header = parse_c_header("int32_t* get_buffer(uint32_t size);\n");
        assert_eq!(header.functions.len(), 1);
        let f = &header.functions[0];
        assert_eq!(f.name, "get_buffer");
        assert_eq!(f.return_type, CType::Pointer(Box::new(CType::Int32)));
        assert_eq!(f.params[0].ty, CType::UInt32);
    }

    #[test]
    fn test_parse_multiple_functions() {
        let src = "int32_t add(int32_t a, int32_t b);\nvoid reset();\ndouble avg(float x, float y);\n";
        let header = parse_c_header(src);
        assert_eq!(header.functions.len(), 3);
        assert_eq!(header.functions[0].name, "add");
        assert_eq!(header.functions[1].name, "reset");
        assert_eq!(header.functions[2].name, "avg");
    }

    #[test]
    fn test_parse_struct_field_types() {
        let src = "typedef struct {\n  double x;\n  float y;\n  bool active;\n} Data;\n";
        let header = parse_c_header(src);
        assert_eq!(header.structs.len(), 1);
        let s = &header.structs[0];
        assert_eq!(s.name, "Data");
        assert_eq!(s.fields[0].name, "x");
        assert_eq!(s.fields[0].ty, CType::Double);
        assert_eq!(s.fields[1].name, "y");
        assert_eq!(s.fields[1].ty, CType::Float);
        assert_eq!(s.fields[2].name, "active");
        assert_eq!(s.fields[2].ty, CType::Bool);
    }

    #[test]
    fn test_parse_single_line_struct() {
        let src = "typedef struct { int32_t x; int32_t y } Point;\n";
        let header = parse_c_header(src);
        assert_eq!(header.structs.len(), 1);
        assert_eq!(header.structs[0].name, "Point");
        assert_eq!(header.structs[0].fields.len(), 2);
    }

    #[test]
    fn test_parse_single_line_enum() {
        let src = "typedef enum { A = 1, B = 2 } AB;\n";
        let header = parse_c_header(src);
        assert_eq!(header.enums.len(), 1);
        assert_eq!(header.enums[0].name, "AB");
        assert_eq!(header.enums[0].variants.len(), 2);
        assert_eq!(header.enums[0].variants[0].value, Some(1));
    }

    #[test]
    fn test_parse_enum_no_values() {
        let src = "typedef enum {\n  X,\n  Y,\n  Z,\n} Axis;\n";
        let header = parse_c_header(src);
        assert_eq!(header.enums.len(), 1);
        let e = &header.enums[0];
        assert_eq!(e.name, "Axis");
        assert_eq!(e.variants.len(), 3);
        assert_eq!(e.variants[0].name, "X");
        assert_eq!(e.variants[0].value, None);
    }

    #[test]
    fn test_parse_mixed_header() {
        let src = "\
#include <stdint.h>\n\
\n\
typedef struct {\n  int32_t x;\n  int32_t y;\n} Point;\n\
\n\
typedef enum {\n  Red = 0,\n  Green = 1,\n} Color;\n\
\n\
int32_t distance(Point* a, Point* b);\n";
        let header = parse_c_header(src);
        assert_eq!(header.structs.len(), 1);
        assert_eq!(header.enums.len(), 1);
        assert_eq!(header.functions.len(), 1);
    }

    #[test]
    fn test_parse_c_type_all_int_aliases() {
        assert_eq!(parse_c_type("int8_t"), CType::Int8);
        assert_eq!(parse_c_type("char"), CType::Int8);
        assert_eq!(parse_c_type("int16_t"), CType::Int16);
        assert_eq!(parse_c_type("short"), CType::Int16);
        assert_eq!(parse_c_type("int32_t"), CType::Int32);
        assert_eq!(parse_c_type("int"), CType::Int32);
        assert_eq!(parse_c_type("int64_t"), CType::Int64);
        assert_eq!(parse_c_type("long"), CType::Int64);
        assert_eq!(parse_c_type("ssize_t"), CType::Int64);
    }

    #[test]
    fn test_parse_c_type_all_uint_aliases() {
        assert_eq!(parse_c_type("uint8_t"), CType::UInt8);
        assert_eq!(parse_c_type("unsigned char"), CType::UInt8);
        assert_eq!(parse_c_type("uint16_t"), CType::UInt16);
        assert_eq!(parse_c_type("unsigned short"), CType::UInt16);
        assert_eq!(parse_c_type("uint32_t"), CType::UInt32);
        assert_eq!(parse_c_type("unsigned int"), CType::UInt32);
        assert_eq!(parse_c_type("unsigned"), CType::UInt32);
        assert_eq!(parse_c_type("uint64_t"), CType::UInt64);
        assert_eq!(parse_c_type("size_t"), CType::UInt64);
        assert_eq!(parse_c_type("uintptr_t"), CType::UInt64);
    }

    #[test]
    fn test_parse_c_type_float_double() {
        assert_eq!(parse_c_type("float"), CType::Float);
        assert_eq!(parse_c_type("double"), CType::Double);
    }

    #[test]
    fn test_parse_c_type_strings() {
        assert_eq!(parse_c_type("const char*"), CType::ConstChar);
        assert_eq!(parse_c_type("const char *"), CType::ConstChar);
        assert_eq!(parse_c_type("char*"), CType::MutChar);
        assert_eq!(parse_c_type("char *"), CType::MutChar);
    }

    #[test]
    fn test_parse_c_type_pointer() {
        assert_eq!(
            parse_c_type("int32_t*"),
            CType::Pointer(Box::new(CType::Int32))
        );
    }

    #[test]
    fn test_parse_c_type_const_pointer() {
        assert_eq!(
            parse_c_type("const int32_t*"),
            CType::ConstPointer(Box::new(CType::Int32))
        );
    }

    #[test]
    fn test_parse_c_type_named() {
        assert_eq!(parse_c_type("MyStruct"), CType::Named("MyStruct".into()));
    }

    #[test]
    fn test_parse_c_type_whitespace() {
        assert_eq!(parse_c_type("  int32_t  "), CType::Int32);
        assert_eq!(parse_c_type("  void  "), CType::Void);
    }

    #[test]
    fn test_parse_c_type_bool_alias() {
        assert_eq!(parse_c_type("_Bool"), CType::Bool);
    }

    #[test]
    fn test_split_type_and_name_basic() {
        assert_eq!(split_type_and_name("int32_t x"), Some(("int32_t", "x")));
    }

    #[test]
    fn test_split_type_and_name_pointer() {
        let result = split_type_and_name("int32_t *ptr");
        assert!(result.is_some());
        let (ty, name) = result.unwrap();
        assert_eq!(name, "ptr");
        assert!(ty.contains("*"));
    }

    #[test]
    fn test_split_type_and_name_empty() {
        assert_eq!(split_type_and_name(""), None);
    }

    #[test]
    fn test_parse_struct_with_pointer_fields() {
        let src = "typedef struct {\n  int32_t* data;\n  uint64_t len;\n} Buffer;\n";
        let header = parse_c_header(src);
        assert_eq!(header.structs.len(), 1);
        let s = &header.structs[0];
        assert_eq!(s.name, "Buffer");
        assert_eq!(s.fields.len(), 2);
        assert_eq!(s.fields[0].name, "data");
        assert_eq!(s.fields[1].name, "len");
        assert_eq!(s.fields[1].ty, CType::UInt64);
    }

    #[test]
    fn test_parse_enum_mixed_values() {
        let src = "typedef enum {\n  A = 10,\n  B,\n  C = 20,\n} Mixed;\n";
        let header = parse_c_header(src);
        let e = &header.enums[0];
        assert_eq!(e.variants[0].value, Some(10));
        assert_eq!(e.variants[1].value, None);
        assert_eq!(e.variants[2].value, Some(20));
    }

    #[test]
    fn test_parse_enum_negative_values() {
        let src = "typedef enum {\n  NEG = -1,\n  ZERO = 0,\n  POS = 1,\n} Sign;\n";
        let header = parse_c_header(src);
        let e = &header.enums[0];
        assert_eq!(e.variants[0].name, "NEG");
        assert_eq!(e.variants[0].value, Some(-1));
        assert_eq!(e.variants[1].value, Some(0));
        assert_eq!(e.variants[2].value, Some(1));
    }

    #[test]
    fn test_typedef_struct_no_name_returns_none() {
        let src = "typedef struct {\n  int32_t x;\n};\n";
        let header = parse_c_header(src);
        assert!(header.structs.is_empty());
    }

    #[test]
    fn test_typedef_enum_no_name_returns_none() {
        let src = "typedef enum {\n  A,\n};\n";
        let header = parse_c_header(src);
        assert!(header.enums.is_empty());
    }

    #[test]
    fn test_parse_function_three_params() {
        let src = "double clamp(double val, double lo, double hi);\n";
        let header = parse_c_header(src);
        assert_eq!(header.functions.len(), 1);
        let f = &header.functions[0];
        assert_eq!(f.name, "clamp");
        assert_eq!(f.params.len(), 3);
        assert_eq!(f.params[0].name, "val");
        assert_eq!(f.params[1].name, "lo");
        assert_eq!(f.params[2].name, "hi");
        assert!(f.params.iter().all(|p| p.ty == CType::Double));
        assert_eq!(f.return_type, CType::Double);
    }

    #[test]
    fn test_parse_function_named_return_type() {
        let src = "MyStruct create_struct(int32_t id);\n";
        let header = parse_c_header(src);
        assert_eq!(header.functions.len(), 1);
        let f = &header.functions[0];
        assert_eq!(f.return_type, CType::Named("MyStruct".into()));
    }
}
