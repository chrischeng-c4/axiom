//! Rust code generators
//!
//! Generators for:
//! - serde (structs with serialization)
//! - sqlx (database models)
//! - axum (route handlers)
//! - reqwest (HTTP client)

pub mod axum;
pub mod reqwest;
pub mod serde;
pub mod sqlx;

pub use self::axum::AxumGenerator;
pub use self::reqwest::ReqwestGenerator;
pub use self::serde::SerdeGenerator;
pub use self::sqlx::SqlxGenerator;

use crate::spec::ir::StringFormat;
use crate::type_inference::Type;

/// Convert Type IR to Rust type string
pub fn type_to_rust(ty: &Type) -> String {
    match ty {
        Type::Never => "!".to_string(),
        Type::None => "()".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Int => "i64".to_string(),
        Type::Float => "f64".to_string(),
        Type::Str => "String".to_string(),
        Type::Bytes => "Vec<u8>".to_string(),
        Type::List(inner) => format!("Vec<{}>", type_to_rust(inner)),
        Type::Dict(key, value) => {
            format!("HashMap<{}, {}>", type_to_rust(key), type_to_rust(value))
        }
        Type::Set(inner) => format!("HashSet<{}>", type_to_rust(inner)),
        Type::Tuple(items) => {
            let items_str: Vec<String> = items.iter().map(type_to_rust).collect();
            format!("({})", items_str.join(", "))
        }
        Type::Optional(inner) => format!("Option<{}>", type_to_rust(inner)),
        Type::Union(types) => {
            // Rust doesn't have built-in union types, use enum or first type
            if types.len() == 2 && types.contains(&Type::None) {
                let non_none = types.iter().find(|t| **t != Type::None).unwrap();
                format!("Option<{}>", type_to_rust(non_none))
            } else {
                // For complex unions, we'd need to generate an enum
                "serde_json::Value".to_string()
            }
        }
        Type::Instance {
            name, type_args, ..
        } => {
            if type_args.is_empty() {
                name.clone()
            } else {
                let args_str: Vec<String> = type_args.iter().map(type_to_rust).collect();
                format!("{}<{}>", name, args_str.join(", "))
            }
        }
        Type::Literal(lit) => {
            // Rust doesn't have literal types like TypeScript
            use crate::type_inference::LiteralValue;
            match lit {
                LiteralValue::Int(_) => "i64".to_string(),
                LiteralValue::Float(_) => "f64".to_string(),
                LiteralValue::Str(_) => "String".to_string(),
                LiteralValue::Bool(_) => "bool".to_string(),
                LiteralValue::None => "()".to_string(),
            }
        }
        Type::Any | Type::Unknown => "serde_json::Value".to_string(),
        _ => "serde_json::Value".to_string(),
    }
}

/// Convert StringFormat to Rust type
pub fn format_to_rust_type(format: &StringFormat) -> &'static str {
    match format {
        StringFormat::Uuid => "uuid::Uuid",
        StringFormat::DateTime => "chrono::DateTime<chrono::Utc>",
        StringFormat::Date => "chrono::NaiveDate",
        StringFormat::Time => "chrono::NaiveTime",
        StringFormat::Duration => "std::time::Duration",
        StringFormat::Uri | StringFormat::Url => "url::Url",
        _ => "String",
    }
}
