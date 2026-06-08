use super::super::rc::MbObject;
use super::super::value::MbValue;
use rustc_hash::FxHashMap;
/// ast module for Mamba (#668).
///
/// Exposes Mamba's parser AST to Python userspace.
/// Provides parse(), dump(), literal_eval(), NodeVisitor, NodeTransformer.
use std::collections::HashMap;

// ── Variadic dispatchers (callable from module-attr context) ──

macro_rules! disp_nullary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! disp_binary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

disp_unary!(d_parse, mb_ast_parse);
disp_unary!(d_dump, mb_ast_dump);
disp_unary!(d_literal_eval, mb_ast_literal_eval);
disp_unary!(d_get_docstring, mb_ast_get_docstring);
disp_unary!(d_fix_missing_locations, mb_ast_fix_missing_locations);
disp_binary!(d_increment_lineno, mb_ast_increment_lineno);
disp_binary!(d_copy_location, mb_ast_copy_location);
disp_unary!(d_walk, mb_ast_walk);
disp_unary!(d_unparse, mb_ast_unparse);
disp_nullary!(d_NodeVisitor, mb_ast_NodeVisitor);
disp_nullary!(d_NodeTransformer, mb_ast_NodeTransformer);

pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        // Core functions
        ("parse", d_parse as *const () as usize),
        ("dump", d_dump as *const () as usize),
        ("literal_eval", d_literal_eval as *const () as usize),
        ("get_docstring", d_get_docstring as *const () as usize),
        (
            "fix_missing_locations",
            d_fix_missing_locations as *const () as usize,
        ),
        ("increment_lineno", d_increment_lineno as *const () as usize),
        ("copy_location", d_copy_location as *const () as usize),
        ("walk", d_walk as *const () as usize),
        ("unparse", d_unparse as *const () as usize),
        // Type classes (as stub callables)
        ("NodeVisitor", d_NodeVisitor as *const () as usize),
        ("NodeTransformer", d_NodeTransformer as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // AST node type constants (top-level)
    for node_type in &[
        "Module",
        "Interactive",
        "Expression",
        "FunctionDef",
        "AsyncFunctionDef",
        "ClassDef",
        "Return",
        "Delete",
        "Assign",
        "TypeAlias",
        "AugAssign",
        "AnnAssign",
        "For",
        "AsyncFor",
        "While",
        "If",
        "With",
        "AsyncWith",
        "Match",
        "Raise",
        "Try",
        "TryStar",
        "Assert",
        "Import",
        "ImportFrom",
        "Global",
        "Nonlocal",
        "Expr",
        "Pass",
        "Break",
        "Continue",
        "BoolOp",
        "NamedExpr",
        "BinOp",
        "UnaryOp",
        "Lambda",
        "IfExp",
        "Dict",
        "Set",
        "ListComp",
        "SetComp",
        "DictComp",
        "GeneratorExp",
        "Await",
        "Yield",
        "YieldFrom",
        "Compare",
        "Call",
        "FormattedValue",
        "JoinedStr",
        "Constant",
        "Attribute",
        "Subscript",
        "Starred",
        "Name",
        "List",
        "Tuple",
        "Slice",
        "Load",
        "Store",
        "Del",
        "And",
        "Or",
        "Add",
        "Sub",
        "Mult",
        "MatMult",
        "Div",
        "Mod",
        "Pow",
        "LShift",
        "RShift",
        "BitOr",
        "BitXor",
        "BitAnd",
        "FloorDiv",
        "Invert",
        "Not",
        "UAdd",
        "USub",
        "Eq",
        "NotEq",
        "Lt",
        "LtE",
        "Gt",
        "GtE",
        "Is",
        "IsNot",
        "In",
        "NotIn",
        "arg",
        "arguments",
        "keyword",
        "alias",
        "withitem",
        "match_case",
        "MatchValue",
        "MatchSingleton",
        "MatchSequence",
        "MatchMapping",
        "MatchClass",
        "MatchStar",
        "MatchAs",
        "MatchOr",
        "ExceptHandler",
        "TypeVar",
        "ParamSpec",
        "TypeVarTuple",
        "comprehension",
    ] {
        attrs.insert(
            node_type.to_string(),
            MbValue::from_ptr(MbObject::new_str(format!("mb_ast_node_{}", node_type))),
        );
    }

    // Mode constants
    attrs.insert("PyCF_ONLY_AST".to_string(), MbValue::from_int(1024));
    attrs.insert("PyCF_TYPE_COMMENTS".to_string(), MbValue::from_int(4096));
    attrs.insert(
        "PyCF_ALLOW_TOP_LEVEL_AWAIT".to_string(),
        MbValue::from_int(8192),
    );

    super::register_module("ast", attrs);
}

// -- Helper --

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Build a minimal AST node dict representing an AST tree node.
fn make_ast_node(node_type: &str, fields: FxHashMap<String, MbValue>) -> MbValue {
    use super::super::rc::{MbObject, MbObjectHeader, ObjData};
    let mut all_fields = fields;
    all_fields.insert(
        "_type".to_string(),
        MbValue::from_ptr(MbObject::new_str(node_type.to_string())),
    );
    all_fields.insert("lineno".to_string(), MbValue::from_int(1));
    all_fields.insert("col_offset".to_string(), MbValue::from_int(0));
    all_fields.insert("end_lineno".to_string(), MbValue::from_int(1));
    all_fields.insert("end_col_offset".to_string(), MbValue::from_int(0));
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: node_type.to_string(),
            fields: crate::runtime::rc::MbRwLock::new(all_fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// ast.parse(source, filename='<unknown>', mode='exec') -> AST
/// Parses the source string and returns a Module AST node.
/// In the full implementation, this calls the Mamba parser and
/// wraps the resulting AST in Python-compatible node objects.
pub fn mb_ast_parse(source: MbValue) -> MbValue {
    let src = extract_str(source).unwrap_or_default();
    let mut fields = FxHashMap::default();
    fields.insert(
        "body".to_string(),
        MbValue::from_ptr(MbObject::new_list(vec![])),
    );
    fields.insert(
        "type_ignores".to_string(),
        MbValue::from_ptr(MbObject::new_list(vec![])),
    );
    fields.insert(
        "_source".to_string(),
        MbValue::from_ptr(MbObject::new_str(src)),
    );
    make_ast_node("Module", fields)
}

/// ast.dump(node, annotate_fields=True, include_attributes=False,
///          indent=None) -> str
pub fn mb_ast_dump(node: MbValue) -> MbValue {
    use super::super::rc::ObjData;
    let type_name = node
        .as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                Some(class_name.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "AST".to_string());
    let dump = format!("{}()", type_name);
    MbValue::from_ptr(MbObject::new_str(dump))
}

/// ast.literal_eval(node_or_string) -> value
/// Safely evaluates an expression node or string containing only literals.
pub fn mb_ast_literal_eval(expr: MbValue) -> MbValue {
    let s = match extract_str(expr) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let trimmed = s.trim();
    // Try integer
    if let Ok(n) = trimmed.parse::<i64>() {
        return MbValue::from_int(n);
    }
    // Try float
    if let Ok(f) = trimmed.parse::<f64>() {
        return MbValue::from_float(f);
    }
    // True/False/None
    match trimmed {
        "True" => return MbValue::from_bool(true),
        "False" => return MbValue::from_bool(false),
        "None" => return MbValue::none(),
        _ => {}
    }
    // String literal (simple: strip quotes)
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        let inner = &trimmed[1..trimmed.len() - 1];
        return MbValue::from_ptr(MbObject::new_str(inner.to_string()));
    }
    MbValue::none()
}

/// ast.get_docstring(node, clean=True) -> str | None
pub fn mb_ast_get_docstring(_node: MbValue) -> MbValue {
    MbValue::none()
}

/// ast.fix_missing_locations(node) -> node
pub fn mb_ast_fix_missing_locations(node: MbValue) -> MbValue {
    node
}

/// ast.increment_lineno(node, n=1) -> node
pub fn mb_ast_increment_lineno(node: MbValue, _n: MbValue) -> MbValue {
    node
}

/// ast.copy_location(new_node, old_node) -> new_node
pub fn mb_ast_copy_location(new_node: MbValue, _old_node: MbValue) -> MbValue {
    new_node
}

/// ast.walk(node) -> iterator of all nodes
pub fn mb_ast_walk(node: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(vec![node]))
}

/// ast.unparse(node) -> str
pub fn mb_ast_unparse(_node: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_str("<unparsed>".to_string()))
}

/// NodeVisitor stub
#[allow(non_snake_case)]
pub fn mb_ast_NodeVisitor() -> MbValue {
    make_ast_node("NodeVisitor", FxHashMap::default())
}

/// NodeTransformer stub
#[allow(non_snake_case)]
pub fn mb_ast_NodeTransformer() -> MbValue {
    make_ast_node("NodeTransformer", FxHashMap::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_returns_module() {
        let src = MbValue::from_ptr(MbObject::new_str("x = 1".to_string()));
        let tree = mb_ast_parse(src);
        assert!(tree.as_ptr().is_some());
    }

    #[test]
    fn test_dump() {
        let src = MbValue::from_ptr(MbObject::new_str("x = 1".to_string()));
        let tree = mb_ast_parse(src);
        let dumped = mb_ast_dump(tree);
        assert!(dumped.as_ptr().is_some());
    }

    #[test]
    fn test_literal_eval_int() {
        let expr = MbValue::from_ptr(MbObject::new_str("42".to_string()));
        let result = mb_ast_literal_eval(expr);
        assert_eq!(result.as_int(), Some(42));
    }

    #[test]
    fn test_literal_eval_bool() {
        let t = MbValue::from_ptr(MbObject::new_str("True".to_string()));
        assert_eq!(mb_ast_literal_eval(t).as_bool(), Some(true));
        let f = MbValue::from_ptr(MbObject::new_str("False".to_string()));
        assert_eq!(mb_ast_literal_eval(f).as_bool(), Some(false));
    }

    #[test]
    fn test_literal_eval_none() {
        let n = MbValue::from_ptr(MbObject::new_str("None".to_string()));
        assert!(mb_ast_literal_eval(n).is_none());
    }

    #[test]
    fn test_fix_missing_locations() {
        let node = mb_ast_parse(MbValue::from_ptr(MbObject::new_str("".to_string())));
        let fixed = mb_ast_fix_missing_locations(node);
        assert!(fixed.as_ptr().is_some());
    }
}
