//! Python code generators for cclab ecosystem
//!
//! Generators for:
//! - cclab.shield (data validation models)
//! - cclab.titan (PostgreSQL ORM)
//! - cclab.nebula (MongoDB documents)
//! - cclab.photon (HTTP client)
//! - cclab.quasar (API routes)
//! - cclab.meteor (event handlers)
//!
pub mod meteor;
pub mod nebula;
pub mod photon;
pub mod quasar;
pub mod rust_scanner;
pub mod shield;
pub mod test_extractor;
pub mod titan;

pub use meteor::SwarmGenerator;
pub use nebula::NebulaGenerator;
pub use photon::PhotonGenerator;
pub use quasar::QuasarGenerator;
pub use rust_scanner::{
    RustEnum, RustEnumVariant, RustExports, RustField, RustFunction, RustMethod, RustParam,
    RustScanner, RustStruct, StructKind,
};
pub use shield::ShieldGenerator;
pub use test_extractor::{RustTest, TestExtractor, TestExtractorConfig};
pub use titan::TitanGenerator;

use crate::spec::ir::StringFormat;
use crate::type_inference::Type;

/// Convert Type IR to Python type annotation string
pub fn type_to_python(ty: &Type) -> String {
    match ty {
        Type::Never => "Never".to_string(),
        Type::None => "None".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Int => "int".to_string(),
        Type::Float => "float".to_string(),
        Type::Str => "str".to_string(),
        Type::Bytes => "bytes".to_string(),
        Type::List(inner) => format!("list[{}]", type_to_python(inner)),
        Type::Dict(key, value) => {
            format!("dict[{}, {}]", type_to_python(key), type_to_python(value))
        }
        Type::Set(inner) => format!("set[{}]", type_to_python(inner)),
        Type::Tuple(items) => {
            let items_str: Vec<String> = items.iter().map(type_to_python).collect();
            format!("tuple[{}]", items_str.join(", "))
        }
        Type::Optional(inner) => format!("Optional[{}]", type_to_python(inner)),
        Type::Union(types) => {
            if types.len() == 2 && types.contains(&Type::None) {
                let non_none = types.iter().find(|t| **t != Type::None).unwrap();
                format!("Optional[{}]", type_to_python(non_none))
            } else {
                let types_str: Vec<String> = types.iter().map(type_to_python).collect();
                format!("Union[{}]", types_str.join(", "))
            }
        }
        Type::Instance {
            name, type_args, ..
        } => {
            if type_args.is_empty() {
                name.clone()
            } else {
                let args_str: Vec<String> = type_args.iter().map(type_to_python).collect();
                format!("{}[{}]", name, args_str.join(", "))
            }
        }
        Type::Callable { params, ret } => {
            let params_str: Vec<String> = params.iter().map(|p| type_to_python(&p.ty)).collect();
            format!(
                "Callable[[{}], {}]",
                params_str.join(", "),
                type_to_python(ret)
            )
        }
        Type::Literal(lit) => {
            use crate::type_inference::LiteralValue;
            match lit {
                LiteralValue::Int(i) => format!("Literal[{}]", i),
                LiteralValue::Float(f) => format!("Literal[{}]", f),
                LiteralValue::Str(s) => format!("Literal[\"{}\"]", s),
                LiteralValue::Bool(b) => format!("Literal[{}]", if *b { "True" } else { "False" }),
                LiteralValue::None => "Literal[None]".to_string(),
            }
        }
        Type::Any => "Any".to_string(),
        Type::Unknown => "Any".to_string(),
        _ => "Any".to_string(),
    }
}

/// Convert StringFormat to Python type hint
pub fn format_to_python_type(format: &StringFormat) -> &'static str {
    match format {
        StringFormat::Email => "EmailStr",
        StringFormat::Uri | StringFormat::Url => "HttpUrl",
        StringFormat::Uuid => "UUID",
        StringFormat::DateTime => "datetime",
        StringFormat::Date => "date",
        StringFormat::Time => "time",
        _ => "str",
    }
}

/// Get imports needed for a type
pub fn get_type_imports(ty: &Type) -> Vec<&'static str> {
    let mut imports = Vec::new();

    match ty {
        Type::Optional(_) => imports.push("Optional"),
        Type::Union(_) => imports.push("Union"),
        Type::Callable { .. } => imports.push("Callable"),
        Type::Literal(_) => imports.push("Literal"),
        Type::Any | Type::Unknown => imports.push("Any"),
        _ => {}
    }

    // Recurse into container types
    match ty {
        Type::List(inner) | Type::Set(inner) | Type::Optional(inner) => {
            imports.extend(get_type_imports(inner));
        }
        Type::Dict(k, v) => {
            imports.extend(get_type_imports(k));
            imports.extend(get_type_imports(v));
        }
        Type::Union(types) | Type::Tuple(types) => {
            for t in types {
                imports.extend(get_type_imports(t));
            }
        }
        _ => {}
    }

    imports
}
