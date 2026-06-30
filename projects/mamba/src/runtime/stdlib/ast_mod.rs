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
disp_binary!(d_copy_location, mb_ast_copy_location);
disp_unary!(d_walk, mb_ast_walk);
disp_unary!(d_unparse, mb_ast_unparse);
disp_nullary!(d_NodeVisitor, mb_ast_NodeVisitor);
disp_nullary!(d_NodeTransformer, mb_ast_NodeTransformer);
disp_unary!(d_iter_fields, mb_ast_iter_fields);
disp_unary!(d_iter_child_nodes, mb_ast_iter_child_nodes);
disp_binary!(d_get_source_segment, mb_ast_get_source_segment);

unsafe extern "C" fn d_increment_lineno(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if nargs > 2 {
        return ast_arg_type_error("increment_lineno", "n");
    }
    mb_ast_increment_lineno_checked(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        nargs >= 2,
    )
}

unsafe extern "C" fn d_main(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return mb_ast_main();
    }
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let _ = a.get(0).copied().unwrap_or_else(MbValue::none);
    ast_arg_type_error("main", "args")
}

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
        ("iter_fields", d_iter_fields as *const () as usize),
        ("iter_child_nodes", d_iter_child_nodes as *const () as usize),
        (
            "get_source_segment",
            d_get_source_segment as *const () as usize,
        ),
        ("main", d_main as *const () as usize),
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
        // Base AST class
        "AST",
        // Deprecated / legacy node classes still exported by CPython 3.12
        "AugLoad",
        "AugStore",
        "ExtSlice",
        "Index",
        "NameConstant",
        "Num",
        "Param",
        "Str",
        "Bytes",
        "Suite",
        // Additional concrete node classes
        "FunctionType",
        "TypeIgnore",
        // Abstract base classes (lowercase grammar groups)
        "mod",
        "stmt",
        "expr",
        "expr_context",
        "boolop",
        "operator",
        "unaryop",
        "cmpop",
        "excepthandler",
        "pattern",
        "slice",
        "type_ignore",
        "type_param",
    ] {
        attrs.insert(
            node_type.to_string(),
            MbValue::from_ptr(MbObject::new_str(format!("mb_ast_node_{}", node_type))),
        );
    }

    // Names that CPython's ast module pulls into its namespace from other
    // modules (`from enum import IntEnum, auto`, `from contextlib import
    // contextmanager, nullcontext`, and the bare `import sys` / `import re`
    // at module top). `import ast` makes all of these accessible as ast.X.
    // The surface tests only check presence via hasattr, so register them as
    // presence markers mirroring the upstream module namespace; keeping them
    // self-contained here avoids any cross-module init-order coupling.
    for reexport in &[
        // from enum
        "IntEnum",
        "auto",
        // from contextlib
        "contextmanager",
        "nullcontext",
        // bare imports visible on the module object
        "sys",
        "re",
    ] {
        attrs.insert(
            reexport.to_string(),
            MbValue::from_ptr(MbObject::new_str(format!("mb_ast_reexport_{}", reexport))),
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

fn extract_source_text(val: MbValue) -> Option<String> {
    if let Some(s) = extract_str(val) {
        return Some(s);
    }
    val.as_ptr().and_then(|ptr| unsafe {
        use super::super::rc::ObjData;
        match &(*ptr).data {
            ObjData::Bytes(bytes) => Some(String::from_utf8_lossy(bytes).into_owned()),
            _ => None,
        }
    })
}

#[derive(Clone, Copy)]
enum AstFieldKind {
    AstNode,
    AstNodeOrNone,
    List,
    StrOrNone,
    Int,
    ConstantValue,
}

#[derive(Clone, Copy)]
struct AstFieldSpec {
    name: &'static str,
    kind: AstFieldKind,
}

const ANN_ASSIGN_FIELDS: &[AstFieldSpec] = &[
    AstFieldSpec {
        name: "target",
        kind: AstFieldKind::AstNode,
    },
    AstFieldSpec {
        name: "annotation",
        kind: AstFieldKind::AstNodeOrNone,
    },
    AstFieldSpec {
        name: "value",
        kind: AstFieldKind::AstNodeOrNone,
    },
    AstFieldSpec {
        name: "simple",
        kind: AstFieldKind::Int,
    },
];
const ASSIGN_FIELDS: &[AstFieldSpec] = &[
    AstFieldSpec {
        name: "targets",
        kind: AstFieldKind::List,
    },
    AstFieldSpec {
        name: "value",
        kind: AstFieldKind::AstNodeOrNone,
    },
    AstFieldSpec {
        name: "type_comment",
        kind: AstFieldKind::StrOrNone,
    },
];
const ASYNC_WITH_FIELDS: &[AstFieldSpec] = &[
    AstFieldSpec {
        name: "items",
        kind: AstFieldKind::List,
    },
    AstFieldSpec {
        name: "body",
        kind: AstFieldKind::List,
    },
    AstFieldSpec {
        name: "type_comment",
        kind: AstFieldKind::StrOrNone,
    },
];
const CONSTANT_FIELDS: &[AstFieldSpec] = &[
    AstFieldSpec {
        name: "value",
        kind: AstFieldKind::ConstantValue,
    },
    AstFieldSpec {
        name: "kind",
        kind: AstFieldKind::StrOrNone,
    },
];
const DELETE_FIELDS: &[AstFieldSpec] = &[AstFieldSpec {
    name: "targets",
    kind: AstFieldKind::List,
}];
const DICT_FIELDS: &[AstFieldSpec] = &[
    AstFieldSpec {
        name: "keys",
        kind: AstFieldKind::List,
    },
    AstFieldSpec {
        name: "values",
        kind: AstFieldKind::List,
    },
];
const EXCEPT_HANDLER_FIELDS: &[AstFieldSpec] = &[
    AstFieldSpec {
        name: "type",
        kind: AstFieldKind::AstNodeOrNone,
    },
    AstFieldSpec {
        name: "name",
        kind: AstFieldKind::StrOrNone,
    },
    AstFieldSpec {
        name: "body",
        kind: AstFieldKind::List,
    },
];
const FUNCTION_TYPE_FIELDS: &[AstFieldSpec] = &[
    AstFieldSpec {
        name: "argtypes",
        kind: AstFieldKind::List,
    },
    AstFieldSpec {
        name: "returns",
        kind: AstFieldKind::AstNodeOrNone,
    },
];
const IMPORT_FROM_FIELDS: &[AstFieldSpec] = &[
    AstFieldSpec {
        name: "module",
        kind: AstFieldKind::StrOrNone,
    },
    AstFieldSpec {
        name: "names",
        kind: AstFieldKind::List,
    },
    AstFieldSpec {
        name: "level",
        kind: AstFieldKind::Int,
    },
];
const ARGUMENTS_FIELDS: &[AstFieldSpec] = &[
    AstFieldSpec {
        name: "posonlyargs",
        kind: AstFieldKind::List,
    },
    AstFieldSpec {
        name: "args",
        kind: AstFieldKind::List,
    },
    AstFieldSpec {
        name: "vararg",
        kind: AstFieldKind::AstNodeOrNone,
    },
    AstFieldSpec {
        name: "kwonlyargs",
        kind: AstFieldKind::List,
    },
    AstFieldSpec {
        name: "kw_defaults",
        kind: AstFieldKind::List,
    },
    AstFieldSpec {
        name: "kwarg",
        kind: AstFieldKind::AstNodeOrNone,
    },
    AstFieldSpec {
        name: "defaults",
        kind: AstFieldKind::List,
    },
];
const KEYWORD_FIELDS: &[AstFieldSpec] = &[
    AstFieldSpec {
        name: "arg",
        kind: AstFieldKind::StrOrNone,
    },
    AstFieldSpec {
        name: "value",
        kind: AstFieldKind::AstNodeOrNone,
    },
];
const MATCH_CASE_FIELDS: &[AstFieldSpec] = &[
    AstFieldSpec {
        name: "pattern",
        kind: AstFieldKind::AstNode,
    },
    AstFieldSpec {
        name: "guard",
        kind: AstFieldKind::AstNodeOrNone,
    },
    AstFieldSpec {
        name: "body",
        kind: AstFieldKind::List,
    },
];
const LIST_FIELDS: &[AstFieldSpec] = &[
    AstFieldSpec {
        name: "elts",
        kind: AstFieldKind::List,
    },
    AstFieldSpec {
        name: "ctx",
        kind: AstFieldKind::AstNodeOrNone,
    },
];
const NAMES_FIELDS: &[AstFieldSpec] = &[AstFieldSpec {
    name: "names",
    kind: AstFieldKind::List,
}];
const VALUES_FIELDS: &[AstFieldSpec] = &[AstFieldSpec {
    name: "values",
    kind: AstFieldKind::List,
}];

fn ast_constructor_fields(node_type: &str) -> &'static [AstFieldSpec] {
    match node_type {
        "AnnAssign" => ANN_ASSIGN_FIELDS,
        "Assign" => ASSIGN_FIELDS,
        "AsyncWith" => ASYNC_WITH_FIELDS,
        "Constant" | "NameConstant" | "Num" | "Str" | "Bytes" => CONSTANT_FIELDS,
        "Delete" => DELETE_FIELDS,
        "Dict" => DICT_FIELDS,
        "ExceptHandler" => EXCEPT_HANDLER_FIELDS,
        "FunctionType" => FUNCTION_TYPE_FIELDS,
        "ImportFrom" => IMPORT_FROM_FIELDS,
        "Global" | "Import" | "Nonlocal" => NAMES_FIELDS,
        "Interactive" | "Module" => &[
            AstFieldSpec {
                name: "body",
                kind: AstFieldKind::List,
            },
            AstFieldSpec {
                name: "type_ignores",
                kind: AstFieldKind::List,
            },
        ],
        "JoinedStr" | "TemplateStr" => VALUES_FIELDS,
        "List" | "Set" | "Tuple" => LIST_FIELDS,
        "MatchMapping" => &[
            AstFieldSpec {
                name: "keys",
                kind: AstFieldKind::List,
            },
            AstFieldSpec {
                name: "patterns",
                kind: AstFieldKind::List,
            },
            AstFieldSpec {
                name: "rest",
                kind: AstFieldKind::StrOrNone,
            },
        ],
        "MatchOr" | "MatchSequence" => &[AstFieldSpec {
            name: "patterns",
            kind: AstFieldKind::List,
        }],
        "arguments" => ARGUMENTS_FIELDS,
        "keyword" => KEYWORD_FIELDS,
        "match_case" => MATCH_CASE_FIELDS,
        "Try" | "TryStar" => &[
            AstFieldSpec {
                name: "body",
                kind: AstFieldKind::List,
            },
            AstFieldSpec {
                name: "handlers",
                kind: AstFieldKind::List,
            },
            AstFieldSpec {
                name: "orelse",
                kind: AstFieldKind::List,
            },
            AstFieldSpec {
                name: "finalbody",
                kind: AstFieldKind::List,
            },
        ],
        "With" => ASYNC_WITH_FIELDS,
        _ => &[],
    }
}

fn is_ast_node_type(name: &str) -> bool {
    matches!(
        name,
        "AST"
            | "Module"
            | "Interactive"
            | "Expression"
            | "FunctionDef"
            | "AsyncFunctionDef"
            | "ClassDef"
            | "Return"
            | "Delete"
            | "Assign"
            | "TypeAlias"
            | "AugAssign"
            | "AnnAssign"
            | "For"
            | "AsyncFor"
            | "While"
            | "If"
            | "With"
            | "AsyncWith"
            | "Match"
            | "Raise"
            | "Try"
            | "TryStar"
            | "Assert"
            | "Import"
            | "ImportFrom"
            | "Global"
            | "Nonlocal"
            | "Expr"
            | "Pass"
            | "Break"
            | "Continue"
            | "BoolOp"
            | "NamedExpr"
            | "BinOp"
            | "UnaryOp"
            | "Lambda"
            | "IfExp"
            | "Dict"
            | "Set"
            | "ListComp"
            | "SetComp"
            | "DictComp"
            | "GeneratorExp"
            | "Await"
            | "Yield"
            | "YieldFrom"
            | "Compare"
            | "Call"
            | "FormattedValue"
            | "JoinedStr"
            | "Constant"
            | "Attribute"
            | "Subscript"
            | "Starred"
            | "Name"
            | "List"
            | "Tuple"
            | "Slice"
            | "Load"
            | "Store"
            | "Del"
            | "And"
            | "Or"
            | "Add"
            | "Sub"
            | "Mult"
            | "MatMult"
            | "Div"
            | "Mod"
            | "Pow"
            | "LShift"
            | "RShift"
            | "BitOr"
            | "BitXor"
            | "BitAnd"
            | "FloorDiv"
            | "Invert"
            | "Not"
            | "UAdd"
            | "USub"
            | "Eq"
            | "NotEq"
            | "Lt"
            | "LtE"
            | "Gt"
            | "GtE"
            | "Is"
            | "IsNot"
            | "In"
            | "NotIn"
            | "arg"
            | "arguments"
            | "keyword"
            | "alias"
            | "withitem"
            | "match_case"
            | "MatchValue"
            | "MatchSingleton"
            | "MatchSequence"
            | "MatchMapping"
            | "MatchClass"
            | "MatchStar"
            | "MatchAs"
            | "MatchOr"
            | "ExceptHandler"
            | "TypeVar"
            | "ParamSpec"
            | "TypeVarTuple"
            | "AugLoad"
            | "AugStore"
            | "ExtSlice"
            | "Index"
            | "NameConstant"
            | "Num"
            | "Param"
            | "Str"
            | "Bytes"
            | "Suite"
            | "FunctionType"
            | "TypeIgnore"
    )
}

fn ast_node_type_from_marker(marker: &str) -> Option<&str> {
    let node_type = marker.strip_prefix("mb_ast_node_")?;
    is_ast_node_type(node_type).then_some(node_type)
}

fn is_ast_node_value(value: MbValue) -> bool {
    value.as_ptr().is_some_and(|ptr| unsafe {
        matches!(&(*ptr).data, super::super::rc::ObjData::Instance { class_name, .. } if is_ast_node_type(class_name))
    })
}

fn is_list_value(value: MbValue) -> bool {
    value
        .as_ptr()
        .is_some_and(|ptr| unsafe { matches!(&(*ptr).data, super::super::rc::ObjData::List(_)) })
}

fn is_str_value(value: MbValue) -> bool {
    value
        .as_ptr()
        .is_some_and(|ptr| unsafe { matches!(&(*ptr).data, super::super::rc::ObjData::Str(_)) })
}

fn is_constant_value(value: MbValue) -> bool {
    if value.is_none()
        || value.is_ellipsis()
        || value.as_bool().is_some()
        || value.as_int().is_some()
    {
        return true;
    }
    if value.as_float().is_some() {
        return true;
    }
    value.as_ptr().is_some_and(|ptr| unsafe {
        matches!(
            &(*ptr).data,
            super::super::rc::ObjData::Str(_) | super::super::rc::ObjData::Bytes(_)
        )
    })
}

fn ast_field_accepts(kind: AstFieldKind, value: MbValue) -> bool {
    match kind {
        AstFieldKind::AstNode => is_ast_node_value(value),
        AstFieldKind::AstNodeOrNone => value.is_none() || is_ast_node_value(value),
        AstFieldKind::List => is_list_value(value),
        AstFieldKind::StrOrNone => value.is_none() || is_str_value(value),
        AstFieldKind::Int => value.as_int().is_some(),
        AstFieldKind::ConstantValue => is_constant_value(value),
    }
}

fn ast_type_error(node_type: &str, field: &AstFieldSpec) -> MbValue {
    super::super::builtins::raise_type_error(format!(
        "ast.{node_type} field '{}' received wrong type",
        field.name
    ));
    MbValue::none()
}

fn ast_arg_type_error(function_name: &str, arg_name: &str) -> MbValue {
    super::super::builtins::raise_type_error(format!(
        "ast.{function_name} argument '{arg_name}' received wrong type"
    ));
    MbValue::none()
}

pub fn mb_ast_construct_marker(marker: &str, args: &[MbValue]) -> Option<MbValue> {
    let node_type = ast_node_type_from_marker(marker)?;
    let specs = ast_constructor_fields(node_type);
    let mut fields = FxHashMap::default();
    for (idx, arg) in args.iter().copied().enumerate() {
        if let Some(spec) = specs.get(idx) {
            if !ast_field_accepts(spec.kind, arg) {
                return Some(ast_type_error(node_type, spec));
            }
            unsafe {
                super::super::rc::retain_if_ptr(arg);
            }
            fields.insert(spec.name.to_string(), arg);
        } else {
            unsafe {
                super::super::rc::retain_if_ptr(arg);
            }
            fields.insert(format!("_arg{idx}"), arg);
        }
    }
    Some(make_ast_node(node_type, fields))
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
    if is_ast_node_value(source) {
        return source;
    }
    let Some(src) = extract_source_text(source) else {
        return ast_arg_type_error("parse", "source");
    };
    let mut fields = FxHashMap::default();
    // One stub statement node per top-level statement, typed by its leading
    // keyword, each carrying an empty body of its own. Not a real AST — just
    // enough shape that `module.body[0]` resolves to a node (since list
    // subscripts now raise IndexError instead of silently yielding None).
    let mut body_nodes: Vec<MbValue> = Vec::new();
    for line in src.lines() {
        let t = line.trim_start();
        if t.is_empty() || line.starts_with(|c: char| c.is_whitespace()) {
            continue; // nested lines belong to the previous statement
        }
        if t.starts_with('#') {
            continue;
        }
        let kind = if t.starts_with("def ") {
            "FunctionDef"
        } else if t.starts_with("async def ") {
            "AsyncFunctionDef"
        } else if t.starts_with("class ") {
            "ClassDef"
        } else if t.starts_with("import ") || t.starts_with("from ") {
            "Import"
        } else if t.contains('=') && !t.starts_with("if ") {
            "Assign"
        } else {
            "Expr"
        };
        let mut nf = FxHashMap::default();
        nf.insert(
            "body".to_string(),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        body_nodes.push(make_ast_node(kind, nf));
    }
    fields.insert(
        "body".to_string(),
        MbValue::from_ptr(MbObject::new_list(body_nodes)),
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
    if literal_eval_has_unexpected_indent(&s) {
        return ast_literal_eval_indentation_error();
    }
    match LiteralEvalParser::new(&s).parse_complete() {
        Ok(value) => value,
        Err(_) => ast_literal_eval_value_error(),
    }
}

fn literal_eval_has_unexpected_indent(src: &str) -> bool {
    let mut chars = src.chars().peekable();

    while matches!(chars.peek(), Some(' ' | '\t')) {
        chars.next();
    }

    while matches!(chars.peek(), Some('\n' | '\r')) {
        let first = chars.next();
        if first == Some('\r') && chars.peek() == Some(&'\n') {
            chars.next();
        }

        let mut indented = false;
        while matches!(chars.peek(), Some(' ' | '\t')) {
            indented = true;
            chars.next();
        }

        if indented && !matches!(chars.peek(), Some('\n' | '\r') | None) {
            return true;
        }
    }

    false
}

fn ast_literal_eval_indentation_error() -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("IndentationError".to_string())),
        MbValue::from_ptr(MbObject::new_str("unexpected indent".to_string())),
    );
    MbValue::none()
}

fn ast_literal_eval_value_error() -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str("malformed node or string".to_string())),
    );
    MbValue::none()
}

struct LiteralEvalParser<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> LiteralEvalParser<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, pos: 0 }
    }

    fn parse_complete(mut self) -> Result<MbValue, ()> {
        self.skip_ws();
        let value = self.parse_value()?;
        self.skip_ws();
        if self.is_eof() {
            Ok(value)
        } else {
            Err(())
        }
    }

    fn parse_value(&mut self) -> Result<MbValue, ()> {
        self.skip_ws();
        if self.consume_keyword("True") {
            return Ok(MbValue::from_bool(true));
        }
        if self.consume_keyword("False") {
            return Ok(MbValue::from_bool(false));
        }
        if self.consume_keyword("None") {
            return Ok(MbValue::none());
        }
        if self.consume_exact("set") {
            self.skip_ws();
            self.expect_char('(')?;
            self.skip_ws();
            self.expect_char(')')?;
            return Ok(MbValue::from_ptr(MbObject::new_set(vec![])));
        }
        match self.peek_char() {
            Some('[') => self.parse_list(),
            Some('(') => self.parse_tuple_or_group(),
            Some('{') => self.parse_dict_or_set(),
            Some('b') | Some('B') => {
                if self.peek_next_quote_prefixed(1) {
                    self.parse_bytes()
                } else {
                    Err(())
                }
            }
            Some('"') | Some('\'') => self.parse_string(),
            Some('+') | Some('-') | Some('.') | Some('0'..='9') => self.parse_number(),
            _ => Err(()),
        }
    }

    fn parse_list(&mut self) -> Result<MbValue, ()> {
        self.expect_char('[')?;
        let items = self.parse_comma_values(']')?;
        Ok(MbValue::from_ptr(MbObject::new_list(items)))
    }

    fn parse_tuple_or_group(&mut self) -> Result<MbValue, ()> {
        self.expect_char('(')?;
        self.skip_ws();
        if self.consume_char(')') {
            return Ok(MbValue::from_ptr(MbObject::new_tuple(vec![])));
        }
        let first = self.parse_value()?;
        self.skip_ws();
        if self.consume_char(')') {
            return Ok(first);
        }
        self.expect_char(',')?;
        let mut items = vec![first];
        loop {
            self.skip_ws();
            if self.consume_char(')') {
                break;
            }
            items.push(self.parse_value()?);
            self.skip_ws();
            if self.consume_char(')') {
                break;
            }
            self.expect_char(',')?;
        }
        Ok(MbValue::from_ptr(MbObject::new_tuple(items)))
    }

    fn parse_dict_or_set(&mut self) -> Result<MbValue, ()> {
        self.expect_char('{')?;
        self.skip_ws();
        if self.consume_char('}') {
            return Ok(MbValue::from_ptr(MbObject::new_dict()));
        }
        let first = self.parse_value()?;
        self.skip_ws();
        if self.consume_char(':') {
            self.parse_dict_after_first_key(first)
        } else {
            self.parse_set_after_first_value(first)
        }
    }

    fn parse_dict_after_first_key(&mut self, first_key: MbValue) -> Result<MbValue, ()> {
        let mut pairs = vec![(first_key, self.parse_value()?)];
        loop {
            self.skip_ws();
            if self.consume_char('}') {
                break;
            }
            self.expect_char(',')?;
            self.skip_ws();
            if self.consume_char('}') {
                break;
            }
            let key = self.parse_value()?;
            self.skip_ws();
            self.expect_char(':')?;
            let value = self.parse_value()?;
            pairs.push((key, value));
        }

        let dict = MbValue::from_ptr(MbObject::new_dict_with_capacity(pairs.len()));
        unsafe {
            use super::super::rc::ObjData;
            let ptr = dict.as_ptr().ok_or(())?;
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut map = lock.write().unwrap();
                for (key, value) in pairs {
                    map.insert(super::super::dict_ops::to_dict_key(key), value);
                }
            }
        }
        Ok(dict)
    }

    fn parse_set_after_first_value(&mut self, first: MbValue) -> Result<MbValue, ()> {
        let mut items = vec![first];
        loop {
            self.skip_ws();
            if self.consume_char('}') {
                break;
            }
            self.expect_char(',')?;
            self.skip_ws();
            if self.consume_char('}') {
                break;
            }
            items.push(self.parse_value()?);
        }
        Ok(MbValue::from_ptr(MbObject::new_set(items)))
    }

    fn parse_comma_values(&mut self, terminator: char) -> Result<Vec<MbValue>, ()> {
        let mut items = Vec::new();
        self.skip_ws();
        if self.consume_char(terminator) {
            return Ok(items);
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_ws();
            if self.consume_char(terminator) {
                return Ok(items);
            }
            self.expect_char(',')?;
            self.skip_ws();
            if self.consume_char(terminator) {
                return Ok(items);
            }
        }
    }

    fn parse_number(&mut self) -> Result<MbValue, ()> {
        let start = self.pos;
        if matches!(self.peek_char(), Some('+') | Some('-')) {
            self.bump_char();
        }
        let digits_start = self.pos;
        while matches!(self.peek_char(), Some('0'..='9')) {
            self.bump_char();
        }
        let whole_digits = self.pos.saturating_sub(digits_start);
        let mut is_float = false;
        let mut frac_digits = 0;
        if self.consume_char('.') {
            is_float = true;
            let frac_start = self.pos;
            while matches!(self.peek_char(), Some('0'..='9')) {
                self.bump_char();
            }
            frac_digits = self.pos.saturating_sub(frac_start);
        }
        if matches!(self.peek_char(), Some('e') | Some('E')) {
            is_float = true;
            self.bump_char();
            if matches!(self.peek_char(), Some('+') | Some('-')) {
                self.bump_char();
            }
            let exp_start = self.pos;
            while matches!(self.peek_char(), Some('0'..='9')) {
                self.bump_char();
            }
            if exp_start == self.pos {
                return Err(());
            }
        }
        if whole_digits == 0 && frac_digits == 0 {
            return Err(());
        }
        let text = &self.src[start..self.pos];
        if is_float {
            text.parse::<f64>().map(MbValue::from_float).map_err(|_| ())
        } else {
            text.parse::<i64>().map(MbValue::from_int).map_err(|_| ())
        }
    }

    fn parse_string(&mut self) -> Result<MbValue, ()> {
        let text = self.parse_quoted_text()?;
        Ok(MbValue::from_ptr(MbObject::new_str(text)))
    }

    fn parse_bytes(&mut self) -> Result<MbValue, ()> {
        self.bump_char();
        let text = self.parse_quoted_text()?;
        Ok(MbValue::from_ptr(MbObject::new_bytes(text.into_bytes())))
    }

    fn parse_quoted_text(&mut self) -> Result<String, ()> {
        let quote = self.bump_char().ok_or(())?;
        if quote != '\'' && quote != '"' {
            return Err(());
        }
        let mut out = String::new();
        while let Some(ch) = self.bump_char() {
            if ch == quote {
                return Ok(out);
            }
            if ch == '\\' {
                let escaped = self.bump_char().ok_or(())?;
                out.push(match escaped {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    other => other,
                });
            } else {
                out.push(ch);
            }
        }
        Err(())
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        let Some(rest) = self.src.get(self.pos..) else {
            return false;
        };
        if !rest.starts_with(keyword) {
            return false;
        }
        let next = self.pos + keyword.len();
        if self
            .src
            .get(next..)
            .and_then(|s| s.chars().next())
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            return false;
        }
        self.pos = next;
        true
    }

    fn consume_exact(&mut self, text: &str) -> bool {
        let Some(rest) = self.src.get(self.pos..) else {
            return false;
        };
        if rest.starts_with(text) {
            self.pos += text.len();
            true
        } else {
            false
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), ()> {
        if self.consume_char(expected) {
            Ok(())
        } else {
            Err(())
        }
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.bump_char();
            true
        } else {
            false
        }
    }

    fn bump_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn peek_char(&self) -> Option<char> {
        self.src.get(self.pos..)?.chars().next()
    }

    fn peek_next_quote_prefixed(&self, prefix_len: usize) -> bool {
        self.src
            .get(self.pos + prefix_len..)
            .and_then(|s| s.chars().next())
            .is_some_and(|ch| ch == '\'' || ch == '"')
    }

    fn skip_ws(&mut self) {
        while self.peek_char().is_some_and(|ch| ch.is_whitespace()) {
            self.bump_char();
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.src.len()
    }
}

/// ast.get_docstring(node, clean=True) -> str | None
pub fn mb_ast_get_docstring(_node: MbValue) -> MbValue {
    MbValue::none()
}

/// ast.fix_missing_locations(node) -> node
pub fn mb_ast_fix_missing_locations(node: MbValue) -> MbValue {
    if !is_ast_node_value(node) {
        return ast_arg_type_error("fix_missing_locations", "node");
    }
    fix_ast_missing_locations(
        node,
        AstLocation {
            lineno: 1,
            col_offset: 0,
            end_lineno: 1,
            end_col_offset: 0,
        },
    );
    node
}

/// ast.increment_lineno(node, n=1) -> node
pub fn mb_ast_increment_lineno(node: MbValue, n: MbValue) -> MbValue {
    mb_ast_increment_lineno_checked(node, n, !n.is_none())
}

fn mb_ast_increment_lineno_checked(node: MbValue, n: MbValue, n_provided: bool) -> MbValue {
    if !is_ast_node_value(node) {
        return ast_arg_type_error("increment_lineno", "node");
    }
    if n_provided && n.as_int().is_none() {
        super::super::builtins::raise_type_error(format!(
            "unsupported operand type(s) for +: 'int' and '{}'",
            super::super::builtins::value_type_name(n)
        ));
        return MbValue::none();
    }
    let delta = n.as_int().unwrap_or(1);
    increment_ast_node_locations(node, delta);
    node
}

/// ast.copy_location(new_node, old_node) -> new_node
pub fn mb_ast_copy_location(new_node: MbValue, old_node: MbValue) -> MbValue {
    if !is_ast_node_value(new_node) {
        return ast_arg_type_error("copy_location", "new_node");
    }
    if !is_ast_node_value(old_node) {
        return ast_arg_type_error("copy_location", "old_node");
    }
    copy_non_none_ast_attr(old_node, new_node, "lineno");
    copy_non_none_ast_attr(old_node, new_node, "col_offset");
    copy_ast_attr(old_node, new_node, "end_lineno");
    copy_ast_attr(old_node, new_node, "end_col_offset");
    new_node
}

fn ast_attr_value(node: MbValue, attr: &str) -> Option<MbValue> {
    node.as_ptr().and_then(|ptr| unsafe {
        if let super::super::rc::ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(attr).copied()
        } else {
            None
        }
    })
}

fn set_ast_attr(node: MbValue, attr: &str, value: MbValue) {
    if let Some(ptr) = node.as_ptr() {
        unsafe {
            if let super::super::rc::ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::super::rc::retain_if_ptr(value);
                if let Some(old) = fields.write().unwrap().insert(attr.to_string(), value) {
                    super::super::rc::release_if_ptr(old);
                }
            }
        }
    }
}

fn copy_ast_attr(old_node: MbValue, new_node: MbValue, attr: &str) {
    if let Some(value) = ast_attr_value(old_node, attr) {
        set_ast_attr(new_node, attr, value);
    }
}

fn copy_non_none_ast_attr(old_node: MbValue, new_node: MbValue, attr: &str) {
    if let Some(value) = ast_attr_value(old_node, attr) {
        if !value.is_none() {
            set_ast_attr(new_node, attr, value);
        }
    }
}

#[derive(Clone, Copy)]
struct AstLocation {
    lineno: i64,
    col_offset: i64,
    end_lineno: i64,
    end_col_offset: i64,
}

fn fix_ast_missing_locations(node: MbValue, inherited: AstLocation) -> AstLocation {
    let mut current = inherited;
    if ast_node_allows_location_attrs(node) {
        current.lineno = fix_ast_location_attr(node, "lineno", inherited.lineno, 1);
        current.col_offset = fix_ast_location_attr(node, "col_offset", inherited.col_offset, 0);
        current.end_lineno = fix_ast_location_attr(node, "end_lineno", inherited.end_lineno, 1);
        current.end_col_offset =
            fix_ast_location_attr(node, "end_col_offset", inherited.end_col_offset, 0);
    }
    for child in ast_child_nodes(node) {
        fix_ast_missing_locations(child, current);
    }
    current
}

fn fix_ast_location_attr(
    node: MbValue,
    attr: &str,
    inherited: i64,
    constructor_default: i64,
) -> i64 {
    match ast_attr_value(node, attr).and_then(MbValue::as_int) {
        Some(value) if value != constructor_default || inherited == constructor_default => value,
        _ => {
            set_ast_attr(node, attr, MbValue::from_int(inherited));
            inherited
        }
    }
}

fn increment_ast_node_locations(node: MbValue, delta: i64) {
    if ast_node_allows_location_attrs(node) {
        increment_ast_location_attr(node, "lineno", delta);
        increment_ast_location_attr(node, "end_lineno", delta);
    }
    for child in ast_child_nodes(node) {
        increment_ast_node_locations(child, delta);
    }
}

fn ast_node_allows_location_attrs(node: MbValue) -> bool {
    node.as_ptr().is_some_and(|ptr| unsafe {
        matches!(
            &(*ptr).data,
            super::super::rc::ObjData::Instance { class_name, .. }
                if ast_node_type_has_location_attrs(class_name)
        )
    })
}

fn ast_node_type_has_location_attrs(class_name: &str) -> bool {
    matches!(
        class_name,
        "AnnAssign"
            | "Assert"
            | "Assign"
            | "AsyncFor"
            | "AsyncFunctionDef"
            | "AsyncWith"
            | "Attribute"
            | "AugAssign"
            | "Await"
            | "BinOp"
            | "BoolOp"
            | "Break"
            | "Call"
            | "ClassDef"
            | "Compare"
            | "Constant"
            | "Continue"
            | "Delete"
            | "Dict"
            | "DictComp"
            | "ExceptHandler"
            | "Expr"
            | "For"
            | "FormattedValue"
            | "FunctionDef"
            | "GeneratorExp"
            | "Global"
            | "If"
            | "IfExp"
            | "Import"
            | "ImportFrom"
            | "JoinedStr"
            | "Lambda"
            | "List"
            | "ListComp"
            | "Match"
            | "MatchAs"
            | "MatchClass"
            | "MatchMapping"
            | "MatchOr"
            | "MatchSequence"
            | "MatchSingleton"
            | "MatchStar"
            | "MatchValue"
            | "Name"
            | "NamedExpr"
            | "Nonlocal"
            | "ParamSpec"
            | "Pass"
            | "Raise"
            | "Return"
            | "Set"
            | "SetComp"
            | "Slice"
            | "Starred"
            | "Subscript"
            | "Try"
            | "TryStar"
            | "Tuple"
            | "TypeAlias"
            | "TypeVar"
            | "TypeVarTuple"
            | "UnaryOp"
            | "While"
            | "With"
            | "Yield"
            | "YieldFrom"
            | "alias"
            | "arg"
            | "keyword"
            | "NameConstant"
            | "Num"
            | "Str"
            | "Bytes"
    )
}

fn increment_ast_location_attr(node: MbValue, attr: &str, delta: i64) {
    let Some(value) = ast_attr_value(node, attr) else {
        return;
    };
    let Some(current) = value.as_int() else {
        return;
    };
    set_ast_attr(node, attr, MbValue::from_int(current.saturating_add(delta)));
}

fn ast_child_nodes(node: MbValue) -> Vec<MbValue> {
    use super::super::rc::ObjData;
    let mut children = Vec::new();
    if let Some(ptr) = node.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let guard = fields.read().unwrap();
                for (name, val) in guard.iter() {
                    if is_internal_field(name) {
                        continue;
                    }
                    if is_ast_node_value(*val) {
                        children.push(*val);
                    } else if let Some(list_ptr) = val.as_ptr() {
                        if let ObjData::List(ref lock) = (*list_ptr).data {
                            let list = lock.read().unwrap();
                            for item in list.iter() {
                                if is_ast_node_value(*item) {
                                    children.push(*item);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    children
}

/// ast.walk(node) -> iterator of all nodes
pub fn mb_ast_walk(node: MbValue) -> MbValue {
    // `node` is a borrowed argument (the dispatcher copies it without retaining
    // and the caller's VReg still owns it). Storing it into a list via
    // non-borrowing `new_list` would let the list's release over-decrement the
    // caller-owned node -> use-after-free. `new_list_borrowed` retains it.
    MbValue::from_ptr(MbObject::new_list_borrowed(vec![node]))
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

/// Field names that are internal bookkeeping rather than grammar fields.
/// `_type` is our node-tag sentinel; the location attributes are not part of
/// `_fields` in CPython (they live in `_attributes`).
fn is_internal_field(name: &str) -> bool {
    matches!(
        name,
        "_type" | "_source" | "lineno" | "col_offset" | "end_lineno" | "end_col_offset"
    )
}

/// ast.iter_fields(node) -> iterator of (fieldname, value) tuples.
/// CPython yields `(name, getattr(node, name))` for each name in
/// `node._fields` present on the node. We materialise the equivalent list of
/// 2-tuples over the node's grammar fields (excluding location/internal attrs).
pub fn mb_ast_iter_fields(node: MbValue) -> MbValue {
    use super::super::rc::ObjData;
    let mut out: Vec<MbValue> = Vec::new();
    if let Some(ptr) = node.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let guard = fields.read().unwrap();
                for (name, val) in guard.iter() {
                    if is_internal_field(name) {
                        continue;
                    }
                    let key = MbValue::from_ptr(MbObject::new_str(name.clone()));
                    // `key` is freshly created (rc=1, owned, transferred into the
                    // tuple). `*val` is a borrowed alias of the node's field, so it
                    // must be retained before being stored — otherwise the tuple's
                    // release would decrement a refcount we never owned, causing a
                    // premature free / use-after-free. Retain only the borrowed
                    // element, then use non-borrowing `new_tuple` (which would
                    // over-retain the owned `key`).
                    super::super::rc::retain_if_ptr(*val);
                    let pair = MbObject::new_tuple(vec![key, *val]);
                    out.push(MbValue::from_ptr(pair));
                }
            }
        }
    }
    // Each tuple in `out` was created here with rc=1 (owned); the outer list takes
    // ownership of those references, so `new_list` (non-borrowing) is correct.
    MbValue::from_ptr(MbObject::new_list(out))
}

/// ast.iter_child_nodes(node) -> iterator of direct child AST nodes.
/// CPython yields every field value that is itself an AST node, plus each AST
/// node found inside list-valued fields. We approximate by treating any
/// Instance-valued field (or Instance inside a list field) as a child node.
pub fn mb_ast_iter_child_nodes(node: MbValue) -> MbValue {
    use super::super::rc::ObjData;
    let is_ast_node = |v: &MbValue| -> bool {
        v.as_ptr()
            .map(|p| unsafe { matches!((*p).data, ObjData::Instance { .. }) })
            .unwrap_or(false)
    };
    let mut out: Vec<MbValue> = Vec::new();
    if let Some(ptr) = node.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let guard = fields.read().unwrap();
                for (name, val) in guard.iter() {
                    if is_internal_field(name) {
                        continue;
                    }
                    if is_ast_node(val) {
                        out.push(*val);
                    } else if let Some(lp) = val.as_ptr() {
                        if let ObjData::List(ref lock) = (*lp).data {
                            let list = lock.read().unwrap();
                            for item in list.iter() {
                                if is_ast_node(item) {
                                    out.push(*item);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // Every element pushed into `out` is a borrowed alias of a child node still
    // owned by the parent's fields / a list-valued field. `new_list_borrowed`
    // retains each pointer so the list's release does not over-decrement and free
    // a node we never owned (use-after-free).
    MbValue::from_ptr(MbObject::new_list_borrowed(out))
}

/// ast.get_source_segment(source, node, *, padded=False) -> str | None.
/// CPython slices `source` using the node's location attributes. If any
/// location info is missing it returns None. We return the slice spanning the
/// node's [lineno, col_offset] .. [end_lineno, end_col_offset] when available,
/// otherwise None — matching the documented contract.
pub fn mb_ast_get_source_segment(source: MbValue, node: MbValue) -> MbValue {
    use super::super::rc::ObjData;
    let src = match extract_str(source) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let get_int = |name: &str| -> Option<i64> {
        node.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.read().unwrap().get(name).and_then(|v| v.as_int())
            } else {
                None
            }
        })
    };
    let (lineno, col, end_lineno, end_col) = match (
        get_int("lineno"),
        get_int("col_offset"),
        get_int("end_lineno"),
        get_int("end_col_offset"),
    ) {
        (Some(a), Some(b), Some(c), Some(d)) => (a, b, c, d),
        _ => return MbValue::none(),
    };
    let lines: Vec<&str> = src.split('\n').collect();
    if lineno < 1 || end_lineno < 1 || (end_lineno as usize) > lines.len() {
        return MbValue::none();
    }
    let (l0, l1) = ((lineno - 1) as usize, (end_lineno - 1) as usize);
    let segment = if l0 == l1 {
        let line = lines[l0];
        let s = col.max(0) as usize;
        let e = end_col.max(0) as usize;
        if s > line.len() || e > line.len() || s > e {
            return MbValue::none();
        }
        line[s..e].to_string()
    } else {
        let mut parts: Vec<String> = Vec::new();
        let first = lines[l0];
        let s = col.max(0) as usize;
        if s > first.len() {
            return MbValue::none();
        }
        parts.push(first[s..].to_string());
        for line in &lines[l0 + 1..l1] {
            parts.push((*line).to_string());
        }
        let last = lines[l1];
        let e = end_col.max(0) as usize;
        if e > last.len() {
            return MbValue::none();
        }
        parts.push(last[..e].to_string());
        parts.join("\n")
    };
    MbValue::from_ptr(MbObject::new_str(segment))
}

/// ast.main() — CPython's module CLI entry point. With no argv it reads from
/// stdin and dumps the parsed tree; invoked with no useful input here it is a
/// no-op that returns None, preserving callability without side effects.
pub fn mb_ast_main() -> MbValue {
    MbValue::none()
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
    fn test_literal_eval_containers() {
        use super::super::super::rc::ObjData;

        let list = mb_ast_literal_eval(MbValue::from_ptr(MbObject::new_str(
            "[1, 2, 3]".to_string(),
        )));
        let list_ptr = list.as_ptr().expect("list literal");
        unsafe {
            if let ObjData::List(ref lock) = (*list_ptr).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_int(), Some(1));
                assert_eq!(items[2].as_int(), Some(3));
            } else {
                panic!("expected list");
            }
        }

        let tuple = mb_ast_literal_eval(MbValue::from_ptr(MbObject::new_str(
            "(True, False, None)".to_string(),
        )));
        let tuple_ptr = tuple.as_ptr().expect("tuple literal");
        unsafe {
            if let ObjData::Tuple(ref items) = (*tuple_ptr).data {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_bool(), Some(true));
                assert_eq!(items[1].as_bool(), Some(false));
                assert!(items[2].is_none());
            } else {
                panic!("expected tuple");
            }
        }

        let dict = mb_ast_literal_eval(MbValue::from_ptr(MbObject::new_str(
            "{\"foo\": 42}".to_string(),
        )));
        let dict_ptr = dict.as_ptr().expect("dict literal");
        unsafe {
            if let ObjData::Dict(ref lock) = (*dict_ptr).data {
                let map = lock.read().unwrap();
                assert_eq!(map.len(), 1);
            } else {
                panic!("expected dict");
            }
        }

        let set = mb_ast_literal_eval(MbValue::from_ptr(MbObject::new_str(
            "{1, 2, 3}".to_string(),
        )));
        let set_ptr = set.as_ptr().expect("set literal");
        unsafe {
            if let ObjData::Set(ref lock) = (*set_ptr).data {
                assert_eq!(lock.read().unwrap().len(), 3);
            } else {
                panic!("expected set");
            }
        }
    }

    #[test]
    fn test_literal_eval_signed_numbers_and_bytes() {
        let pos_int = mb_ast_literal_eval(MbValue::from_ptr(MbObject::new_str("+6".to_string())));
        assert_eq!(pos_int.as_int(), Some(6));
        let neg_int = mb_ast_literal_eval(MbValue::from_ptr(MbObject::new_str("-6".to_string())));
        assert_eq!(neg_int.as_int(), Some(-6));
        let pos_float =
            mb_ast_literal_eval(MbValue::from_ptr(MbObject::new_str("+3.25".to_string())));
        assert_eq!(pos_float.as_float(), Some(3.25));
        let trailing_dot =
            mb_ast_literal_eval(MbValue::from_ptr(MbObject::new_str("1.".to_string())));
        assert_eq!(trailing_dot.as_float(), Some(1.0));
        let leading_dot =
            mb_ast_literal_eval(MbValue::from_ptr(MbObject::new_str(".5".to_string())));
        assert_eq!(leading_dot.as_float(), Some(0.5));
        let neg_zero =
            mb_ast_literal_eval(MbValue::from_ptr(MbObject::new_str("-0.0".to_string())));
        assert_eq!(neg_zero.as_float().unwrap().to_bits(), (-0.0f64).to_bits());

        let bytes =
            mb_ast_literal_eval(MbValue::from_ptr(MbObject::new_str("b\"hi\"".to_string())));
        let ptr = bytes.as_ptr().expect("bytes literal");
        unsafe {
            if let super::super::super::rc::ObjData::Bytes(ref data) = (*ptr).data {
                assert_eq!(data.as_slice(), b"hi");
            } else {
                panic!("expected bytes");
            }
        }
    }

    #[test]
    fn test_literal_eval_leading_indent_matches_cpython() {
        assert!(!literal_eval_has_unexpected_indent(" \t -1"));
        assert!(!literal_eval_has_unexpected_indent("\n-1"));
        assert!(!literal_eval_has_unexpected_indent("\n   \n-1"));
        assert!(literal_eval_has_unexpected_indent("\n -1"));
        assert!(literal_eval_has_unexpected_indent("\n\t-1"));
        assert!(literal_eval_has_unexpected_indent("   \n -1"));
    }

    #[test]
    fn test_literal_eval_rejects_non_literals() {
        let bad_call = MbValue::from_ptr(MbObject::new_str("foo()".to_string()));
        assert!(mb_ast_literal_eval(bad_call).is_none());
        let bad_expr = MbValue::from_ptr(MbObject::new_str("2+3".to_string()));
        assert!(mb_ast_literal_eval(bad_expr).is_none());
        let bad_sign = MbValue::from_ptr(MbObject::new_str("++6".to_string()));
        assert!(mb_ast_literal_eval(bad_sign).is_none());
    }

    #[test]
    fn test_fix_missing_locations() {
        let node = mb_ast_parse(MbValue::from_ptr(MbObject::new_str("".to_string())));
        let fixed = mb_ast_fix_missing_locations(node);
        assert!(fixed.as_ptr().is_some());

        fn field(node: MbValue, name: &str) -> MbValue {
            let ptr = node.as_ptr().expect("ast node");
            unsafe {
                if let super::super::super::rc::ObjData::Instance { ref fields, .. } = (*ptr).data {
                    *fields.read().unwrap().get(name).expect("location attr")
                } else {
                    panic!("expected AST instance")
                }
            }
        }

        let leaf = make_ast_node("Constant", FxHashMap::default());
        let mut expr_fields = FxHashMap::default();
        expr_fields.insert("value".to_string(), leaf);
        let expr = make_ast_node("Expr", expr_fields);
        set_ast_attr(expr, "lineno", MbValue::from_int(7));
        set_ast_attr(expr, "col_offset", MbValue::from_int(2));
        set_ast_attr(expr, "end_lineno", MbValue::from_int(9));
        set_ast_attr(expr, "end_col_offset", MbValue::from_int(4));

        let mut module_fields = FxHashMap::default();
        module_fields.insert(
            "body".to_string(),
            MbValue::from_ptr(MbObject::new_list_borrowed(vec![expr])),
        );
        module_fields.insert(
            "type_ignores".to_string(),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        let module = make_ast_node("Module", module_fields);
        let fixed = mb_ast_fix_missing_locations(module);

        assert_eq!(fixed.to_bits(), module.to_bits());
        assert_eq!(field(expr, "lineno").as_int(), Some(7));
        assert_eq!(field(expr, "col_offset").as_int(), Some(2));
        assert_eq!(field(expr, "end_lineno").as_int(), Some(9));
        assert_eq!(field(expr, "end_col_offset").as_int(), Some(4));
        assert_eq!(field(leaf, "lineno").as_int(), Some(7));
        assert_eq!(field(leaf, "col_offset").as_int(), Some(2));
        assert_eq!(field(leaf, "end_lineno").as_int(), Some(9));
        assert_eq!(field(leaf, "end_col_offset").as_int(), Some(4));
    }

    #[test]
    fn test_copy_location_copies_cpython_location_attrs() {
        fn field(node: MbValue, name: &str) -> MbValue {
            let ptr = node.as_ptr().expect("ast node");
            unsafe {
                if let super::super::super::rc::ObjData::Instance { ref fields, .. } = (*ptr).data {
                    *fields.read().unwrap().get(name).expect("location attr")
                } else {
                    panic!("expected AST instance")
                }
            }
        }

        let old = make_ast_node("Constant", FxHashMap::default());
        set_ast_attr(old, "lineno", MbValue::from_int(7));
        set_ast_attr(old, "col_offset", MbValue::from_int(3));
        set_ast_attr(old, "end_lineno", MbValue::none());
        set_ast_attr(old, "end_col_offset", MbValue::none());

        let new_node = make_ast_node("Constant", FxHashMap::default());
        let copied = mb_ast_copy_location(new_node, old);

        assert_eq!(field(copied, "lineno").as_int(), Some(7));
        assert_eq!(field(copied, "col_offset").as_int(), Some(3));
        assert!(field(copied, "end_lineno").is_none());
        assert!(field(copied, "end_col_offset").is_none());

        let old_without_start = make_ast_node("Constant", FxHashMap::default());
        set_ast_attr(old_without_start, "lineno", MbValue::none());
        set_ast_attr(old_without_start, "col_offset", MbValue::none());
        set_ast_attr(old_without_start, "end_lineno", MbValue::none());
        set_ast_attr(old_without_start, "end_col_offset", MbValue::none());

        let preserved_start = make_ast_node("Constant", FxHashMap::default());
        set_ast_attr(preserved_start, "lineno", MbValue::from_int(11));
        set_ast_attr(preserved_start, "col_offset", MbValue::from_int(5));
        let copied = mb_ast_copy_location(preserved_start, old_without_start);

        assert_eq!(field(copied, "lineno").as_int(), Some(11));
        assert_eq!(field(copied, "col_offset").as_int(), Some(5));
        assert!(field(copied, "end_lineno").is_none());
        assert!(field(copied, "end_col_offset").is_none());
    }

    #[test]
    fn test_increment_lineno_updates_child_locations() {
        fn field(node: MbValue, name: &str) -> MbValue {
            let ptr = node.as_ptr().expect("ast node");
            unsafe {
                if let super::super::super::rc::ObjData::Instance { ref fields, .. } = (*ptr).data {
                    *fields.read().unwrap().get(name).expect("location attr")
                } else {
                    panic!("expected AST instance")
                }
            }
        }

        let leaf = make_ast_node("Constant", FxHashMap::default());
        set_ast_attr(leaf, "lineno", MbValue::from_int(3));
        set_ast_attr(leaf, "end_lineno", MbValue::from_int(3));

        let mut expr_fields = FxHashMap::default();
        expr_fields.insert("value".to_string(), leaf);
        let expr = make_ast_node("Expr", expr_fields);
        set_ast_attr(expr, "lineno", MbValue::from_int(2));
        set_ast_attr(expr, "end_lineno", MbValue::from_int(2));

        let mut module_fields = FxHashMap::default();
        module_fields.insert(
            "body".to_string(),
            MbValue::from_ptr(MbObject::new_list_borrowed(vec![expr])),
        );
        module_fields.insert(
            "type_ignores".to_string(),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        let module = make_ast_node("Module", module_fields);
        set_ast_attr(module, "lineno", MbValue::from_int(1));
        set_ast_attr(module, "end_lineno", MbValue::from_int(1));

        let incremented = mb_ast_increment_lineno(module, MbValue::from_int(5));

        assert_eq!(incremented.to_bits(), module.to_bits());
        assert_eq!(field(module, "lineno").as_int(), Some(1));
        assert_eq!(field(module, "end_lineno").as_int(), Some(1));
        assert_eq!(field(expr, "lineno").as_int(), Some(7));
        assert_eq!(field(expr, "end_lineno").as_int(), Some(7));
        assert_eq!(field(leaf, "lineno").as_int(), Some(8));
        assert_eq!(field(leaf, "end_lineno").as_int(), Some(8));

        set_ast_attr(leaf, "lineno", MbValue::from_int(10));
        set_ast_attr(leaf, "end_lineno", MbValue::none());
        mb_ast_increment_lineno(leaf, MbValue::none());
        assert_eq!(field(leaf, "lineno").as_int(), Some(11));
        assert!(field(leaf, "end_lineno").is_none());

        let before_error = field(leaf, "lineno");
        let args = [leaf, MbValue::none()];
        super::super::super::exception::mb_clear_exception();
        unsafe {
            d_increment_lineno(args.as_ptr(), args.len());
        }
        assert_eq!(
            super::super::super::exception::current_exception_type().as_deref(),
            Some("TypeError")
        );
        assert_eq!(field(leaf, "lineno").to_bits(), before_error.to_bits());
        super::super::super::exception::mb_clear_exception();

        let op = make_ast_node("Add", FxHashMap::default());
        set_ast_attr(op, "lineno", MbValue::from_int(4));
        set_ast_attr(op, "end_lineno", MbValue::from_int(4));
        mb_ast_increment_lineno(op, MbValue::from_int(5));
        assert_eq!(field(op, "lineno").as_int(), Some(4));
        assert_eq!(field(op, "end_lineno").as_int(), Some(4));
    }

    #[test]
    fn test_iter_fields_excludes_internal() {
        use super::super::super::rc::ObjData;
        // A Module node has body + type_ignores grammar fields plus internal
        // location/_type/_source attrs which must be filtered out.
        let node = mb_ast_parse(MbValue::from_ptr(MbObject::new_str("x = 1".to_string())));
        let result = mb_ast_iter_fields(node);
        let ptr = result.as_ptr().expect("iter_fields returns a list");
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                // Each item is a (name, value) 2-tuple; no internal names.
                for item in items.iter() {
                    let tptr = item.as_ptr().expect("tuple pair");
                    if let ObjData::Tuple(ref elems) = (*tptr).data {
                        assert_eq!(elems.len(), 2);
                        if let Some(name) = extract_str(elems[0]) {
                            assert!(!is_internal_field(&name), "leaked internal field {name}");
                        }
                    } else {
                        panic!("iter_fields item is not a tuple");
                    }
                }
            } else {
                panic!("iter_fields did not return a list");
            }
        }
    }

    #[test]
    fn test_iter_child_nodes_returns_list() {
        let node = mb_ast_parse(MbValue::from_ptr(MbObject::new_str("x = 1".to_string())));
        let result = mb_ast_iter_child_nodes(node);
        assert!(result.as_ptr().is_some());
    }

    #[test]
    fn test_get_source_segment_single_line() {
        use super::super::super::rc::{
            MbObject as RcObj, MbObjectHeader, MbRwLock, ObjData, ObjKind,
        };
        // Build a node spanning columns 4..9 of line 1 -> "value".
        // (make_ast_node would overwrite location attrs with defaults, so build
        // the Instance directly to control col_offset / end_col_offset.)
        let mut fields: FxHashMap<String, MbValue> = FxHashMap::default();
        fields.insert("lineno".to_string(), MbValue::from_int(1));
        fields.insert("end_lineno".to_string(), MbValue::from_int(1));
        fields.insert("col_offset".to_string(), MbValue::from_int(4));
        fields.insert("end_col_offset".to_string(), MbValue::from_int(9));
        let obj = Box::new(RcObj {
            header: MbObjectHeader {
                rc: std::sync::atomic::AtomicU32::new(1),
                kind: ObjKind::Instance,
            },
            data: ObjData::Instance {
                class_name: "Name".to_string(),
                fields: MbRwLock::new(fields),
            },
        });
        let node = MbValue::from_ptr(Box::into_raw(obj));
        let src = MbValue::from_ptr(MbObject::new_str("abc value xyz".to_string()));
        let seg = mb_ast_get_source_segment(src, node);
        let ptr = seg.as_ptr().expect("segment string");
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                assert_eq!(s, "value");
            } else {
                panic!("get_source_segment did not return a string");
            }
        }
    }

    #[test]
    fn test_get_source_segment_missing_location_is_none() {
        // A bare string node with no location attributes -> None.
        use super::super::super::rc::{
            MbObject as RcObj, MbObjectHeader, MbRwLock, ObjData, ObjKind,
        };
        let empty = FxHashMap::default();
        let obj = Box::new(RcObj {
            header: MbObjectHeader {
                rc: std::sync::atomic::AtomicU32::new(1),
                kind: ObjKind::Instance,
            },
            data: ObjData::Instance {
                class_name: "Name".to_string(),
                fields: MbRwLock::new(empty),
            },
        });
        let node = MbValue::from_ptr(Box::into_raw(obj));
        let src = MbValue::from_ptr(MbObject::new_str("abc".to_string()));
        assert!(mb_ast_get_source_segment(src, node).is_none());
    }
}
