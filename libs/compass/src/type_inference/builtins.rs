//! Builtin type definitions for Python
//!
//! This module provides type bindings for Python builtin functions.

use super::ty::{Param, ParamKind, Type};
use super::type_env::TypeEnv;

/// Add Python builtin function types to the environment
pub fn add_builtins(env: &mut TypeEnv) {
    // Builtin functions
    env.bind(
        "len".to_string(),
        Type::callable(vec![Type::Any], Type::Int),
    );
    env.bind(
        "str".to_string(),
        Type::callable(vec![Type::Any], Type::Str),
    );
    env.bind(
        "int".to_string(),
        Type::callable(vec![Type::Any], Type::Int),
    );
    env.bind(
        "float".to_string(),
        Type::callable(vec![Type::Any], Type::Float),
    );
    env.bind(
        "bool".to_string(),
        Type::callable(vec![Type::Any], Type::Bool),
    );
    env.bind(
        "list".to_string(),
        Type::callable(vec![], Type::list(Type::Unknown)),
    );
    env.bind(
        "dict".to_string(),
        Type::callable(vec![], Type::dict(Type::Unknown, Type::Unknown)),
    );
    env.bind(
        "set".to_string(),
        Type::callable(vec![], Type::Set(Box::new(Type::Unknown))),
    );
    env.bind(
        "print".to_string(),
        Type::Callable {
            params: vec![Param {
                name: "values".to_string(),
                ty: Type::Any,
                has_default: false,
                kind: ParamKind::VarPositional,
            }],
            ret: Box::new(Type::None),
        },
    );
    env.bind(
        "range".to_string(),
        Type::callable(vec![Type::Int], Type::list(Type::Int)),
    );
    env.bind(
        "enumerate".to_string(),
        Type::callable(
            vec![Type::list(Type::Unknown)],
            Type::list(Type::Tuple(vec![Type::Int, Type::Unknown])),
        ),
    );
    env.bind(
        "zip".to_string(),
        Type::callable(
            vec![Type::list(Type::Unknown), Type::list(Type::Unknown)],
            Type::list(Type::Tuple(vec![Type::Unknown, Type::Unknown])),
        ),
    );
    env.bind(
        "isinstance".to_string(),
        Type::callable(vec![Type::Any, Type::Any], Type::Bool),
    );
    env.bind(
        "hasattr".to_string(),
        Type::callable(vec![Type::Any, Type::Str], Type::Bool),
    );
    env.bind(
        "getattr".to_string(),
        Type::callable(vec![Type::Any, Type::Str], Type::Any),
    );
}
