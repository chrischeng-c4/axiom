use super::*;
use rustc_hash::FxHashMap;

// ── eval/exec/compile/globals/locals (#441) ──

/// eval(expression) — evaluate a string expression by parsing and
/// walking the AST for pure-value sub-expressions (#1256 sub-priority 6).
///
/// Supported: int/float/complex/string/bytes/bool/None/ellipsis
/// literals; BinOp (arith, comparison, bitwise, logical, identity
/// for None); UnaryOp (Pos/Neg/Not/BitNot); list/tuple/set/dict
/// literals; ternary `a if c else b`; chained comparison
/// (single-pass); membership for sequence-literal RHS.
///
/// Unsupported (returns None): identifier resolution (no scope
/// hook), function calls (no resolver), attribute access, index,
/// slice, comprehensions, lambda, f-strings, generators, yield,
/// await, walrus, unpacks. This is wider than the prior literal-
/// only fallback while staying inside the runtime's pure-value
/// surface; the full scope-bound eval that CPython exposes still
/// needs the parser+interpreter integration tracked under #1256.
pub fn mb_eval(expr: MbValue) -> MbValue {
    mb_eval_impl(expr, None, None)
}

pub fn mb_eval_with_globals(expr: MbValue, globals: MbValue) -> MbValue {
    mb_eval_impl(expr, Some(globals), None)
}

pub fn mb_eval_with_namespaces(expr: MbValue, globals: MbValue, locals: MbValue) -> MbValue {
    mb_eval_impl(expr, Some(globals), Some(locals))
}

fn mb_eval_impl(expr: MbValue, globals: Option<MbValue>, locals: Option<MbValue>) -> MbValue {
    use crate::lexer;
    use crate::parser::Parser;
    use crate::source::SourceMap;

    if let Some(ptr) = expr.as_ptr() {
        unsafe {
            if let ObjData::CodeObject { mode, ast, .. } = &(*ptr).data {
                return mb_eval_code_object(mode, ast, globals, locals);
            }
        }
    }

    let source = if let Some(ptr) = expr.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => s.clone(),
                _ => return MbValue::none(),
            }
        }
    } else {
        return MbValue::none();
    };
    // CPython quirk: eval() (unlike exec()/compile()) strips leading spaces
    // and tabs from the source before parsing, so an indented one-liner
    // like `eval(" 1 + 1")` doesn't trip an IndentationError. Leading
    // newlines/blank lines are NOT stripped.
    let source = source.trim_start_matches([' ', '\t']).to_string();

    let mut source_map = SourceMap::new();
    let file_id = source_map.add_file("<eval>".to_string(), source.clone());
    let tokens = lexer::lex(&source, file_id);
    let mut parser = Parser::new(tokens, &source, file_id);
    parser.skip_newlines();
    let ast = match parser.parse_expr() {
        Ok(e) => e,
        Err(_err) => {
            // CPython: eval of unparseable source raises SyntaxError.
            crate::runtime::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("SyntaxError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "invalid syntax (<string>, line 1)".to_string(),
                )),
            );
            return MbValue::none();
        }
    };
    if globals.is_some() || locals.is_some() {
        let mut ctx = ExecContext {
            globals,
            locals,
            ..ExecContext::default()
        };
        exec_eval_expr(&mut ctx, &ast.node)
    } else {
        eval_expr(&ast.node)
    }
}

fn mb_eval_code_object(
    mode: &str,
    ast: &crate::parser::ast::Module,
    globals: Option<MbValue>,
    locals: Option<MbValue>,
) -> MbValue {
    let mut ctx = ExecContext {
        globals,
        locals,
        ..ExecContext::default()
    };
    if mode == "eval" {
        if let Some(stmt) = ast.stmts.first() {
            if let crate::parser::ast::Stmt::ExprStmt(expr) = &stmt.node {
                return exec_eval_expr(&mut ctx, &expr.node);
            }
        }
        crate::runtime::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "eval() arg 1 must be a string, bytes or code object".to_string(),
            )),
        );
        return MbValue::none();
    }
    exec_stmts_with_context(&mut ctx, &ast.stmts);
    MbValue::none()
}

/// Pending-exception probe for the eval tree walker: sub-evaluations raise
/// via mb_raise (NameError, ZeroDivisionError, format ValueError, ...) and
/// the walker must stop folding once one is pending.
pub(super) fn eval_pending() -> bool {
    crate::runtime::exception::mb_has_exception().as_bool() == Some(true)
}

fn eval_dotted_path(expr: &crate::parser::ast::Expr) -> Option<Vec<String>> {
    use crate::parser::ast::Expr;
    match expr {
        Expr::Ident(name) => Some(vec![name.clone()]),
        Expr::Attr { object, attr } => {
            let mut parts = eval_dotted_path(&object.node)?;
            parts.push(attr.clone());
            Some(parts)
        }
        _ => None,
    }
}

pub(super) fn eval_str_value(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn eval_call_values(args: &[crate::parser::ast::CallArg]) -> Option<(Vec<MbValue>, bool)> {
    use crate::parser::ast::CallArg;
    let mut vals = Vec::new();
    let kwargs = crate::runtime::dict_ops::mb_dict_new();
    let mut has_kwargs = false;
    for arg in args {
        match arg {
            CallArg::Positional(e) => {
                vals.push(eval_expr(&e.node));
                if eval_pending() {
                    return None;
                }
            }
            CallArg::Keyword { name, value } => {
                let v = eval_expr(&value.node);
                if eval_pending() {
                    return None;
                }
                crate::runtime::dict_ops::mb_dict_setitem(
                    kwargs,
                    MbValue::from_ptr(MbObject::new_str(name.clone())),
                    v,
                );
                has_kwargs = true;
            }
            CallArg::StarArg(_) | CallArg::DoubleStarArg(_) => return None,
        }
    }
    if has_kwargs {
        vals.push(kwargs);
    }
    Some((vals, has_kwargs))
}

fn eval_make_args_list(vals: &[MbValue]) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(vals.to_vec()))
}

fn eval_expr(expr: &crate::parser::ast::Expr) -> MbValue {
    use crate::parser::ast::Expr;
    match expr {
        Expr::IntLit(i) => MbValue::from_int(*i),
        Expr::BigIntLit(s) => crate::runtime::bigint_ops::bigint_from_literal(s),
        Expr::FloatLit(f) => MbValue::from_float(*f),
        Expr::BoolLit(b) => MbValue::from_bool(*b),
        Expr::NoneLit => MbValue::none(),
        Expr::StrLit(s) => MbValue::from_ptr(MbObject::new_str(s.clone())),
        Expr::BytesLit(b) => MbValue::from_ptr(MbObject::new_bytes(b.clone())),
        Expr::ComplexLit(imag) => MbValue::from_ptr(MbObject::new_complex(0.0, *imag)),
        Expr::Ellipsis => MbValue::ellipsis(),
        Expr::Ident(name) => {
            if crate::runtime::exception::is_builtin_exception_name(name) {
                return make_type_object(name);
            }
            // Resolve module globals by name (the globals() introspection
            // path); unknown names raise NameError like CPython eval.
            let globals = crate::runtime::closure::build_globals_dict();
            let key = MbValue::from_ptr(MbObject::new_str(name.clone()));
            let contains = crate::runtime::dict_ops::mb_dict_contains(globals, key)
                .as_bool()
                .unwrap_or(false);
            if contains {
                return crate::runtime::dict_ops::mb_dict_get(globals, key, MbValue::none());
            }
            crate::runtime::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("NameError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!("name '{name}' is not defined"))),
            );
            MbValue::none()
        }
        Expr::FString(parts) => {
            fn fold_parts(parts: &[crate::parser::ast::FStringPart]) -> Option<String> {
                let mut out = String::new();
                for p in parts {
                    match p {
                        crate::parser::ast::FStringPart::Literal(s) => out.push_str(s),
                        crate::parser::ast::FStringPart::Expr(e, spec) => {
                            let v = eval_expr(&e.node);
                            if eval_pending() {
                                return None;
                            }
                            let formatted = match spec {
                                None => crate::runtime::string_ops::mb_fstring_value(v),
                                Some(spec_parts) => {
                                    let mut spec_str = String::new();
                                    for sp in spec_parts {
                                        match sp {
                                            crate::parser::ast::FStringPart::Literal(l) => {
                                                spec_str.push_str(l)
                                            }
                                            crate::parser::ast::FStringPart::Expr(se, _) => {
                                                let sv = eval_expr(&se.node);
                                                if eval_pending() {
                                                    return None;
                                                }
                                                let txt = mb_str(sv);
                                                if let Some(ptr) = txt.as_ptr() {
                                                    unsafe {
                                                        if let ObjData::Str(ref t) = (*ptr).data {
                                                            spec_str.push_str(t);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    crate::runtime::string_ops::mb_format_value(
                                        v,
                                        MbValue::from_ptr(MbObject::new_str(spec_str)),
                                    )
                                }
                            };
                            if eval_pending() {
                                return None;
                            }
                            if let Some(ptr) = formatted.as_ptr() {
                                unsafe {
                                    if let ObjData::Str(ref t) = (*ptr).data {
                                        out.push_str(t);
                                    }
                                }
                            }
                        }
                    }
                }
                Some(out)
            }
            match fold_parts(parts) {
                Some(out) => MbValue::from_ptr(MbObject::new_str(out)),
                None => MbValue::none(),
            }
        }
        Expr::BinOp { op, lhs, rhs } => {
            let l = eval_expr(&lhs.node);
            if eval_pending() {
                return MbValue::none();
            }
            let r = eval_expr(&rhs.node);
            if eval_pending() {
                return MbValue::none();
            }
            eval_binop(*op, l, r)
        }
        Expr::UnaryOp { op, operand } => {
            let v = eval_expr(&operand.node);
            eval_unaryop(*op, v)
        }
        Expr::ListLit(items) => {
            let vals: Vec<MbValue> = items.iter().map(|e| eval_expr(&e.node)).collect();
            MbValue::from_ptr(MbObject::new_list(vals))
        }
        Expr::TupleLit(items) => {
            let vals: Vec<MbValue> = items.iter().map(|e| eval_expr(&e.node)).collect();
            MbValue::from_ptr(MbObject::new_tuple(vals))
        }
        Expr::SetLit(items) => {
            let vals: Vec<MbValue> = items.iter().map(|e| eval_expr(&e.node)).collect();
            crate::runtime::set_ops::mb_set_from_list(MbValue::from_ptr(MbObject::new_list(vals)))
        }
        Expr::DictLit(entries) => {
            let d = crate::runtime::dict_ops::mb_dict_new();
            for (k, v) in entries {
                if let Some(k_expr) = k {
                    let kv = eval_expr(&k_expr.node);
                    let vv = eval_expr(&v.node);
                    crate::runtime::dict_ops::mb_dict_setitem(d, kv, vv);
                }
            }
            d
        }
        Expr::IfExpr {
            body,
            condition,
            else_body,
        } => {
            let c = eval_expr(&condition.node);
            if c.as_bool().unwrap_or(false) || c.as_int().unwrap_or(0) != 0 {
                eval_expr(&body.node)
            } else {
                eval_expr(&else_body.node)
            }
        }
        Expr::ChainedCompare { operands, ops } => {
            if operands.is_empty() {
                return MbValue::from_bool(true);
            }
            let mut prev = eval_expr(&operands[0].node);
            for (i, op) in ops.iter().enumerate() {
                let next = eval_expr(&operands[i + 1].node);
                let r = eval_binop(*op, prev, next);
                if !r.as_bool().unwrap_or(false) {
                    return MbValue::from_bool(false);
                }
                prev = next;
            }
            MbValue::from_bool(true)
        }
        Expr::Call { func, args } => {
            // Narrow constructor support so `eval(repr(x))` round-trips for
            // the stdlib numeric handle types (Decimal('0.3'),
            // Fraction(3, 4)). Only positional literal-ish args evaluate.
            if let Some(path) = eval_dotted_path(&func.node) {
                let (vals, has_kwargs) = match eval_call_values(args) {
                    Some(v) => v,
                    None => return MbValue::none(),
                };
                match path.as_slice() {
                    [name] if name == "Decimal" && !has_kwargs && vals.len() == 1 => {
                        return crate::runtime::stdlib::decimal_mod::mb_decimal_new(vals[0]);
                    }
                    [name]
                        if name == "Fraction" && !has_kwargs && (1..=2).contains(&vals.len()) =>
                    {
                        return crate::runtime::stdlib::fractions_mod::mb_fraction_new(
                            vals[0],
                            vals.get(1).copied().unwrap_or_else(MbValue::none),
                        );
                    }
                    [name] if name == "repr" && !has_kwargs && vals.len() == 1 => {
                        return mb_repr(vals[0])
                    }
                    [name] if name == "str" && !has_kwargs && vals.len() == 1 => {
                        return mb_str(vals[0])
                    }
                    [name]
                        if crate::runtime::exception::is_builtin_exception_name(name)
                            && !has_kwargs =>
                    {
                        let typ = make_type_object(name);
                        let args_list = eval_make_args_list(&vals);
                        return mb_call_spread(typ, args_list);
                    }
                    [module, name]
                        if module == "contextlib" && name == "suppress" && !has_kwargs =>
                    {
                        return crate::runtime::stdlib::contextlib_mod::mb_contextlib_suppress_instance(
                            vals,
                        );
                    }
                    [module, ctor] if module == "datetime" && ctor == "timedelta" => {
                        return crate::runtime::stdlib::datetime_mod::mb_timedelta_new(
                            MbValue::from_ptr(MbObject::new_list(vals)),
                        );
                    }
                    [module, ctor] if module == "datetime" && ctor == "timezone" && !has_kwargs => {
                        let offset = vals.first().copied().unwrap_or_else(MbValue::none);
                        let name = vals.get(1).copied().and_then(eval_str_value);
                        return crate::runtime::stdlib::datetime_mod::timezone_from_offset(
                            offset, name,
                        );
                    }
                    _ => {}
                }
            } else if let Expr::Ident(name) = &func.node {
                let vals: Vec<MbValue> = args
                    .iter()
                    .filter_map(|a| match a {
                        crate::parser::ast::CallArg::Positional(e) => Some(eval_expr(&e.node)),
                        _ => None,
                    })
                    .collect();
                match name.as_str() {
                    "Decimal" if vals.len() == 1 => {
                        return crate::runtime::stdlib::decimal_mod::mb_decimal_new(vals[0]);
                    }
                    "Fraction" if (1..=2).contains(&vals.len()) => {
                        return crate::runtime::stdlib::fractions_mod::mb_fraction_new(
                            vals[0],
                            vals.get(1).copied().unwrap_or_else(MbValue::none),
                        );
                    }
                    // f-string conversion wrappers (!r lowers to repr(...)).
                    "repr" if vals.len() == 1 => return mb_repr(vals[0]),
                    "str" if vals.len() == 1 => return mb_str(vals[0]),
                    _ => {}
                }
            }
            // Calling a non-callable literal ((1)() in an eval'd f-string)
            // raises TypeError like CPython.
            let callee_type = match &func.node {
                Expr::IntLit(_) | Expr::BigIntLit(_) => Some("int"),
                Expr::FloatLit(_) => Some("float"),
                Expr::StrLit(_) => Some("str"),
                Expr::BoolLit(_) => Some("bool"),
                Expr::NoneLit => Some("NoneType"),
                Expr::ListLit(_) => Some("list"),
                Expr::DictLit(_) => Some("dict"),
                Expr::TupleLit(_) => Some("tuple"),
                _ => None,
            };
            if let Some(tn) = callee_type {
                crate::runtime::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!("'{tn}' object is not callable"))),
                );
            }
            MbValue::none()
        }
        Expr::Attr { .. } => {
            if let Some(path) = eval_dotted_path(expr) {
                match path.as_slice() {
                    [module, class, attr]
                        if module == "datetime"
                            && class == "timezone"
                            && matches!(attr.as_str(), "utc" | "min" | "max") =>
                    {
                        return crate::runtime::stdlib::datetime_mod::timezone_class_attr(attr)
                            .unwrap_or_else(MbValue::none);
                    }
                    _ => {}
                }
            }
            MbValue::none()
        }
        _ => MbValue::none(),
    }
}

fn exec_has_pending_exception() -> bool {
    crate::runtime::exception::mb_has_exception().as_bool() == Some(true)
}

#[derive(Default)]
struct ExecContext {
    class_match_args: FxHashMap<String, Option<MbValue>>,
    type_vars: std::collections::HashSet<String>,
    type_param_reuse_once: FxHashMap<String, MbValue>,
    functions: FxHashMap<String, ExecFunction>,
    frames: Vec<FxHashMap<String, MbValue>>,
    generic_class_body_depth: usize,
    globals: Option<MbValue>,
    locals: Option<MbValue>,
}

#[derive(Clone)]
struct ExecFunction {
    params: Vec<String>,
    defaults: Vec<Option<MbValue>>,
    body: Vec<crate::source::span::Spanned<crate::parser::ast::Stmt>>,
}

#[derive(Clone)]
struct ExecFunctionBinding {
    name: String,
    is_async: bool,
    globals: Option<MbValue>,
    captures: Vec<FxHashMap<String, MbValue>>,
    function: ExecFunction,
}

static EXEC_FUNCTIONS: std::sync::LazyLock<std::sync::RwLock<FxHashMap<u64, ExecFunctionBinding>>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new(FxHashMap::default()));
static NEXT_EXEC_FUNCTION_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

#[derive(Clone, Copy)]
enum ExecFlow {
    Normal,
    Return(MbValue),
    Break,
    Continue,
}

fn exec_raise_type_error(message: String) {
    crate::runtime::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(message)),
    );
}

fn exec_match_type_name(value: MbValue) -> &'static str {
    if value.is_none() {
        return "NoneType";
    }
    if let Some(ptr) = value.as_ptr() {
        unsafe {
            return match &(*ptr).data {
                ObjData::Tuple(_) => "tuple",
                ObjData::List(_) => "list",
                ObjData::Str(_) => "str",
                ObjData::Bytes(_) => "bytes",
                ObjData::ByteArray(_) => "bytearray",
                ObjData::Dict(_) => "dict",
                ObjData::Set(_) => "set",
                _ => "object",
            };
        }
    }
    if value.as_bool().is_some() {
        "bool"
    } else if value.as_int().is_some() {
        "int"
    } else if value.as_float().is_some() {
        "float"
    } else {
        "object"
    }
}

fn exec_raise_class_pattern_count_error(class_name: &str, accepted: usize, given: usize) {
    let sub = if accepted == 1 {
        "sub-pattern"
    } else {
        "sub-patterns"
    };
    exec_raise_type_error(format!(
        "{class_name}() accepts {accepted} positional {sub} ({given} given)"
    ));
}

fn exec_is_typevar_constructor(expr: &crate::parser::ast::Expr) -> bool {
    let crate::parser::ast::Expr::Call { func, .. } = expr else {
        return false;
    };
    matches!(
        eval_dotted_path(&func.node).as_deref(),
        Some([name]) if name == crate::lower::pep695::TYPEVAR_INTRINSIC
    ) || matches!(
        eval_dotted_path(&func.node).as_deref(),
        Some([name]) if name == "TypeVar"
    ) || matches!(
        eval_dotted_path(&func.node).as_deref(),
        Some([module, name]) if module == "typing" && name == "TypeVar"
    )
}

fn exec_is_typing_generic_expr(expr: &crate::parser::ast::Expr) -> bool {
    matches!(eval_dotted_path(expr).as_deref(), Some([name]) if name == "Generic")
        || matches!(
            eval_dotted_path(expr).as_deref(),
            Some([module, name]) if module == "typing" && name == "Generic"
        )
}

fn exec_base_is_typing_generic(base: &crate::parser::ast::Expr) -> bool {
    use crate::parser::ast::Expr;
    match base {
        Expr::Index { object, .. } => exec_is_typing_generic_expr(&object.node),
        _ => exec_is_typing_generic_expr(base),
    }
}

fn exec_base_is_object(base: &crate::parser::ast::Expr) -> bool {
    matches!(eval_dotted_path(base).as_deref(), Some([name]) if name == "object")
}

fn exec_expr_contains_comprehension_expr(expr: &crate::parser::ast::Expr) -> bool {
    use crate::parser::ast::{CallArg, Expr};
    match expr {
        Expr::GeneratorExpr { .. }
        | Expr::ListComp { .. }
        | Expr::SetComp { .. }
        | Expr::DictComp { .. } => true,
        Expr::Call { func, args } => {
            exec_expr_contains_comprehension_expr(&func.node)
                || args.iter().any(|arg| match arg {
                    CallArg::Positional(expr)
                    | CallArg::Keyword { value: expr, .. }
                    | CallArg::StarArg(expr)
                    | CallArg::DoubleStarArg(expr) => {
                        exec_expr_contains_comprehension_expr(&expr.node)
                    }
                })
        }
        Expr::Index { object, index } => {
            exec_expr_contains_comprehension_expr(&object.node)
                || exec_expr_contains_comprehension_expr(&index.node)
        }
        Expr::Attr { object, .. } => exec_expr_contains_comprehension_expr(&object.node),
        Expr::TupleLit(items) | Expr::ListLit(items) | Expr::SetLit(items) => items
            .iter()
            .any(|item| exec_expr_contains_comprehension_expr(&item.node)),
        Expr::DictLit(entries) => entries.iter().any(|(key, value)| {
            key.as_ref()
                .is_some_and(|key| exec_expr_contains_comprehension_expr(&key.node))
                || exec_expr_contains_comprehension_expr(&value.node)
        }),
        Expr::BinOp { lhs, rhs, .. } => {
            exec_expr_contains_comprehension_expr(&lhs.node)
                || exec_expr_contains_comprehension_expr(&rhs.node)
        }
        Expr::UnaryOp { operand: expr, .. }
        | Expr::Await(expr)
        | Expr::Yield(Some(expr))
        | Expr::YieldFrom(expr)
        | Expr::Starred(expr) => exec_expr_contains_comprehension_expr(&expr.node),
        Expr::IfExpr {
            body,
            condition,
            else_body,
        } => {
            exec_expr_contains_comprehension_expr(&body.node)
                || exec_expr_contains_comprehension_expr(&condition.node)
                || exec_expr_contains_comprehension_expr(&else_body.node)
        }
        Expr::Lambda { body, .. } => exec_expr_contains_comprehension_expr(&body.node),
        Expr::Walrus { value, .. } => exec_expr_contains_comprehension_expr(&value.node),
        Expr::Slice { start, stop, step } => start
            .iter()
            .chain(stop.iter())
            .chain(step.iter())
            .any(|expr| exec_expr_contains_comprehension_expr(&expr.node)),
        Expr::ChainedCompare { operands, .. } => operands
            .iter()
            .any(|operand| exec_expr_contains_comprehension_expr(&operand.node)),
        _ => false,
    }
}

fn exec_raise_pep695_class_annotation_comprehension_syntax_error() {
    crate::runtime::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("SyntaxError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "Cannot use comprehension in annotation scope within class scope".to_string(),
        )),
    );
}

fn exec_collect_index_idents(expr: &crate::parser::ast::Expr, out: &mut Vec<String>) {
    use crate::parser::ast::Expr;
    match expr {
        Expr::Ident(name) => out.push(name.clone()),
        Expr::Index { object, index } => {
            exec_collect_index_idents(&object.node, out);
            exec_collect_index_idents(&index.node, out);
        }
        Expr::TupleLit(items) | Expr::ListLit(items) | Expr::SetLit(items) => {
            for item in items {
                exec_collect_index_idents(&item.node, out);
            }
        }
        Expr::BinOp { lhs, rhs, .. } => {
            exec_collect_index_idents(&lhs.node, out);
            exec_collect_index_idents(&rhs.node, out);
        }
        Expr::Attr { .. }
        | Expr::Call { .. }
        | Expr::IntLit(_)
        | Expr::BigIntLit(_)
        | Expr::FloatLit(_)
        | Expr::ComplexLit(_)
        | Expr::StrLit(_)
        | Expr::BytesLit(_)
        | Expr::BoolLit(_)
        | Expr::NoneLit
        | Expr::Ellipsis
        | Expr::UnaryOp { .. }
        | Expr::Slice { .. }
        | Expr::DictLit(_)
        | Expr::IfExpr { .. }
        | Expr::ChainedCompare { .. }
        | Expr::Lambda { .. }
        | Expr::FString(_)
        | Expr::ListComp { .. }
        | Expr::SetComp { .. }
        | Expr::DictComp { .. }
        | Expr::GeneratorExpr { .. }
        | Expr::Await(_)
        | Expr::Yield(_)
        | Expr::YieldFrom(_)
        | Expr::Walrus { .. }
        | Expr::Starred(_)
        | Expr::UnpackTarget(_) => {}
    }
}

fn exec_validate_pep695_class_bases(
    ctx: &ExecContext,
    type_params: &[crate::parser::ast::TypeParam],
    bases: &[crate::source::span::Spanned<crate::parser::ast::Expr>],
) {
    if type_params.is_empty() {
        return;
    }

    if bases
        .iter()
        .any(|base| exec_base_is_typing_generic(&base.node))
    {
        exec_raise_type_error("Cannot inherit from Generic[...] multiple times.".to_string());
        return;
    }

    if bases.iter().any(|base| exec_base_is_object(&base.node)) {
        exec_raise_type_error(
            "Cannot create a consistent method resolution order (MRO) for bases object, Generic"
                .to_string(),
        );
        return;
    }

    if ctx.generic_class_body_depth > 0
        && bases
            .iter()
            .any(|base| exec_expr_contains_comprehension_expr(&base.node))
    {
        exec_raise_pep695_class_annotation_comprehension_syntax_error();
        return;
    }

    let declared: std::collections::HashSet<&str> = type_params
        .iter()
        .map(|param| param.name.as_str())
        .collect();
    for base in bases {
        let crate::parser::ast::Expr::Index { index, .. } = &base.node else {
            continue;
        };
        let mut names = Vec::new();
        exec_collect_index_idents(&index.node, &mut names);
        if let Some(name) = names
            .iter()
            .find(|name| ctx.type_vars.contains(*name) && !declared.contains(name.as_str()))
        {
            exec_raise_type_error(format!(
                "Some type variables (~{name}) are not listed in Generic"
            ));
            return;
        }
    }
}

fn exec_subject_class_name(expr: &crate::parser::ast::Expr) -> Option<String> {
    use crate::parser::ast::{CallArg, Expr};
    let Expr::Call { func, args } = expr else {
        return None;
    };
    if args
        .iter()
        .any(|arg| !matches!(arg, CallArg::Positional(_)))
    {
        return None;
    }
    match &func.node {
        Expr::Ident(name) => Some(name.clone()),
        Expr::Attr { attr, .. } => Some(attr.clone()),
        _ => None,
    }
}

fn exec_validate_class_pattern(
    ctx: &ExecContext,
    subject_class: &str,
    pattern: &crate::parser::ast::Pattern,
) {
    use crate::parser::ast::Pattern;
    let Pattern::ClassPattern { cls, patterns } = pattern else {
        return;
    };
    let Some(pattern_class) = cls.last() else {
        return;
    };
    if pattern_class != subject_class {
        return;
    }

    let positional = patterns.iter().filter(|(name, _)| name.is_none()).count();
    let match_args = ctx.class_match_args.get(subject_class).copied().flatten();
    let items = match match_args {
        Some(value) => {
            if let Some(ptr) = value.as_ptr() {
                unsafe {
                    match &(*ptr).data {
                        ObjData::Tuple(items) => items.clone(),
                        _ => {
                            exec_raise_type_error(format!(
                                "{subject_class}.__match_args__ must be a tuple (got {})",
                                exec_match_type_name(value)
                            ));
                            return;
                        }
                    }
                }
            } else {
                exec_raise_type_error(format!(
                    "{subject_class}.__match_args__ must be a tuple (got {})",
                    exec_match_type_name(value)
                ));
                return;
            }
        }
        None => Vec::new(),
    };

    if positional > items.len() {
        exec_raise_class_pattern_count_error(subject_class, items.len(), positional);
        return;
    }

    let mut seen = std::collections::HashSet::new();
    for item in items.iter().take(positional) {
        let Some(name) = eval_str_value(*item) else {
            exec_raise_type_error(format!(
                "__match_args__ elements must be strings (got {})",
                exec_match_type_name(*item)
            ));
            return;
        };
        if !seen.insert(name.clone()) {
            exec_raise_type_error(format!(
                "{subject_class}() got multiple sub-patterns for attribute '{name}'"
            ));
            return;
        }
    }
    for (keyword, _) in patterns.iter().filter(|(name, _)| name.is_some()) {
        if let Some(name) = keyword {
            if !seen.insert(name.clone()) {
                exec_raise_type_error(format!(
                    "{subject_class}() got multiple sub-patterns for attribute '{name}'"
                ));
                return;
            }
        }
    }
}

fn exec_store_global(ctx: &ExecContext, name: &str, value: MbValue) {
    let Some(globals) = ctx.globals else {
        return;
    };
    crate::runtime::dict_ops::mb_dict_setitem(
        globals,
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
        value,
    );
}

fn exec_lookup_builtin_name(name: &str) -> Option<MbValue> {
    let value = crate::runtime::module::mb_builtin_get(MbValue::from_ptr(MbObject::new_str(
        name.to_string(),
    )));
    if value.is_none() {
        None
    } else {
        Some(value)
    }
}

fn exec_lookup_name(ctx: &ExecContext, name: &str) -> Option<MbValue> {
    for frame in ctx.frames.iter().rev() {
        if let Some(&value) = frame.get(name) {
            return Some(value);
        }
    }
    if let Some(locals) = ctx.locals {
        let key = MbValue::from_ptr(MbObject::new_str(name.to_string()));
        if crate::runtime::dict_ops::mb_dict_contains(locals, key)
            .as_bool()
            .unwrap_or(false)
        {
            return Some(crate::runtime::dict_ops::mb_dict_get(
                locals,
                key,
                MbValue::none(),
            ));
        }
    }
    let globals = ctx.globals?;
    let key = MbValue::from_ptr(MbObject::new_str(name.to_string()));
    if crate::runtime::dict_ops::mb_dict_contains(globals, key)
        .as_bool()
        .unwrap_or(false)
    {
        Some(crate::runtime::dict_ops::mb_dict_get(
            globals,
            key,
            MbValue::none(),
        ))
    } else {
        exec_lookup_builtin_name(name)
    }
}

fn exec_store_name(ctx: &mut ExecContext, name: &str, value: MbValue) {
    if let Some(frame) = ctx.frames.last_mut() {
        frame.insert(name.to_string(), value);
    } else if let Some(locals) = ctx.locals {
        crate::runtime::dict_ops::mb_dict_setitem(
            locals,
            MbValue::from_ptr(MbObject::new_str(name.to_string())),
            value,
        );
    } else {
        exec_store_global(ctx, name, value);
    }
}

enum ExecMaskedNameScope {
    Frame,
    Locals,
    Globals,
}

struct ExecMaskedName {
    name: String,
    scope: ExecMaskedNameScope,
    value: MbValue,
}

fn exec_take_name_binding(ctx: &mut ExecContext, name: &str) -> Option<ExecMaskedName> {
    if let Some(frame) = ctx.frames.last_mut() {
        return frame.remove(name).map(|value| ExecMaskedName {
            name: name.to_string(),
            scope: ExecMaskedNameScope::Frame,
            value,
        });
    }
    let key = MbValue::from_ptr(MbObject::new_str(name.to_string()));
    if let Some(locals) = ctx.locals {
        if crate::runtime::dict_ops::mb_dict_contains(locals, key)
            .as_bool()
            .unwrap_or(false)
        {
            let value = crate::runtime::dict_ops::mb_dict_get(locals, key, MbValue::none());
            unsafe {
                crate::runtime::rc::retain_if_ptr(value);
            }
            crate::runtime::dict_ops::mb_dict_delitem(locals, key);
            return Some(ExecMaskedName {
                name: name.to_string(),
                scope: ExecMaskedNameScope::Locals,
                value,
            });
        }
    }
    let globals = ctx.globals?;
    if crate::runtime::dict_ops::mb_dict_contains(globals, key)
        .as_bool()
        .unwrap_or(false)
    {
        let value = crate::runtime::dict_ops::mb_dict_get(globals, key, MbValue::none());
        unsafe {
            crate::runtime::rc::retain_if_ptr(value);
        }
        crate::runtime::dict_ops::mb_dict_delitem(globals, key);
        return Some(ExecMaskedName {
            name: name.to_string(),
            scope: ExecMaskedNameScope::Globals,
            value,
        });
    }
    None
}

fn exec_restore_name_bindings(ctx: &mut ExecContext, masked: Vec<ExecMaskedName>) {
    for masked_name in masked {
        match masked_name.scope {
            ExecMaskedNameScope::Frame => {
                if let Some(frame) = ctx.frames.last_mut() {
                    frame.insert(masked_name.name, masked_name.value);
                }
            }
            ExecMaskedNameScope::Locals => {
                if let Some(locals) = ctx.locals {
                    crate::runtime::dict_ops::mb_dict_setitem(
                        locals,
                        MbValue::from_ptr(MbObject::new_str(masked_name.name)),
                        masked_name.value,
                    );
                    unsafe {
                        crate::runtime::rc::release_if_ptr(masked_name.value);
                    }
                }
            }
            ExecMaskedNameScope::Globals => {
                if let Some(globals) = ctx.globals {
                    crate::runtime::dict_ops::mb_dict_setitem(
                        globals,
                        MbValue::from_ptr(MbObject::new_str(masked_name.name)),
                        masked_name.value,
                    );
                    unsafe {
                        crate::runtime::rc::release_if_ptr(masked_name.value);
                    }
                }
            }
        }
    }
}

struct ExecTemporaryName {
    name: String,
    previous: Option<ExecMaskedName>,
}

fn exec_type_param_value(param: &crate::parser::ast::TypeParam) -> MbValue {
    let kind = match param.kind {
        crate::parser::ast::TypeParamKind::TypeVar => 0,
        crate::parser::ast::TypeParamKind::TypeVarTuple => 1,
        crate::parser::ast::TypeParamKind::ParamSpec => 2,
    };
    crate::runtime::pep695::mb_pep695_typevar(
        MbValue::from_ptr(MbObject::new_str(param.name.clone())),
        MbValue::from_int(kind),
        MbValue::none(),
        MbValue::none(),
        MbValue::none(),
    )
}

fn exec_bind_temporary_type_params(
    ctx: &mut ExecContext,
    type_params: &[crate::parser::ast::TypeParam],
) -> Vec<ExecTemporaryName> {
    let mut bindings = Vec::with_capacity(type_params.len());
    for param in type_params {
        let previous = exec_take_name_binding(ctx, &param.name);
        exec_store_name(ctx, &param.name, exec_type_param_value(param));
        bindings.push(ExecTemporaryName {
            name: param.name.clone(),
            previous,
        });
    }
    bindings
}

fn exec_type_params_tuple_value(
    ctx: &ExecContext,
    type_params: &[crate::parser::ast::TypeParam],
) -> Option<MbValue> {
    if type_params.is_empty() {
        return None;
    }
    let mut values = Vec::with_capacity(type_params.len());
    for param in type_params {
        values.push(exec_lookup_name(ctx, &param.name)?);
    }
    Some(MbValue::from_ptr(MbObject::new_tuple(values)))
}

fn exec_restore_temporary_names(ctx: &mut ExecContext, bindings: Vec<ExecTemporaryName>) {
    for binding in bindings {
        if let Some(current) = exec_take_name_binding(ctx, &binding.name) {
            unsafe {
                crate::runtime::rc::release_if_ptr(current.value);
            }
        }
        if let Some(previous) = binding.previous {
            exec_restore_name_bindings(ctx, vec![previous]);
        }
    }
}

fn exec_drop_masked_name(masked_name: ExecMaskedName) {
    match masked_name.scope {
        ExecMaskedNameScope::Frame => {}
        ExecMaskedNameScope::Locals | ExecMaskedNameScope::Globals => unsafe {
            crate::runtime::rc::release_if_ptr(masked_name.value);
        },
    }
}

fn exec_drop_name_bindings(masked: Vec<ExecMaskedName>) {
    for masked_name in masked {
        exec_drop_masked_name(masked_name);
    }
}

fn exec_commit_temporary_names(ctx: &mut ExecContext, bindings: Vec<ExecTemporaryName>) {
    for binding in bindings {
        if let Some(value) = exec_lookup_name(ctx, &binding.name) {
            if exec_is_pep695_type_param_value(value) {
                unsafe {
                    crate::runtime::rc::retain_if_ptr(value);
                }
                if let Some(previous) = ctx
                    .type_param_reuse_once
                    .insert(binding.name.clone(), value)
                {
                    unsafe {
                        crate::runtime::rc::release_if_ptr(previous);
                    }
                }
            }
        }
        if let Some(previous) = binding.previous {
            exec_drop_masked_name(previous);
        }
    }
}

fn exec_mask_type_param_bindings(
    ctx: &mut ExecContext,
    type_params: &[crate::parser::ast::TypeParam],
) -> Vec<ExecMaskedName> {
    type_params
        .iter()
        .filter_map(|param| exec_take_name_binding(ctx, &param.name))
        .collect()
}

fn exec_string_value(value: MbValue) -> Option<String> {
    value.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Str(text) => Some(text.clone()),
            _ => None,
        }
    })
}

fn exec_is_pep695_type_param_value(value: MbValue) -> bool {
    value.as_ptr().is_some_and(|ptr| unsafe {
        matches!(
            &(*ptr).data,
            ObjData::Instance { class_name, .. }
                if matches!(class_name.as_str(), "TypeVar" | "TypeVarTuple" | "ParamSpec")
        )
    })
}

fn make_exec_function_value(name: &str, is_async: bool, return_value: MbValue) -> MbValue {
    let inst = MbObject::new_instance("__exec_function__".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut guard = fields.write().unwrap();
            guard.insert(
                "__name__".to_string(),
                MbValue::from_ptr(MbObject::new_str(name.to_string())),
            );
            guard.insert("__is_async__".to_string(), MbValue::from_bool(is_async));
            unsafe {
                crate::runtime::rc::retain_if_ptr(return_value);
            }
            guard.insert("__return__".to_string(), return_value);
        }
    }
    MbValue::from_ptr(inst)
}

/// The docstring of an exec()/compile()-interpreted function or class body:
/// the string-literal value of a leading bare-expression statement (mirrors
/// `hir_to_mir::extract_leading_docstring` for the compiled path). `None`
/// when the body doesn't start with a bare string literal — including when
/// compile(..., optimize=2) has already stripped it from the AST (R2).
fn exec_leading_docstring(
    body: &[crate::source::span::Spanned<crate::parser::ast::Stmt>],
) -> Option<String> {
    use crate::parser::ast::{Expr, Stmt};
    match &body.first()?.node {
        Stmt::ExprStmt(e) => match &e.node {
            Expr::StrLit(s) => Some(s.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn exec_capture_frames(ctx: &ExecContext) -> Vec<FxHashMap<String, MbValue>> {
    let mut captures = Vec::with_capacity(ctx.frames.len());
    for frame in &ctx.frames {
        let mut captured = FxHashMap::default();
        for (name, value) in frame {
            unsafe {
                crate::runtime::rc::retain_if_ptr(*value);
            }
            captured.insert(name.clone(), *value);
        }
        captures.push(captured);
    }
    captures
}

fn make_exec_function_body_value(
    name: &str,
    is_async: bool,
    function: ExecFunction,
    ctx: &ExecContext,
    doc: Option<String>,
) -> MbValue {
    let globals = ctx.globals;
    if let Some(globals) = globals {
        unsafe {
            crate::runtime::rc::retain_if_ptr(globals);
        }
    }
    for default in &function.defaults {
        if let Some(value) = default {
            unsafe {
                crate::runtime::rc::retain_if_ptr(*value);
            }
        }
    }
    let captures = exec_capture_frames(ctx);
    let id = NEXT_EXEC_FUNCTION_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    EXEC_FUNCTIONS.write().unwrap().insert(
        id,
        ExecFunctionBinding {
            name: name.to_string(),
            is_async,
            globals,
            captures,
            function,
        },
    );

    let inst = MbObject::new_instance("__exec_function__".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut guard = fields.write().unwrap();
            guard.insert(
                "__name__".to_string(),
                MbValue::from_ptr(MbObject::new_str(name.to_string())),
            );
            guard.insert("__is_async__".to_string(), MbValue::from_bool(is_async));
            guard.insert("__function_id__".to_string(), MbValue::from_int(id as i64));
            guard.insert(
                "__doc__".to_string(),
                match doc {
                    Some(d) => MbValue::from_ptr(MbObject::new_str(d)),
                    None => MbValue::none(),
                },
            );
        }
    }
    MbValue::from_ptr(inst)
}

fn exec_eval_annotation_type_expr(
    ctx: &mut ExecContext,
    ty: &crate::parser::ast::TypeExpr,
) -> Option<MbValue> {
    use crate::parser::ast::TypeExpr;

    match ty {
        // Match the compiled-path convention: parser-injected fillers mean
        // "no annotation", so exec() should not materialize them.
        TypeExpr::Named(name) if name == "Any" || name == "Self" => None,
        TypeExpr::Named(name) => {
            let value = exec_lookup_name(ctx, name).unwrap_or_else(|| {
                crate::runtime::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("NameError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!("name '{name}' is not defined"))),
                );
                MbValue::none()
            });
            if exec_has_pending_exception() {
                None
            } else {
                Some(value)
            }
        }
        TypeExpr::Generic { name, args } => {
            let origin = exec_eval_annotation_type_expr(ctx, &TypeExpr::Named(name.clone()))?;
            let mut items = Vec::with_capacity(args.len());
            for arg in args {
                let value = exec_eval_annotation_type_expr(ctx, &arg.node)?;
                items.push(value);
            }
            let key = if items.len() == 1 {
                items[0]
            } else {
                MbValue::from_ptr(MbObject::new_tuple(items))
            };
            let value = crate::runtime::class::mb_obj_getitem(origin, key);
            if exec_has_pending_exception() {
                None
            } else {
                Some(value)
            }
        }
        TypeExpr::Optional(inner) => {
            let inner = exec_eval_annotation_type_expr(ctx, &inner.node)?;
            Some(crate::runtime::stdlib::typing_mod::typing_union(vec![
                inner,
                MbValue::none(),
            ]))
        }
        TypeExpr::Union(parts) => {
            let mut members = Vec::with_capacity(parts.len());
            for part in parts {
                let value = exec_eval_annotation_type_expr(ctx, &part.node)?;
                members.push(value);
            }
            Some(crate::runtime::stdlib::typing_mod::typing_union(members))
        }
        TypeExpr::Tuple(parts) => {
            let mut items = Vec::with_capacity(parts.len());
            for part in parts {
                let value = exec_eval_annotation_type_expr(ctx, &part.node)?;
                items.push(value);
            }
            let key = if items.len() == 1 {
                items[0]
            } else {
                MbValue::from_ptr(MbObject::new_tuple(items))
            };
            let tuple_type = make_type_object("tuple");
            let value = crate::runtime::class::mb_obj_getitem(tuple_type, key);
            if exec_has_pending_exception() {
                None
            } else {
                Some(value)
            }
        }
        TypeExpr::Fn { .. } => None,
    }
}

fn exec_function_annotations_value(
    ctx: &mut ExecContext,
    params: &[crate::parser::ast::Param],
    return_ty: Option<&crate::source::span::Spanned<crate::parser::ast::TypeExpr>>,
) -> Option<MbValue> {
    let annotations = crate::runtime::dict_ops::mb_dict_new();

    for param in params {
        let Some(value) = exec_eval_annotation_type_expr(ctx, &param.ty.node) else {
            if exec_has_pending_exception() {
                return None;
            }
            continue;
        };
        crate::runtime::dict_ops::mb_dict_setitem(
            annotations,
            MbValue::from_ptr(MbObject::new_str(param.name.clone())),
            value,
        );
    }

    if let Some(return_ty) = return_ty {
        if let Some(value) = exec_eval_annotation_type_expr(ctx, &return_ty.node) {
            crate::runtime::dict_ops::mb_dict_setitem(
                annotations,
                MbValue::from_ptr(MbObject::new_str("return".to_string())),
                value,
            );
        } else if exec_has_pending_exception() {
            return None;
        }
    }

    Some(annotations)
}

fn exec_function_field(func: MbValue, key: &str) -> Option<MbValue> {
    let ptr = func.as_ptr()?;
    unsafe {
        let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*ptr).data
        else {
            return None;
        };
        if class_name != "__exec_function__" {
            return None;
        }
        fields.read().unwrap().get(key).copied()
    }
}

pub fn mb_exec_function_is_async(func: MbValue) -> bool {
    exec_function_field(func, "__is_async__")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

pub fn mb_exec_function_call(func: MbValue, args: Vec<MbValue>) -> MbValue {
    if let Some(id) = exec_function_field(func, "__function_id__").and_then(|value| value.as_int())
    {
        let binding = {
            let registry = EXEC_FUNCTIONS.read().unwrap();
            registry.get(&(id as u64)).cloned()
        };
        if let Some(binding) = binding {
            let mut ctx = ExecContext {
                globals: binding.globals,
                frames: binding.captures,
                ..ExecContext::default()
            };
            let return_value = exec_call_function(&mut ctx, &binding.function, &args);
            if exec_has_pending_exception() {
                return MbValue::none();
            }
            if binding.is_async {
                let coro = crate::runtime::async_rt::mb_coroutine_new(
                    MbValue::from_ptr(MbObject::new_str(binding.name)),
                    MbValue::from_ptr(MbObject::new_list(Vec::new())),
                );
                crate::runtime::async_rt::mb_coroutine_complete(coro, return_value);
                return coro;
            }
            return return_value;
        }
    }
    if !args.is_empty() {
        exec_raise_type_error(format!(
            "function takes 0 arguments but {} were given",
            args.len()
        ));
        return MbValue::none();
    }
    let return_value = exec_function_field(func, "__return__").unwrap_or_else(MbValue::none);
    unsafe {
        crate::runtime::rc::retain_if_ptr(return_value);
    }
    if mb_exec_function_is_async(func) {
        let name = exec_function_field(func, "__name__")
            .and_then(eval_str_value)
            .unwrap_or_else(|| "<exec coroutine>".to_string());
        let coro = crate::runtime::async_rt::mb_coroutine_new(
            MbValue::from_ptr(MbObject::new_str(name)),
            MbValue::from_ptr(MbObject::new_list(Vec::new())),
        );
        crate::runtime::async_rt::mb_coroutine_complete(coro, return_value);
        return coro;
    }
    return_value
}

fn exec_truthy(value: MbValue) -> bool {
    if let Some(b) = value.as_bool() {
        return b;
    }
    if let Some(i) = value.as_int_pyint() {
        return i != 0;
    }
    if let Some(f) = value.as_float() {
        return f != 0.0;
    }
    if value.is_none() {
        return false;
    }
    if let Some(ptr) = value.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => !s.is_empty(),
                ObjData::Bytes(b) => !b.is_empty(),
                ObjData::ByteArray(lock) => !lock.read().unwrap().is_empty(),
                ObjData::List(lock) => !lock.read().unwrap().is_empty(),
                ObjData::Tuple(items) => !items.is_empty(),
                ObjData::Dict(lock) => !lock.read().unwrap().is_empty(),
                ObjData::Set(lock) => !lock.read().unwrap().is_empty(),
                ObjData::FrozenSet(items) => !items.is_empty(),
                _ => true,
            }
        }
    } else {
        true
    }
}

fn exec_eval_call_args(
    ctx: &mut ExecContext,
    args: &[crate::parser::ast::CallArg],
) -> Option<Vec<MbValue>> {
    use crate::parser::ast::CallArg;
    let mut values = Vec::with_capacity(args.len());
    for arg in args {
        match arg {
            CallArg::Positional(expr) => {
                values.push(exec_eval_expr(ctx, &expr.node));
                if exec_has_pending_exception() {
                    return None;
                }
            }
            CallArg::Keyword { .. } | CallArg::StarArg(_) | CallArg::DoubleStarArg(_) => {
                return None;
            }
        }
    }
    Some(values)
}

fn exec_range_values(args: &[MbValue]) -> MbValue {
    let (start, stop, step) = match args {
        [stop] => (0, stop.as_int_pyint().unwrap_or(0), 1),
        [start, stop] => (
            start.as_int_pyint().unwrap_or(0),
            stop.as_int_pyint().unwrap_or(0),
            1,
        ),
        [start, stop, step] => (
            start.as_int_pyint().unwrap_or(0),
            stop.as_int_pyint().unwrap_or(0),
            step.as_int_pyint().unwrap_or(0),
        ),
        _ => return MbValue::from_ptr(MbObject::new_list(Vec::new())),
    };
    if step == 0 {
        crate::runtime::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "range() arg 3 must not be zero".to_string(),
            )),
        );
        return MbValue::none();
    }
    let mut items = Vec::new();
    let mut cur = start;
    if step > 0 {
        while cur < stop {
            items.push(MbValue::from_int(cur));
            cur += step;
        }
    } else {
        while cur > stop {
            items.push(MbValue::from_int(cur));
            cur += step;
        }
    }
    MbValue::from_ptr(MbObject::new_list(items))
}

fn exec_lookup_global_pep695_type_param(ctx: &ExecContext, name: &str) -> Option<MbValue> {
    let globals = ctx.globals?;
    let key = MbValue::from_ptr(MbObject::new_str(name.to_string()));
    if !crate::runtime::dict_ops::mb_dict_contains(globals, key)
        .as_bool()
        .unwrap_or(false)
    {
        return None;
    }
    let value = crate::runtime::dict_ops::mb_dict_get(globals, key, MbValue::none());
    if exec_is_pep695_type_param_value(value) {
        Some(value)
    } else {
        None
    }
}

fn exec_eval_generator_element(
    ctx: &mut ExecContext,
    element: &crate::parser::ast::Expr,
) -> MbValue {
    if let crate::parser::ast::Expr::Ident(name) = element {
        if let Some(value) = exec_lookup_global_pep695_type_param(ctx, name) {
            return value;
        }
    }
    exec_eval_expr(ctx, element)
}

fn exec_bind_comprehension_targets(
    ctx: &mut ExecContext,
    targets: &[String],
    value: MbValue,
) -> Vec<ExecTemporaryName> {
    let bindings = targets
        .iter()
        .map(|target| ExecTemporaryName {
            name: target.clone(),
            previous: exec_take_name_binding(ctx, target),
        })
        .collect::<Vec<_>>();
    if targets.len() == 1 {
        exec_store_name(ctx, &targets[0], value);
    } else {
        for (target, item) in targets.iter().zip(extract_items(value)) {
            exec_store_name(ctx, target, item);
        }
    }
    bindings
}

fn exec_visit_comprehension<F>(
    ctx: &mut ExecContext,
    generators: &[crate::parser::ast::Comprehension],
    index: usize,
    emit: &mut F,
) -> bool
where
    F: FnMut(&mut ExecContext) -> bool,
{
    if index >= generators.len() {
        return emit(ctx);
    }

    let generator = &generators[index];
    let iter_value = exec_eval_expr(ctx, &generator.iter.node);
    if exec_has_pending_exception() {
        return false;
    }
    for item in extract_items(iter_value) {
        let masked = exec_bind_comprehension_targets(ctx, &generator.targets, item);
        let mut include = true;
        for condition in &generator.conditions {
            let condition_value = exec_eval_expr(ctx, &condition.node);
            if exec_has_pending_exception() {
                exec_restore_temporary_names(ctx, masked);
                return false;
            }
            if !exec_truthy(condition_value) {
                include = false;
                break;
            }
        }
        if include && !exec_visit_comprehension(ctx, generators, index + 1, emit) {
            exec_restore_temporary_names(ctx, masked);
            return false;
        }
        exec_restore_temporary_names(ctx, masked);
    }
    true
}

fn exec_eval_generator_expr(
    ctx: &mut ExecContext,
    element: &crate::source::span::Spanned<crate::parser::ast::Expr>,
    generators: &[crate::parser::ast::Comprehension],
) -> MbValue {
    let mut values = Vec::new();
    let mut emit = |ctx: &mut ExecContext| {
        let value = exec_eval_generator_element(ctx, &element.node);
        if exec_has_pending_exception() {
            return false;
        }
        values.push(value);
        true
    };
    if exec_visit_comprehension(ctx, generators, 0, &mut emit) {
        MbValue::from_ptr(MbObject::new_list(values))
    } else {
        MbValue::none()
    }
}

fn exec_eval_list_comp(
    ctx: &mut ExecContext,
    element: &crate::source::span::Spanned<crate::parser::ast::Expr>,
    generators: &[crate::parser::ast::Comprehension],
) -> MbValue {
    exec_eval_generator_expr(ctx, element, generators)
}

fn exec_eval_set_comp(
    ctx: &mut ExecContext,
    element: &crate::source::span::Spanned<crate::parser::ast::Expr>,
    generators: &[crate::parser::ast::Comprehension],
) -> MbValue {
    let values = exec_eval_list_comp(ctx, element, generators);
    if exec_has_pending_exception() {
        MbValue::none()
    } else {
        crate::runtime::set_ops::mb_set_from_list(values)
    }
}

fn exec_eval_dict_comp(
    ctx: &mut ExecContext,
    key: &crate::source::span::Spanned<crate::parser::ast::Expr>,
    value: &crate::source::span::Spanned<crate::parser::ast::Expr>,
    generators: &[crate::parser::ast::Comprehension],
) -> MbValue {
    let dict = crate::runtime::dict_ops::mb_dict_new();
    let mut emit = |ctx: &mut ExecContext| {
        let key = exec_eval_generator_element(ctx, &key.node);
        if exec_has_pending_exception() {
            return false;
        }
        let value = exec_eval_generator_element(ctx, &value.node);
        if exec_has_pending_exception() {
            return false;
        }
        crate::runtime::dict_ops::mb_dict_setitem(dict, key, value);
        true
    };
    if exec_visit_comprehension(ctx, generators, 0, &mut emit) {
        dict
    } else {
        MbValue::none()
    }
}

fn exec_call_function(ctx: &mut ExecContext, func: &ExecFunction, args: &[MbValue]) -> MbValue {
    if args.len() > func.params.len() {
        exec_raise_type_error(format!(
            "function takes {} arguments but {} were given",
            func.params.len(),
            args.len()
        ));
        return MbValue::none();
    }
    let mut frame = FxHashMap::default();
    for (idx, param) in func.params.iter().enumerate() {
        let value = if let Some(value) = args.get(idx) {
            *value
        } else if let Some(Some(default)) = func.defaults.get(idx) {
            *default
        } else {
            exec_raise_type_error(format!("missing required argument: '{param}'"));
            return MbValue::none();
        };
        frame.insert(param.clone(), value);
    }
    ctx.frames.push(frame);
    let flow = exec_block_flow(ctx, &func.body);
    ctx.frames.pop();
    match flow {
        ExecFlow::Return(value) => value,
        ExecFlow::Normal | ExecFlow::Break | ExecFlow::Continue => MbValue::none(),
    }
}

fn exec_class_body_namespace(
    ctx: &mut ExecContext,
    body: &[crate::source::span::Spanned<crate::parser::ast::Stmt>],
    in_generic_class_body: bool,
) -> Option<FxHashMap<String, MbValue>> {
    if in_generic_class_body {
        ctx.generic_class_body_depth += 1;
    }
    ctx.frames.push(FxHashMap::default());
    let flow = exec_block_flow(ctx, body);
    let frame = ctx.frames.pop().unwrap_or_default();
    if in_generic_class_body {
        ctx.generic_class_body_depth = ctx.generic_class_body_depth.saturating_sub(1);
    }
    if exec_has_pending_exception() || !matches!(flow, ExecFlow::Normal) {
        return None;
    }
    Some(frame)
}

fn exec_static_return_value(
    ctx: &mut ExecContext,
    body: &[crate::source::span::Spanned<crate::parser::ast::Stmt>],
) -> MbValue {
    let Some(stmt) = body.iter().find(|stmt| {
        matches!(
            stmt.node,
            crate::parser::ast::Stmt::Return(_) | crate::parser::ast::Stmt::ExprStmt(_)
        )
    }) else {
        return MbValue::none();
    };
    match &stmt.node {
        crate::parser::ast::Stmt::Return(Some(expr)) => Some(exec_eval_expr(ctx, &expr.node)),
        crate::parser::ast::Stmt::Return(None) => Some(MbValue::none()),
        crate::parser::ast::Stmt::ExprStmt(expr) => Some(exec_eval_expr(ctx, &expr.node)),
        _ => None,
    }
    .unwrap_or_else(MbValue::none)
}

fn exec_eval_fstring_parts(
    ctx: &mut ExecContext,
    parts: &[crate::parser::ast::FStringPart],
) -> Option<String> {
    let mut out = String::new();
    for part in parts {
        match part {
            crate::parser::ast::FStringPart::Literal(text) => out.push_str(text),
            crate::parser::ast::FStringPart::Expr(expr, spec) => {
                let value = exec_eval_expr(ctx, &expr.node);
                if exec_has_pending_exception() {
                    return None;
                }
                let formatted = match spec {
                    None => crate::runtime::string_ops::mb_fstring_value(value),
                    Some(spec_parts) => {
                        let spec_text = exec_eval_fstring_parts(ctx, spec_parts)?;
                        crate::runtime::string_ops::mb_format_value(
                            value,
                            MbValue::from_ptr(MbObject::new_str(spec_text)),
                        )
                    }
                };
                if exec_has_pending_exception() {
                    return None;
                }
                if let Some(ptr) = formatted.as_ptr() {
                    unsafe {
                        if let ObjData::Str(text) = &(*ptr).data {
                            out.push_str(text);
                        }
                    }
                }
            }
        }
    }
    Some(out)
}

fn exec_eval_expr(ctx: &mut ExecContext, expr: &crate::parser::ast::Expr) -> MbValue {
    use crate::parser::ast::Expr;
    match expr {
        Expr::IntLit(i) => MbValue::from_int(*i),
        Expr::BigIntLit(s) => crate::runtime::bigint_ops::bigint_from_literal(s),
        Expr::FloatLit(f) => MbValue::from_float(*f),
        Expr::BoolLit(b) => MbValue::from_bool(*b),
        Expr::NoneLit => MbValue::none(),
        Expr::StrLit(s) => MbValue::from_ptr(MbObject::new_str(s.clone())),
        Expr::BytesLit(b) => MbValue::from_ptr(MbObject::new_bytes(b.clone())),
        Expr::ComplexLit(imag) => MbValue::from_ptr(MbObject::new_complex(0.0, *imag)),
        Expr::Ellipsis => MbValue::ellipsis(),
        Expr::Ident(name) => {
            if let Some(value) = exec_lookup_name(ctx, name) {
                return value;
            }
            if crate::runtime::exception::is_builtin_exception_name(name) {
                return make_type_object(name);
            }
            crate::runtime::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("NameError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!("name '{name}' is not defined"))),
            );
            MbValue::none()
        }
        Expr::FString(parts) => exec_eval_fstring_parts(ctx, parts)
            .map(|text| MbValue::from_ptr(MbObject::new_str(text)))
            .unwrap_or_else(MbValue::none),
        Expr::BinOp { op, lhs, rhs } => {
            let left = exec_eval_expr(ctx, &lhs.node);
            if exec_has_pending_exception() {
                return MbValue::none();
            }
            match op {
                crate::parser::ast::BinOp::And if !exec_truthy(left) => return left,
                crate::parser::ast::BinOp::Or if exec_truthy(left) => return left,
                _ => {}
            }
            let right = exec_eval_expr(ctx, &rhs.node);
            if exec_has_pending_exception() {
                return MbValue::none();
            }
            eval_binop(*op, left, right)
        }
        Expr::UnaryOp { op, operand } => {
            let value = exec_eval_expr(ctx, &operand.node);
            eval_unaryop(*op, value)
        }
        Expr::ListLit(items) => {
            let mut values = Vec::with_capacity(items.len());
            for item in items {
                values.push(exec_eval_expr(ctx, &item.node));
                if exec_has_pending_exception() {
                    return MbValue::none();
                }
            }
            MbValue::from_ptr(MbObject::new_list(values))
        }
        Expr::TupleLit(items) => {
            let mut values = Vec::with_capacity(items.len());
            for item in items {
                values.push(exec_eval_expr(ctx, &item.node));
                if exec_has_pending_exception() {
                    return MbValue::none();
                }
            }
            MbValue::from_ptr(MbObject::new_tuple(values))
        }
        Expr::SetLit(items) => {
            let mut values = Vec::with_capacity(items.len());
            for item in items {
                values.push(exec_eval_expr(ctx, &item.node));
                if exec_has_pending_exception() {
                    return MbValue::none();
                }
            }
            crate::runtime::set_ops::mb_set_from_list(MbValue::from_ptr(MbObject::new_list(values)))
        }
        Expr::GeneratorExpr {
            element,
            generators,
        } => exec_eval_generator_expr(ctx, element, generators),
        Expr::ListComp {
            element,
            generators,
        } => exec_eval_list_comp(ctx, element, generators),
        Expr::SetComp {
            element,
            generators,
        } => exec_eval_set_comp(ctx, element, generators),
        Expr::DictComp {
            key,
            value,
            generators,
        } => exec_eval_dict_comp(ctx, key, value, generators),
        Expr::DictLit(entries) => {
            let dict = crate::runtime::dict_ops::mb_dict_new();
            for (key, value) in entries {
                if let Some(key_expr) = key {
                    let key = exec_eval_expr(ctx, &key_expr.node);
                    if exec_has_pending_exception() {
                        return MbValue::none();
                    }
                    let value = exec_eval_expr(ctx, &value.node);
                    if exec_has_pending_exception() {
                        return MbValue::none();
                    }
                    crate::runtime::dict_ops::mb_dict_setitem(dict, key, value);
                }
            }
            dict
        }
        Expr::IfExpr {
            body,
            condition,
            else_body,
        } => {
            let condition = exec_eval_expr(ctx, &condition.node);
            if exec_has_pending_exception() {
                return MbValue::none();
            }
            if exec_truthy(condition) {
                exec_eval_expr(ctx, &body.node)
            } else {
                exec_eval_expr(ctx, &else_body.node)
            }
        }
        Expr::ChainedCompare { operands, ops } => {
            if operands.is_empty() {
                return MbValue::from_bool(true);
            }
            let mut prev = exec_eval_expr(ctx, &operands[0].node);
            if exec_has_pending_exception() {
                return MbValue::none();
            }
            for (idx, op) in ops.iter().enumerate() {
                let next = exec_eval_expr(ctx, &operands[idx + 1].node);
                if exec_has_pending_exception() {
                    return MbValue::none();
                }
                let result = eval_binop(*op, prev, next);
                if !result.as_bool().unwrap_or(false) {
                    return MbValue::from_bool(false);
                }
                prev = next;
            }
            MbValue::from_bool(true)
        }
        Expr::Lambda { params, body } => {
            let mut param_names = Vec::with_capacity(params.len());
            let mut defaults = Vec::with_capacity(params.len());
            for param in params {
                param_names.push(param.name.clone());
                let default = match &param.default {
                    Some(default) => {
                        let value = exec_eval_expr(ctx, &default.node);
                        if exec_has_pending_exception() {
                            return MbValue::none();
                        }
                        Some(value)
                    }
                    None => None,
                };
                defaults.push(default);
            }
            let function = ExecFunction {
                params: param_names,
                defaults,
                body: vec![crate::source::span::Spanned::new(
                    crate::parser::ast::Stmt::Return(Some((**body).clone())),
                    body.span,
                )],
            };
            make_exec_function_body_value("<lambda>", false, function, ctx, None)
        }
        Expr::Call { func, args } => {
            let values = match exec_eval_call_args(ctx, args) {
                Some(values) => values,
                None => return MbValue::none(),
            };
            if let Expr::Ident(name) = &func.node {
                if name == crate::lower::pep695::TYPEVAR_INTRINSIC && values.len() == 5 {
                    if let Some(param_name) = exec_string_value(values[0]) {
                        if let Some(reusable) = ctx.type_param_reuse_once.remove(&param_name) {
                            if let Some(existing) = exec_lookup_name(ctx, &param_name) {
                                if exec_is_pep695_type_param_value(existing) {
                                    return existing;
                                }
                            }
                            if exec_is_pep695_type_param_value(reusable) {
                                return reusable;
                            }
                        }
                    }
                    return crate::runtime::pep695::mb_pep695_typevar(
                        values[0], values[1], values[2], values[3], values[4],
                    );
                }
                if name == crate::lower::pep695::TYPE_ALIAS_INTRINSIC && values.len() == 3 {
                    return crate::runtime::pep695::mb_pep695_type_alias(
                        values[0], values[1], values[2],
                    );
                }
                if name == "range" {
                    return exec_range_values(&values);
                }
                if name == "repr" && values.len() == 1 {
                    return mb_repr(values[0]);
                }
                if name == "str" && values.len() == 1 {
                    return mb_str(values[0]);
                }
                if crate::runtime::exception::is_builtin_exception_name(name) {
                    let typ = make_type_object(name);
                    let args_list = MbValue::from_ptr(MbObject::new_list(values));
                    return mb_call_spread(typ, args_list);
                }
                if let Some(func) = ctx.functions.get(name).cloned() {
                    return exec_call_function(ctx, &func, &values);
                }
                // Fall back to a real callable resolved from the supplied
                // globals/locals (e.g. a module-level `def` invoked from
                // `exec(cmd, globals())` as cProfile.run does) so the call
                // goes through the normal compiled call path rather than
                // silently no-op'ing.
                if let Some(callee) = exec_lookup_name(ctx, name) {
                    if !callee.is_none() {
                        let args_list = MbValue::from_ptr(MbObject::new_list(values));
                        return mb_call_spread(callee, args_list);
                    }
                }
            }
            if let Expr::Attr { object, attr } = &func.node {
                let receiver = exec_eval_expr(ctx, &object.node);
                if exec_has_pending_exception() {
                    return MbValue::none();
                }
                if attr == "append" && values.len() == 1 {
                    crate::runtime::list_ops::mb_list_append(receiver, values[0]);
                    return MbValue::none();
                }
                return crate::runtime::class::mb_call_method(
                    receiver,
                    MbValue::from_ptr(MbObject::new_str(attr.clone())),
                    MbValue::from_ptr(MbObject::new_list(values)),
                );
            }
            let callee = exec_eval_expr(ctx, &func.node);
            if exec_has_pending_exception() {
                return MbValue::none();
            }
            if mb_callable(callee).as_bool() == Some(true) {
                return mb_call_spread(callee, MbValue::from_ptr(MbObject::new_list(values)));
            }
            eval_expr(expr)
        }
        Expr::Attr { object, attr } => {
            let receiver = exec_eval_expr(ctx, &object.node);
            if exec_has_pending_exception() {
                return MbValue::none();
            }
            if receiver.is_none() {
                crate::runtime::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "'NoneType' object has no attribute '{attr}'"
                    ))),
                );
                return MbValue::none();
            }
            crate::runtime::class::mb_getattr(
                receiver,
                MbValue::from_ptr(MbObject::new_str(attr.clone())),
            )
        }
        Expr::Index { object, index } => {
            let object = exec_eval_expr(ctx, &object.node);
            if exec_has_pending_exception() {
                return MbValue::none();
            }
            let index = exec_eval_expr(ctx, &index.node);
            if exec_has_pending_exception() {
                return MbValue::none();
            }
            crate::runtime::class::mb_obj_getitem(object, index)
        }
        _ => eval_expr(expr),
    }
}

fn exec_bind_targets(ctx: &mut ExecContext, targets: &[String], value: MbValue) {
    if let [name] = targets {
        exec_store_name(ctx, name, value);
        return;
    }
    let items = extract_items(value);
    for (name, item) in targets.iter().zip(items) {
        exec_store_name(ctx, name, item);
    }
}

fn exec_assign_attr_target(
    ctx: &mut ExecContext,
    object: &crate::source::span::Spanned<crate::parser::ast::Expr>,
    attr: &str,
    value: &crate::source::span::Spanned<crate::parser::ast::Expr>,
) {
    let receiver = exec_eval_expr(ctx, &object.node);
    if exec_has_pending_exception() {
        return;
    }
    let assigned = exec_eval_expr(ctx, &value.node);
    if exec_has_pending_exception() {
        return;
    }
    crate::runtime::class::mb_setattr(
        receiver,
        MbValue::from_ptr(MbObject::new_str(attr.to_string())),
        assigned,
    );
}

fn exec_class_bases_value(
    ctx: &mut ExecContext,
    bases: &[crate::source::span::Spanned<crate::parser::ast::Expr>],
) -> Option<MbValue> {
    let mut base_values = Vec::with_capacity(bases.len());
    for base in bases {
        base_values.push(exec_eval_expr(ctx, &base.node));
        if exec_has_pending_exception() {
            return None;
        }
    }
    Some(MbValue::from_ptr(MbObject::new_list(base_values)))
}

fn exec_instance_field(value: MbValue, field: &str) -> Option<MbValue> {
    value.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(field).copied()
        } else {
            None
        }
    })
}

fn exec_is_generic_alias(value: MbValue) -> bool {
    value.as_ptr().is_some_and(|ptr| unsafe {
        matches!(
            &(*ptr).data,
            ObjData::Instance { class_name, .. }
                if matches!(
                    class_name.as_str(),
                    "GenericAlias" | "types.GenericAlias" | "typing.Alias"
                )
        )
    })
}

fn exec_generic_alias_parameters(bases: &[MbValue]) -> Vec<MbValue> {
    let mut params = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for base in bases {
        let Some(alias_params) = exec_instance_field(*base, "__parameters__") else {
            continue;
        };
        for param in extract_items(alias_params) {
            if seen.insert(param.to_bits()) {
                params.push(param);
            }
        }
    }
    params
}

fn exec_pep695_generic_alias(
    ctx: &ExecContext,
    type_params: &[crate::parser::ast::TypeParam],
) -> Option<MbValue> {
    if type_params.is_empty() {
        return None;
    }
    let mut params = Vec::with_capacity(type_params.len());
    for param in type_params {
        params.push(exec_lookup_name(ctx, &param.name)?);
    }
    let args = if params.len() == 1 {
        params[0]
    } else {
        MbValue::from_ptr(MbObject::new_tuple(params))
    };
    Some(crate::runtime::stdlib::typing_mod::generic_subscript(
        make_type_object("typing.Generic"),
        args,
    ))
}

struct ExecClassGenericMetadata {
    runtime_bases: MbValue,
    orig_bases: Option<MbValue>,
    parameters: Option<MbValue>,
}

fn exec_prepare_class_generic_metadata(
    ctx: &ExecContext,
    type_params: &[crate::parser::ast::TypeParam],
    base_values: MbValue,
) -> ExecClassGenericMetadata {
    let mut bases = extract_items(base_values);
    let has_generic_alias = bases.iter().any(|base| exec_is_generic_alias(*base));
    if !has_generic_alias && type_params.is_empty() {
        return ExecClassGenericMetadata {
            runtime_bases: base_values,
            orig_bases: None,
            parameters: None,
        };
    }
    if let Some(generic_alias) = exec_pep695_generic_alias(ctx, type_params) {
        bases.push(generic_alias);
    }
    let original_bases = MbValue::from_ptr(MbObject::new_tuple(bases.clone()));
    let params = exec_generic_alias_parameters(&bases);
    ExecClassGenericMetadata {
        runtime_bases: MbValue::from_ptr(MbObject::new_list(bases)),
        orig_bases: Some(original_bases),
        parameters: Some(MbValue::from_ptr(MbObject::new_tuple(params))),
    }
}

fn exec_class_metaclass_name(
    ctx: &mut ExecContext,
    keyword_args: &[(String, crate::source::Spanned<crate::parser::ast::Expr>)],
) -> Option<String> {
    let (_, expr) = keyword_args.iter().find(|(name, _)| name == "metaclass")?;
    let value = exec_eval_expr(ctx, &expr.node);
    if exec_has_pending_exception() {
        return None;
    }
    crate::runtime::class::resolve_class_name(value)
}

fn exec_assignment_target_names(expr: &crate::parser::ast::Expr, out: &mut Vec<String>) -> bool {
    use crate::parser::ast::Expr;
    match expr {
        Expr::Ident(name) => {
            out.push(name.clone());
            true
        }
        Expr::TupleLit(items) | Expr::ListLit(items) | Expr::UnpackTarget(items) => items
            .iter()
            .all(|item| exec_assignment_target_names(&item.node, out)),
        Expr::Starred(inner) => exec_assignment_target_names(&inner.node, out),
        _ => false,
    }
}

fn exec_handler_matches(
    ctx: &mut ExecContext,
    handler: &crate::parser::ast::ExceptHandler,
    exc: MbValue,
) -> bool {
    let Some(exc_type) = &handler.exc_type else {
        return true;
    };
    let expected = exec_eval_expr(ctx, &exc_type.node);
    if exec_has_pending_exception() {
        return false;
    }
    let matched = crate::runtime::exception::mb_exception_matches(exc, expected);
    if exec_has_pending_exception() {
        return false;
    }
    matched.as_bool().unwrap_or(false)
}

fn exec_import_stmt(
    ctx: &ExecContext,
    module: &[String],
    names: &Option<Vec<(String, Option<String>)>>,
    module_alias: &Option<String>,
) {
    let module_name = module.join(".");
    let module_name_val = MbValue::from_ptr(MbObject::new_str(module_name.clone()));

    if let Some(names) = names {
        let is_star = names.len() == 1 && names[0].0 == "*";
        if is_star {
            let exports = crate::runtime::module::mb_import_star(module_name_val);
            if exec_has_pending_exception() {
                return;
            }
            for (name, value) in kwargs_dict_pairs(exports) {
                exec_store_global(ctx, &name, value);
            }
            return;
        }

        let _ = crate::runtime::module::mb_import(module_name_val);
        if exec_has_pending_exception() {
            return;
        }
        for (name, alias) in names {
            let attr = MbValue::from_ptr(MbObject::new_str(name.clone()));
            let module_name_val = MbValue::from_ptr(MbObject::new_str(module_name.clone()));
            let value = crate::runtime::module::mb_module_getattr(module_name_val, attr);
            if exec_has_pending_exception() {
                return;
            }
            let bound = alias.as_deref().unwrap_or(name.as_str());
            exec_store_global(ctx, bound, value);
        }
        return;
    }

    let value = crate::runtime::module::mb_import(module_name_val);
    if exec_has_pending_exception() {
        return;
    }
    if let Some(alias) = module_alias {
        exec_store_global(ctx, alias, value);
    } else if let Some(top_name) = module.first() {
        let bound_value = if module.len() > 1 {
            crate::runtime::module::mb_import(MbValue::from_ptr(MbObject::new_str(
                top_name.clone(),
            )))
        } else {
            value
        };
        exec_store_global(ctx, top_name, bound_value);
    }
}

fn exec_stmt_flow(ctx: &mut ExecContext, stmt: &crate::parser::ast::Stmt) -> ExecFlow {
    use crate::parser::ast::Stmt;
    match stmt {
        Stmt::Pass => ExecFlow::Normal,
        Stmt::Import {
            module,
            names,
            module_alias,
        } => {
            exec_import_stmt(ctx, module, names, module_alias);
            ExecFlow::Normal
        }
        Stmt::Assign { target, value } => {
            if let crate::parser::ast::Expr::Attr { object, attr } = &target.node {
                exec_assign_attr_target(ctx, object, attr, value);
                return ExecFlow::Normal;
            }
            let mut target_names = Vec::new();
            if exec_assignment_target_names(&target.node, &mut target_names) {
                if exec_is_typevar_constructor(&value.node) {
                    if let Some(name) = target_names.first() {
                        ctx.type_vars.insert(name.clone());
                    }
                }
                if ctx.globals.is_some() || !ctx.frames.is_empty() {
                    let assigned = exec_eval_expr(ctx, &value.node);
                    if !exec_has_pending_exception() {
                        exec_bind_targets(ctx, &target_names, assigned);
                    }
                }
            }
            ExecFlow::Normal
        }
        Stmt::VarDecl { name, value, .. } => {
            if exec_is_typevar_constructor(&value.node) {
                ctx.type_vars.insert(name.clone());
            }
            if ctx.globals.is_some() || !ctx.frames.is_empty() {
                let assigned = exec_eval_expr(ctx, &value.node);
                if !exec_has_pending_exception() {
                    exec_store_name(ctx, name, assigned);
                }
            }
            ExecFlow::Normal
        }
        Stmt::ClassDef {
            decorators,
            name,
            type_params,
            bases,
            keyword_args,
            body,
            ..
        } => {
            exec_validate_pep695_class_bases(ctx, type_params, bases);
            if exec_has_pending_exception() {
                return ExecFlow::Normal;
            }
            let mut decorator_values = Vec::with_capacity(decorators.len());
            let masked_type_params = exec_mask_type_param_bindings(ctx, type_params);
            for decorator in decorators {
                let value = exec_eval_expr(ctx, &decorator.node);
                if exec_has_pending_exception() {
                    exec_restore_name_bindings(ctx, masked_type_params);
                    return ExecFlow::Normal;
                }
                decorator_values.push(value);
            }
            exec_restore_name_bindings(ctx, masked_type_params);
            let mut match_args = None;
            for class_stmt in body {
                match &class_stmt.node {
                    Stmt::Assign { target, value } => {
                        if let crate::parser::ast::Expr::Ident(attr) = &target.node {
                            if attr == "__match_args__" {
                                match_args = Some(exec_eval_expr(ctx, &value.node));
                                if exec_has_pending_exception() {
                                    return ExecFlow::Normal;
                                }
                            }
                        }
                    }
                    Stmt::VarDecl {
                        name: attr, value, ..
                    } if attr == "__match_args__" => {
                        match_args = Some(exec_eval_expr(ctx, &value.node));
                        if exec_has_pending_exception() {
                            return ExecFlow::Normal;
                        }
                    }
                    _ => {}
                }
            }
            ctx.class_match_args.insert(name.clone(), match_args);
            let Some(base_values) = exec_class_bases_value(ctx, bases) else {
                return ExecFlow::Normal;
            };
            let explicit_metaclass = exec_class_metaclass_name(ctx, keyword_args);
            if exec_has_pending_exception() {
                return ExecFlow::Normal;
            }
            let class_name = MbValue::from_ptr(MbObject::new_str(name.clone()));
            let generic_metadata =
                exec_prepare_class_generic_metadata(ctx, type_params, base_values);
            crate::runtime::class::mb_class_define_multi(
                class_name,
                generic_metadata.runtime_bases,
                MbValue::from_ptr(MbObject::new_list(Vec::new())),
                MbValue::from_ptr(MbObject::new_list(Vec::new())),
            );
            if let Some(meta_name) = explicit_metaclass {
                crate::runtime::class::mb_class_set_metaclass(
                    class_name,
                    MbValue::from_ptr(MbObject::new_str(meta_name)),
                );
            }
            if let Some(orig_bases) = generic_metadata.orig_bases {
                crate::runtime::class::mb_class_set_class_attr(
                    class_name,
                    MbValue::from_ptr(MbObject::new_str("__orig_bases__".to_string())),
                    orig_bases,
                );
            }
            if let Some(parameters) = generic_metadata.parameters {
                crate::runtime::class::mb_class_set_class_attr(
                    class_name,
                    MbValue::from_ptr(MbObject::new_str("__parameters__".to_string())),
                    parameters,
                );
            }
            if let Some(match_args) = match_args {
                crate::runtime::class::mb_class_set_match_args(class_name, match_args);
            }
            if let Some(namespace) = exec_class_body_namespace(ctx, body, !type_params.is_empty()) {
                for (attr, value) in namespace {
                    crate::runtime::class::mb_class_set_class_attr(
                        class_name,
                        MbValue::from_ptr(MbObject::new_str(attr)),
                        value,
                    );
                }
            }
            if exec_has_pending_exception() {
                return ExecFlow::Normal;
            }
            crate::runtime::class::mb_class_finalize_definition(class_name);
            let mut class_value = make_type_object(name);
            for decorator in decorator_values.into_iter().rev() {
                if mb_callable(decorator).as_bool() != Some(true) {
                    let type_name = exec_match_type_name(decorator);
                    crate::runtime::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "'{type_name}' object is not callable"
                        ))),
                    );
                    return ExecFlow::Normal;
                }
                class_value = mb_call_spread(
                    decorator,
                    MbValue::from_ptr(MbObject::new_list(vec![class_value])),
                );
                if exec_has_pending_exception() {
                    return ExecFlow::Normal;
                }
            }
            exec_store_name(ctx, name, class_value);
            ExecFlow::Normal
        }
        Stmt::FnDef {
            decorators,
            name,
            params,
            type_params,
            return_ty,
            body,
            ..
        } => {
            let mut param_names = Vec::with_capacity(params.len());
            let mut defaults = Vec::with_capacity(params.len());
            for param in params {
                param_names.push(param.name.clone());
                let default = match &param.default {
                    Some(default) => {
                        let value = exec_eval_expr(ctx, &default.node);
                        if exec_has_pending_exception() {
                            return ExecFlow::Normal;
                        }
                        Some(value)
                    }
                    None => None,
                };
                defaults.push(default);
            }
            let annotation_type_params = exec_bind_temporary_type_params(ctx, type_params);
            let annotations = exec_function_annotations_value(ctx, params, return_ty.as_ref());
            let Some(annotations) = annotations else {
                exec_restore_temporary_names(ctx, annotation_type_params);
                return ExecFlow::Normal;
            };
            let function = ExecFunction {
                params: param_names,
                defaults,
                body: body.clone(),
            };
            let doc = exec_leading_docstring(body);
            let func_value = make_exec_function_body_value(name, false, function.clone(), ctx, doc);
            crate::runtime::pep695::instance_field_set_pub(
                func_value,
                "__annotations__",
                annotations,
            );
            if let Some(params_tuple) = exec_type_params_tuple_value(ctx, type_params) {
                crate::runtime::pep695::instance_field_set_pub(
                    func_value,
                    "__type_params__",
                    params_tuple,
                );
            }
            if decorators.is_empty() {
                exec_commit_temporary_names(ctx, annotation_type_params);
                ctx.functions.insert(name.clone(), function);
                exec_store_name(ctx, name, func_value);
            } else {
                let mut decorator_values = Vec::with_capacity(decorators.len());
                let masked_type_params = exec_mask_type_param_bindings(ctx, type_params);
                for decorator in decorators {
                    let value = exec_eval_expr(ctx, &decorator.node);
                    if exec_has_pending_exception() {
                        exec_drop_name_bindings(masked_type_params);
                        exec_restore_temporary_names(ctx, annotation_type_params);
                        return ExecFlow::Normal;
                    }
                    decorator_values.push(value);
                }
                let mut decorated_value = func_value;
                for decorator in decorator_values.into_iter().rev() {
                    if mb_callable(decorator).as_bool() != Some(true) {
                        let type_name = exec_match_type_name(decorator);
                        crate::runtime::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(format!(
                                "'{type_name}' object is not callable"
                            ))),
                        );
                        exec_drop_name_bindings(masked_type_params);
                        exec_restore_temporary_names(ctx, annotation_type_params);
                        return ExecFlow::Normal;
                    }
                    decorated_value = mb_call_spread(
                        decorator,
                        MbValue::from_ptr(MbObject::new_list(vec![decorated_value])),
                    );
                    if exec_has_pending_exception() {
                        exec_drop_name_bindings(masked_type_params);
                        exec_restore_temporary_names(ctx, annotation_type_params);
                        return ExecFlow::Normal;
                    }
                }
                exec_restore_name_bindings(ctx, masked_type_params);
                exec_commit_temporary_names(ctx, annotation_type_params);
                exec_store_name(ctx, name, decorated_value);
            }
            ExecFlow::Normal
        }
        Stmt::AsyncFnDef {
            decorators,
            name,
            params,
            type_params,
            return_ty,
            body,
            ..
        } => {
            if decorators.is_empty() {
                let mut param_names = Vec::with_capacity(params.len());
                let mut defaults = Vec::with_capacity(params.len());
                for param in params {
                    param_names.push(param.name.clone());
                    let default = match &param.default {
                        Some(default) => {
                            let value = exec_eval_expr(ctx, &default.node);
                            if exec_has_pending_exception() {
                                return ExecFlow::Normal;
                            }
                            Some(value)
                        }
                        None => None,
                    };
                    defaults.push(default);
                }
                let annotation_type_params = exec_bind_temporary_type_params(ctx, type_params);
                let annotations = exec_function_annotations_value(ctx, params, return_ty.as_ref());
                let Some(annotations) = annotations else {
                    exec_restore_temporary_names(ctx, annotation_type_params);
                    return ExecFlow::Normal;
                };
                exec_commit_temporary_names(ctx, annotation_type_params);
                let function = ExecFunction {
                    params: param_names,
                    defaults,
                    body: body.clone(),
                };
                let doc = exec_leading_docstring(body);
                let func_value = make_exec_function_body_value(name, true, function, ctx, doc);
                crate::runtime::pep695::instance_field_set_pub(
                    func_value,
                    "__annotations__",
                    annotations,
                );
                if let Some(params_tuple) = exec_type_params_tuple_value(ctx, type_params) {
                    crate::runtime::pep695::instance_field_set_pub(
                        func_value,
                        "__type_params__",
                        params_tuple,
                    );
                }
                exec_store_name(ctx, name, func_value);
            }
            ExecFlow::Normal
        }
        Stmt::Return(value) => {
            let value = value
                .as_ref()
                .map(|expr| exec_eval_expr(ctx, &expr.node))
                .unwrap_or_else(MbValue::none);
            ExecFlow::Return(value)
        }
        Stmt::Break => ExecFlow::Break,
        Stmt::Continue => ExecFlow::Continue,
        Stmt::If {
            condition,
            body,
            elif_clauses,
            else_body,
        } => {
            let condition = exec_eval_expr(ctx, &condition.node);
            if exec_has_pending_exception() {
                return ExecFlow::Normal;
            }
            if exec_truthy(condition) {
                return exec_block_flow(ctx, body);
            }
            for (elif, elif_body) in elif_clauses {
                let condition = exec_eval_expr(ctx, &elif.node);
                if exec_has_pending_exception() {
                    return ExecFlow::Normal;
                }
                if exec_truthy(condition) {
                    return exec_block_flow(ctx, elif_body);
                }
            }
            if let Some(else_body) = else_body {
                exec_block_flow(ctx, else_body)
            } else {
                ExecFlow::Normal
            }
        }
        Stmt::For {
            targets,
            iter,
            body,
            else_body,
            ..
        } => {
            let iterable = exec_eval_expr(ctx, &iter.node);
            if exec_has_pending_exception() {
                return ExecFlow::Normal;
            }
            let mut broke = false;
            for item in extract_items(iterable) {
                exec_bind_targets(ctx, targets, item);
                let flow = exec_block_flow(ctx, body);
                if exec_has_pending_exception() {
                    return ExecFlow::Normal;
                }
                match flow {
                    ExecFlow::Normal => {}
                    ExecFlow::Continue => continue,
                    ExecFlow::Break => {
                        broke = true;
                        break;
                    }
                    ExecFlow::Return(_) => return flow,
                }
            }
            if !broke {
                if let Some(else_body) = else_body {
                    return exec_block_flow(ctx, else_body);
                }
            }
            ExecFlow::Normal
        }
        Stmt::ExprStmt(expr) => {
            let _ = exec_eval_expr(ctx, &expr.node);
            ExecFlow::Normal
        }
        Stmt::Match { expr, arms } => {
            if let Some(subject_class) = exec_subject_class_name(&expr.node) {
                for arm in arms {
                    exec_validate_class_pattern(ctx, &subject_class, &arm.pattern.node);
                    if exec_has_pending_exception() {
                        return ExecFlow::Normal;
                    }
                }
            }
            ExecFlow::Normal
        }
        Stmt::Raise { value, from } => {
            let Some(value) = value else {
                crate::runtime::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("RuntimeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        "No active exception to reraise".to_string(),
                    )),
                );
                return ExecFlow::Normal;
            };
            let raised = exec_eval_expr(ctx, &value.node);
            if exec_has_pending_exception() {
                return ExecFlow::Normal;
            }
            if let Some(cause_expr) = from {
                let _ = exec_eval_expr(ctx, &cause_expr.node);
                if exec_has_pending_exception() {
                    return ExecFlow::Normal;
                }
            }
            crate::runtime::class::mb_raise_instance(raised);
            ExecFlow::Normal
        }
        Stmt::Try {
            body,
            handlers,
            else_body,
            finally_body,
        } => {
            let mut flow = exec_block_flow(ctx, body);
            let mut unhandled = None;
            if exec_has_pending_exception() {
                let exc = crate::runtime::exception::mb_catch_exception();
                let mut handled = false;
                for handler in handlers {
                    if exec_handler_matches(ctx, handler, exc) {
                        if let Some(name) = &handler.name {
                            exec_store_name(ctx, name, exc);
                        }
                        flow = exec_block_flow(ctx, &handler.body);
                        handled = true;
                        break;
                    }
                }
                if !handled {
                    unhandled = Some(exc);
                    flow = ExecFlow::Normal;
                }
            } else if matches!(flow, ExecFlow::Normal) {
                if let Some(else_body) = else_body {
                    flow = exec_block_flow(ctx, else_body);
                }
            }
            if let Some(finally_body) = finally_body {
                let finally_flow = exec_block_flow(ctx, finally_body);
                if !matches!(finally_flow, ExecFlow::Normal) || exec_has_pending_exception() {
                    unhandled = None;
                    flow = finally_flow;
                }
            }
            if let Some(exc) = unhandled {
                crate::runtime::exception::mb_reraise(exc);
            }
            flow
        }
        Stmt::With { items, body } => {
            let mut managers = Vec::with_capacity(items.len());
            for item in items {
                let manager = exec_eval_expr(ctx, &item.context.node);
                if exec_has_pending_exception() {
                    return ExecFlow::Normal;
                }
                let _ = crate::runtime::class::mb_context_enter(manager);
                if exec_has_pending_exception() {
                    return ExecFlow::Normal;
                }
                managers.push(manager);
            }
            let flow = exec_block_flow(ctx, body);
            for manager in managers.into_iter().rev() {
                let _ = crate::runtime::class::mb_context_exit(manager, MbValue::none());
            }
            flow
        }
        _ => ExecFlow::Normal,
    }
}

fn exec_block_flow(
    ctx: &mut ExecContext,
    stmts: &[crate::source::span::Spanned<crate::parser::ast::Stmt>],
) -> ExecFlow {
    for stmt in stmts {
        let flow = exec_stmt_flow(ctx, &stmt.node);
        if exec_has_pending_exception() || !matches!(flow, ExecFlow::Normal) {
            return flow;
        }
    }
    ExecFlow::Normal
}

fn exec_stmt(ctx: &mut ExecContext, stmt: &crate::parser::ast::Stmt) {
    let _ = exec_stmt_flow(ctx, stmt);
}

fn exec_stmts_with_context(
    ctx: &mut ExecContext,
    stmts: &[crate::source::span::Spanned<crate::parser::ast::Stmt>],
) {
    let _ = exec_block_flow(ctx, stmts);
}

fn exec_stmts(stmts: &[crate::source::span::Spanned<crate::parser::ast::Stmt>]) {
    let mut ctx = ExecContext::default();
    exec_stmts_with_context(&mut ctx, stmts);
}

fn eval_binop(op: crate::parser::ast::BinOp, l: MbValue, r: MbValue) -> MbValue {
    use crate::parser::ast::BinOp as B;
    match op {
        B::Add => mb_add(l, r),
        B::Sub => mb_sub(l, r),
        B::Mul => mb_mul(l, r),
        B::Div => mb_div(l, r),
        B::FloorDiv => mb_floordiv(l, r),
        B::Mod => mb_mod(l, r),
        B::Pow => mb_pow(l, r),
        B::MatMul => crate::runtime::class::mb_matmul(l, r),
        B::Eq => mb_eq(l, r),
        B::NotEq => mb_ne(l, r),
        B::Lt => mb_lt(l, r),
        B::Gt => mb_gt(l, r),
        B::LtEq => mb_le(l, r),
        B::GtEq => mb_ge(l, r),
        B::And => {
            if l.as_bool().unwrap_or(false) || l.as_int().unwrap_or(0) != 0 {
                r
            } else {
                l
            }
        }
        B::Or => {
            if l.as_bool().unwrap_or(false) || l.as_int().unwrap_or(0) != 0 {
                l
            } else {
                r
            }
        }
        B::BitAnd => {
            if let (Some(a), Some(b)) = (l.as_int(), r.as_int()) {
                MbValue::from_int(a & b)
            } else {
                MbValue::none()
            }
        }
        B::BitOr => {
            if let (Some(a), Some(b)) = (l.as_int(), r.as_int()) {
                MbValue::from_int(a | b)
            } else {
                MbValue::none()
            }
        }
        B::BitXor => {
            if let (Some(a), Some(b)) = (l.as_int(), r.as_int()) {
                MbValue::from_int(a ^ b)
            } else {
                MbValue::none()
            }
        }
        B::LShift => {
            if let (Some(a), Some(b)) = (l.as_int(), r.as_int()) {
                MbValue::from_int(a.wrapping_shl(b as u32))
            } else {
                MbValue::none()
            }
        }
        B::RShift => {
            if let (Some(a), Some(b)) = (l.as_int(), r.as_int()) {
                MbValue::from_int(a.wrapping_shr(b as u32))
            } else {
                MbValue::none()
            }
        }
        B::Is => mb_is_identity(l, r),
        B::IsNot => mb_is_not_identity(l, r),
        B::In | B::NotIn => MbValue::none(),
    }
}

fn eval_unaryop(op: crate::parser::ast::UnaryOp, v: MbValue) -> MbValue {
    use crate::parser::ast::UnaryOp as U;
    match op {
        U::Pos => v,
        U::Neg => mb_neg(v),
        U::Not => mb_not(v),
        U::BitNot => {
            if let Some(i) = v.as_int() {
                MbValue::from_int(!i)
            } else {
                MbValue::none()
            }
        }
    }
}

/// exec(code) — execute a string of code (#1256, partial).
///
/// Mamba does not yet expose a full runtime scope hook, but `exec(src, globals)`
/// does mutate the supplied globals dict for the supported interpreted subset.
/// It still validates the input so common defensive patterns
/// (`try: exec(src) except SyntaxError: ...`) behave like CPython:
///   * Non-string input → silent no-op returning None (matches the previous
///     stub; raising TypeError here would break benches that already pass
///     compiled code objects).
///   * String input → parse as a module; raise SyntaxError on failure.
///   * A narrow runtime subset (assignments, imports, interpreted zero-arg
///     functions, if/for, try/except/else/finally, expression statements,
///     raise, and with cleanup) is executed so exceptions and control flow
///     propagate through `exec`.
/// Remaining side-effecting statements are still dropped on the floor; see #1256.
pub fn mb_exec(code: MbValue) -> MbValue {
    mb_exec_impl(code, None, None)
}

pub fn mb_exec_with_globals(code: MbValue, globals: MbValue) -> MbValue {
    mb_exec_impl(code, Some(globals), None)
}

pub fn mb_exec_with_globals_locals(code: MbValue, globals: MbValue, locals: MbValue) -> MbValue {
    mb_exec_impl(code, Some(globals), Some(locals))
}

fn pep695_class_annotation_comprehension_syntax_error(source: &str) -> bool {
    fn leading_indent(line: &str) -> usize {
        line.chars()
            .take_while(|ch| matches!(ch, ' ' | '\t'))
            .count()
    }

    fn has_type_params_before_call(line: &str) -> bool {
        let Some(open_paren) = line.find('(') else {
            return false;
        };
        let prefix = &line[..open_paren];
        prefix.contains('[') && prefix.contains(']')
    }

    fn has_class_type_params(line: &str) -> bool {
        let header = line.split_once(':').map_or(line, |(header, _)| header);
        header.contains('[') && header.contains(']')
    }

    let mut generic_class_indents: Vec<usize> = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = leading_indent(line);
        while generic_class_indents
            .last()
            .is_some_and(|class_indent| indent <= *class_indent)
        {
            generic_class_indents.pop();
        }

        if trimmed.starts_with("class ") {
            if has_class_type_params(trimmed) {
                generic_class_indents.push(indent);
            }
            continue;
        }

        if generic_class_indents.is_empty() {
            continue;
        }
        let method = trimmed
            .strip_prefix("def ")
            .or_else(|| trimmed.strip_prefix("async def "));
        let Some(method) = method else {
            continue;
        };
        if !has_type_params_before_call(method) {
            continue;
        }
        if method.contains(": (") && method.contains(" for ") {
            return true;
        }
    }
    false
}

fn mb_exec_impl(code: MbValue, globals: Option<MbValue>, locals: Option<MbValue>) -> MbValue {
    use crate::lexer;
    use crate::parser::Parser;
    use crate::source::SourceMap;

    if let Some(ptr) = code.as_ptr() {
        unsafe {
            if let ObjData::CodeObject { ast, .. } = &(*ptr).data {
                let mut ast = ast.clone();
                crate::lower::pep695::desugar_module(&mut ast);
                let mut ctx = ExecContext {
                    globals,
                    locals,
                    ..ExecContext::default()
                };
                exec_stmts_with_context(&mut ctx, &ast.stmts);
                return MbValue::none();
            }
        }
    }

    let source = if let Some(ptr) = code.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => s.clone(),
                _ => return MbValue::none(),
            }
        }
    } else {
        return MbValue::none();
    };

    let mut source_map = SourceMap::new();
    let file_id = source_map.add_file("<exec>".to_string(), source.clone());
    let tokens = lexer::lex(&source, file_id);
    let mut parser = Parser::new(tokens, &source, file_id);
    parser.skip_newlines();
    let mut module = match parser.parse_module() {
        Ok(module) => module,
        Err(err) => {
            let message = match err {
                _ if pep695_class_annotation_comprehension_syntax_error(&source) => {
                    "Cannot use comprehension in annotation scope within class scope".to_string()
                }
                crate::error::MambaError::Syntax { message, .. }
                    if message.contains("invalid number") =>
                {
                    "invalid binary literal (<string>, line 1)".to_string()
                }
                _ => "invalid syntax (<string>, line 1)".to_string(),
            };
            crate::runtime::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("SyntaxError".to_string())),
                MbValue::from_ptr(MbObject::new_str(message)),
            );
            return MbValue::none();
        }
    };
    crate::lower::pep695::desugar_module(&mut module);
    let mut ctx = ExecContext {
        globals,
        locals,
        ..ExecContext::default()
    };
    exec_stmts_with_context(&mut ctx, &module.stmts);
    MbValue::none()
}

// @spec .aw/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md#R1
// @spec .aw/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md#R2
// @spec .aw/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md#R3
// @spec .aw/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md#R4
// @spec .aw/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md#R5
// @spec .aw/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-runtime.md#R6
/// compile(source, filename, mode[, flags, dont_inherit]) — compile source to a code object (#976).
///
/// Returns a heap-allocated `CodeObject` (ObjData::CodeObject) wrapping the parsed AST,
/// filename, mode, and original source. The code object is designed to be consumed by
/// exec()/eval() once #441 lands.
///
/// Raises:
/// - `ValueError` for unknown mode strings.
/// - `SyntaxError` for parse failures (with line/column info).
/// - `SyntaxError` when eval mode source is a statement, not an expression.
/// - `SyntaxError` when single mode source contains multiple statements.
pub fn mb_compile(source: MbValue, filename: MbValue, mode: MbValue) -> MbValue {
    mb_compile_impl(
        source,
        filename,
        mode,
        MbValue::from_int(0),
        MbValue::from_bool(false),
        -1,
    )
}

/// compile(source, filename, mode, flags, dont_inherit) — 5-argument form (R5).
pub fn mb_compile_5(
    source: MbValue,
    filename: MbValue,
    mode: MbValue,
    _flags: MbValue,
    _dont_inherit: MbValue,
) -> MbValue {
    mb_compile_impl(source, filename, mode, _flags, _dont_inherit, -1)
}

/// compile(source, filename, mode, **kwargs) — keyword-argument form, used
/// when the call site passes `optimize=` (or any other keyword) so the
/// dynamic dispatch path (`mb_call_spread_kwargs`) folds the kwargs dict
/// into a trailing positional arg (see `dispatch_compile` in
/// `runtime/stdlib/builtins_mod.rs`). Only `optimize` is honored; other
/// keywords (`flags`, `dont_inherit`) are accepted but not yet meaningful.
pub fn mb_compile_kwargs(
    source: MbValue,
    filename: MbValue,
    mode: MbValue,
    kwargs: MbValue,
) -> MbValue {
    let optimize_key = MbValue::from_ptr(MbObject::new_str("optimize".to_string()));
    let optimize_val =
        crate::runtime::dict_ops::mb_dict_get(kwargs, optimize_key, MbValue::from_int(-1));
    let optimize: i64 = optimize_val.as_int().unwrap_or(-1);
    mb_compile_impl(
        source,
        filename,
        mode,
        MbValue::from_int(0),
        MbValue::from_bool(false),
        optimize,
    )
}

fn compile_ast_constant_as_name_error(source: MbValue) -> Option<String> {
    use crate::runtime::rc::ObjData;

    fn instance_field_value(node: MbValue, field: &str) -> Option<MbValue> {
        node.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.read().unwrap().get(field).copied()
            } else {
                None
            }
        })
    }

    fn instance_class_name(node: MbValue) -> Option<String> {
        node.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                Some(class_name.clone())
            } else {
                None
            }
        })
    }

    let body = match source.as_ptr() {
        Some(ptr) => unsafe {
            match &(*ptr).data {
                ObjData::Instance { class_name, fields } if class_name == "Expression" => {
                    fields.read().unwrap().get("body").copied()?
                }
                _ => return None,
            }
        },
        None => return None,
    };

    if instance_class_name(body).as_deref() != Some("Name") {
        return None;
    }
    if instance_class_name(instance_field_value(body, "ctx")?).as_deref() != Some("Load") {
        return None;
    }
    let ident = instance_field_value(body, "id")?
        .as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })?;
    match ident.as_str() {
        "True" | "False" | "None" => Some(format!(
            "identifier field can't represent '{ident}' constant"
        )),
        _ => None,
    }
}

fn compile_ast_exec_import_from_module(
    source: MbValue,
) -> Option<(String, crate::parser::ast::Module)> {
    use crate::parser::ast::{Module, Stmt};
    use crate::runtime::rc::ObjData;
    use crate::source::{Span, Spanned};

    fn instance_field_value(node: MbValue, field: &str) -> Option<MbValue> {
        node.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.read().unwrap().get(field).copied()
            } else {
                None
            }
        })
    }

    fn instance_class_name(node: MbValue) -> Option<String> {
        node.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                Some(class_name.clone())
            } else {
                None
            }
        })
    }

    fn list_items(value: MbValue) -> Option<Vec<MbValue>> {
        value.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::List(ref items) = (*ptr).data {
                Some(items.read().unwrap().iter().copied().collect())
            } else {
                None
            }
        })
    }

    fn str_value(value: MbValue) -> Option<String> {
        value.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
    }

    fn optional_str_value(value: MbValue) -> Option<Option<String>> {
        if value.is_none() {
            Some(None)
        } else {
            str_value(value).map(Some)
        }
    }

    if instance_class_name(source).as_deref() != Some("Module") {
        return None;
    }

    let body = list_items(instance_field_value(source, "body")?)?;
    if body.len() != 1 {
        return None;
    }

    let import_from = body[0];
    if instance_class_name(import_from).as_deref() != Some("ImportFrom") {
        return None;
    }

    let module_name = optional_str_value(instance_field_value(import_from, "module")?)?;
    let level_value = instance_field_value(import_from, "level").unwrap_or_else(MbValue::none);
    let level = if level_value.is_none() {
        0usize
    } else {
        usize::try_from(level_value.as_int()?).ok()?
    };

    let alias_nodes = list_items(instance_field_value(import_from, "names")?)?;
    if alias_nodes.is_empty() {
        return None;
    }

    let mut names = Vec::with_capacity(alias_nodes.len());
    let mut rendered_names = Vec::with_capacity(alias_nodes.len());
    for alias in alias_nodes {
        if instance_class_name(alias).as_deref() != Some("alias") {
            return None;
        }
        let name = str_value(instance_field_value(alias, "name")?)?;
        let asname = optional_str_value(
            instance_field_value(alias, "asname").unwrap_or_else(MbValue::none),
        )?;
        rendered_names.push(match &asname {
            Some(asname) => format!("{name} as {asname}"),
            None => name.clone(),
        });
        names.push((name, asname));
    }

    let module_path = module_name
        .as_deref()
        .map(|name| {
            name.split('.')
                .filter(|part| !part.is_empty())
                .map(|part| part.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if module_path.is_empty() && level == 0 {
        return None;
    }

    let source_text = format!(
        "from {}{} import {}",
        ".".repeat(level),
        module_name.as_deref().unwrap_or(""),
        rendered_names.join(", ")
    );
    Some((
        source_text,
        Module {
            stmts: vec![Spanned::new(
                Stmt::Import {
                    module: module_path,
                    names: Some(names),
                    module_alias: None,
                },
                Span::dummy(),
            )],
        },
    ))
}

struct CompileErrorDetails {
    exc_type: &'static str,
    message: String,
    lineno: u32,
    offset: Option<u32>,
    text: String,
    end_lineno: Option<u32>,
    end_offset: Option<u32>,
}

fn raise_compile_exception_instance(filename: &str, details: CompileErrorDetails) {
    let mut location = vec![
        MbValue::from_ptr(MbObject::new_str(filename.to_string())),
        MbValue::from_int(details.lineno as i64),
        details
            .offset
            .map(|offset| MbValue::from_int(offset as i64))
            .unwrap_or_else(MbValue::none),
        MbValue::from_ptr(MbObject::new_str(details.text)),
    ];
    if details.end_lineno.is_some() || details.end_offset.is_some() {
        location.push(MbValue::from_int(
            details.end_lineno.unwrap_or(details.lineno) as i64,
        ));
        location.push(
            details
                .end_offset
                .map(|offset| MbValue::from_int(offset as i64))
                .unwrap_or_else(MbValue::none),
        );
    }
    let args = MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_ptr(MbObject::new_str(details.message)),
        MbValue::from_ptr(MbObject::new_tuple(location)),
    ]));
    let instance = crate::runtime::exception::mb_exception_new_with_args(
        MbValue::from_ptr(MbObject::new_str(details.exc_type.to_string())),
        args,
    );
    if !instance.is_none() {
        crate::runtime::class::mb_raise_instance(instance);
    }
}

fn compile_error_details_from_span(
    source_map: &crate::source::SourceMap,
    span: crate::source::Span,
    exc_type: &'static str,
    message: impl Into<String>,
) -> CompileErrorDetails {
    let file = source_map.get_file(span.file);
    let (lineno, offset) = file.line_col(span.start);
    let text = file.line_text(lineno).to_string();
    let end = if span.end > span.start {
        span.end.saturating_sub(1)
    } else {
        span.start
    };
    let (end_lineno, end_col_inclusive) = file.line_col(end);
    CompileErrorDetails {
        exc_type,
        message: message.into(),
        lineno,
        offset: Some(offset),
        text,
        end_lineno: Some(end_lineno),
        end_offset: Some(end_col_inclusive + 1),
    }
}

fn first_non_lexed_span(
    source: &str,
    file_id: crate::source::FileId,
    raw_tokens: &[crate::lexer::token::Token],
) -> Option<crate::source::Span> {
    fn first_non_whitespace_byte(source: &str, start: usize, end: usize) -> Option<(u32, u32)> {
        let mut idx = start;
        while idx < end {
            let ch = source[idx..end].chars().next()?;
            if !ch.is_whitespace() {
                let next = idx + ch.len_utf8();
                return Some((idx as u32, next as u32));
            }
            idx += ch.len_utf8();
        }
        None
    }

    let mut cursor = 0usize;
    for token in raw_tokens {
        let start = token.start as usize;
        if let Some((gap_start, gap_end)) = first_non_whitespace_byte(source, cursor, start) {
            return Some(crate::source::Span::new(file_id, gap_start, gap_end));
        }
        cursor = token.end as usize;
    }
    first_non_whitespace_byte(source, cursor, source.len())
        .map(|(gap_start, gap_end)| crate::source::Span::new(file_id, gap_start, gap_end))
}

fn detect_compile_indentation_error(
    raw_tokens: &[crate::lexer::token::Token],
    source_map: &crate::source::SourceMap,
    file_id: crate::source::FileId,
) -> Option<CompileErrorDetails> {
    use crate::lexer::token::TokenKind;

    let file = source_map.get_file(file_id);
    let mut indent_stack = vec![0u32];
    let mut at_line_start = true;
    let mut paren_depth = 0u32;
    let mut last_newline_end = 0u32;
    let mut indent_allowed = false;
    let mut last_sig_on_line: Option<TokenKind> = None;

    for token in raw_tokens {
        let check_indent = |token: &crate::lexer::token::Token,
                            indent_stack: &mut Vec<u32>,
                            indent_allowed: &mut bool|
         -> Option<CompileErrorDetails> {
            let indent = token.start.saturating_sub(last_newline_end);
            let current = *indent_stack.last().unwrap_or(&0);
            if indent > current {
                if !*indent_allowed {
                    let (lineno, _) = file.line_col(token.start);
                    return Some(CompileErrorDetails {
                        exc_type: "IndentationError",
                        message: "unexpected indent".to_string(),
                        lineno,
                        offset: None,
                        text: file.line_text(lineno).to_string(),
                        end_lineno: None,
                        end_offset: None,
                    });
                }
                indent_stack.push(indent);
            } else if indent < current {
                while let Some(&top) = indent_stack.last() {
                    if top <= indent {
                        break;
                    }
                    indent_stack.pop();
                }
                if indent_stack.last().copied().unwrap_or(0) != indent {
                    let (lineno, _) = file.line_col(token.start);
                    let text = file.line_text(lineno).to_string();
                    let caret = text.chars().count() as u32 + 1;
                    return Some(CompileErrorDetails {
                        exc_type: "IndentationError",
                        message: "unindent does not match any outer indentation level".to_string(),
                        lineno,
                        offset: Some(caret),
                        text,
                        end_lineno: None,
                        end_offset: None,
                    });
                }
            } else if *indent_allowed {
                let (lineno, offset) = file.line_col(token.start);
                return Some(CompileErrorDetails {
                    exc_type: "IndentationError",
                    message: "expected an indented block".to_string(),
                    lineno,
                    offset: Some(offset),
                    text: file.line_text(lineno).to_string(),
                    end_lineno: None,
                    end_offset: None,
                });
            }
            *indent_allowed = false;
            None
        };

        match &token.kind {
            TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => {
                if at_line_start && paren_depth == 0 {
                    if let Some(err) = check_indent(token, &mut indent_stack, &mut indent_allowed) {
                        return Some(err);
                    }
                }
                paren_depth += 1;
                at_line_start = false;
                last_sig_on_line = Some(token.kind.clone());
            }
            TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                paren_depth = paren_depth.saturating_sub(1);
                at_line_start = false;
                last_sig_on_line = Some(token.kind.clone());
            }
            TokenKind::Newline => {
                if paren_depth == 0 {
                    at_line_start = true;
                    last_newline_end = token.end;
                    indent_allowed = matches!(last_sig_on_line, Some(TokenKind::Colon));
                    last_sig_on_line = None;
                }
            }
            TokenKind::Comment => {}
            TokenKind::Eof => break,
            _ => {
                if at_line_start && paren_depth == 0 {
                    if let Some(err) = check_indent(token, &mut indent_stack, &mut indent_allowed) {
                        return Some(err);
                    }
                }
                at_line_start = false;
                last_sig_on_line = Some(token.kind.clone());
            }
        }
    }
    if indent_allowed {
        if let Some(token) = raw_tokens.last() {
            let file = source_map.get_file(file_id);
            let (lineno, offset) = file.line_col(token.end);
            return Some(CompileErrorDetails {
                exc_type: "IndentationError",
                message: "expected an indented block".to_string(),
                lineno,
                offset: Some(offset),
                text: file.line_text(lineno).to_string(),
                end_lineno: None,
                end_offset: None,
            });
        }
    }
    None
}

fn mb_compile_impl(
    source: MbValue,
    filename: MbValue,
    mode: MbValue,
    _flags: MbValue,
    _dont_inherit: MbValue,
    optimize: i64,
) -> MbValue {
    use crate::lexer;
    use crate::parser::{ast::Module, Parser};
    use crate::runtime::rc::ObjData;
    use crate::source::SourceMap;

    if let Some(msg) = compile_ast_constant_as_name_error(source) {
        raise_value_error(msg);
        return MbValue::none();
    }

    let ast_filename_str: String = if let Some(ptr) = filename.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => s.clone(),
                _ => "<string>".to_string(),
            }
        }
    } else {
        "<string>".to_string()
    };

    let ast_mode_str: String = if let Some(ptr) = mode.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => s.clone(),
                _ => String::new(),
            }
        }
    } else {
        String::new()
    };

    if ast_mode_str != "exec" && ast_mode_str != "eval" && ast_mode_str != "single" {
        crate::runtime::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "compile() mode must be 'exec', 'eval' or 'single'".to_string(),
            )),
        );
        return MbValue::none();
    }

    if ast_mode_str == "exec" {
        if let Some((source_str, mut ast)) = compile_ast_exec_import_from_module(source) {
            if optimize >= 2 {
                strip_ast_docstrings(&mut ast.stmts);
            }
            return MbValue::from_ptr(MbObject::new_code_object(
                source_str,
                ast_filename_str,
                ast_mode_str,
                ast,
            ));
        }
    }

    // ── Extract source string (R1 / R6 bytes support) ──────────────────────
    let source_str: String = if let Some(ptr) = source.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => s.clone(),
                ObjData::Bytes(data) => {
                    // R6: decode bytes as UTF-8
                    match std::str::from_utf8(data) {
                        Ok(s) => s.to_string(),
                        Err(_) => {
                            crate::runtime::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(
                                    "compile() source bytes are not valid UTF-8".to_string(),
                                )),
                            );
                            return MbValue::none();
                        }
                    }
                }
                _ => {
                    crate::runtime::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(
                            "compile() source must be a string or bytes".to_string(),
                        )),
                    );
                    return MbValue::none();
                }
            }
        }
    } else {
        crate::runtime::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "compile() source must be a string or bytes".to_string(),
            )),
        );
        return MbValue::none();
    };

    // ── Extract filename string (R3) ────────────────────────────────────────
    let filename_str: String = if let Some(ptr) = filename.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => s.clone(),
                _ => "<string>".to_string(),
            }
        }
    } else {
        "<string>".to_string()
    };

    // ── Extract mode string (R2) ────────────────────────────────────────────
    let mode_str: String = if let Some(ptr) = mode.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => s.clone(),
                _ => String::new(),
            }
        }
    } else {
        String::new()
    };

    // Validate mode (R2)
    if mode_str != "exec" && mode_str != "eval" && mode_str != "single" {
        crate::runtime::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "compile() mode must be 'exec', 'eval' or 'single'".to_string(),
            )),
        );
        return MbValue::none();
    }

    // ── Build SourceFile for error location (R3 / R4) ──────────────────────
    let mut source_map = SourceMap::new();
    let file_id = source_map.add_file(filename_str.clone(), source_str.clone());

    // ── Parse according to mode (R2 / R4) ──────────────────────────────────
    let raw_tokens = lexer::lex_raw(&source_str, file_id);
    if let Some(span) = first_non_lexed_span(&source_str, file_id, &raw_tokens) {
        let details =
            compile_error_details_from_span(&source_map, span, "SyntaxError", "invalid syntax");
        raise_compile_exception_instance(&filename_str, details);
        return MbValue::none();
    }

    let tokens = lexer::lex(&source_str, file_id);
    if mode_str != "eval" {
        if let Some(details) = detect_compile_indentation_error(&raw_tokens, &source_map, file_id) {
            raise_compile_exception_instance(&filename_str, details);
            return MbValue::none();
        }
    }
    let mut parser = Parser::new(tokens, &source_str, file_id);

    let ast: Module = match mode_str.as_str() {
        "exec" => {
            // Parse as full module (any number of statements)
            match parser.parse_module() {
                Ok(m) => m,
                Err(err) => {
                    let span = err
                        .span()
                        .unwrap_or_else(|| crate::source::Span::new(file_id, 0, 0));
                    let details = compile_error_details_from_span(
                        &source_map,
                        span,
                        "SyntaxError",
                        err.to_string(),
                    );
                    raise_compile_exception_instance(&filename_str, details);
                    return MbValue::none();
                }
            }
        }
        "eval" => {
            // Parse as a single expression (R2: statements are rejected)
            parser.skip_newlines();
            match parser.parse_expr() {
                Ok(expr) => {
                    // Check that nothing remains after the expression
                    parser.skip_newlines();
                    let remaining = parser.peek_kind();
                    if remaining.is_some() && remaining != Some(crate::lexer::token::TokenKind::Eof)
                    {
                        let span = parser
                            .peek()
                            .map(|token| crate::source::Span::new(file_id, token.start, token.end))
                            .unwrap_or_else(|| crate::source::Span::new(file_id, 0, 0));
                        let details = compile_error_details_from_span(
                            &source_map,
                            span,
                            "SyntaxError",
                            "invalid syntax",
                        );
                        raise_compile_exception_instance(&filename_str, details);
                        return MbValue::none();
                    }
                    // Wrap expression in a Module
                    use crate::parser::ast::Stmt;
                    use crate::source::Spanned;
                    let span = expr.span;
                    Module {
                        stmts: vec![Spanned::new(Stmt::ExprStmt(expr), span)],
                    }
                }
                Err(err) => {
                    // Could be a statement — give the CPython-compatible message
                    let span = err
                        .span()
                        .unwrap_or_else(|| crate::source::Span::new(file_id, 0, 0));
                    let details = compile_error_details_from_span(
                        &source_map,
                        span,
                        "SyntaxError",
                        "invalid syntax",
                    );
                    raise_compile_exception_instance(&filename_str, details);
                    return MbValue::none();
                }
            }
        }
        "single" => {
            // Parse exactly one statement (R2: multi-statement is rejected)
            parser.skip_newlines();
            match parser.parse_stmt() {
                Ok(stmt) => {
                    parser.skip_newlines();
                    let remaining = parser.peek_kind();
                    if remaining.is_some() && remaining != Some(crate::lexer::token::TokenKind::Eof)
                    {
                        let span = parser
                            .peek()
                            .map(|token| crate::source::Span::new(file_id, token.start, token.end))
                            .unwrap_or_else(|| crate::source::Span::new(file_id, 0, 0));
                        let details = compile_error_details_from_span(
                            &source_map,
                            span,
                            "SyntaxError",
                            "multiple statements found while compiling a single statement",
                        );
                        raise_compile_exception_instance(&filename_str, details);
                        return MbValue::none();
                    }
                    Module { stmts: vec![stmt] }
                }
                Err(err) => {
                    let span = err
                        .span()
                        .unwrap_or_else(|| crate::source::Span::new(file_id, 0, 0));
                    let details = compile_error_details_from_span(
                        &source_map,
                        span,
                        "SyntaxError",
                        err.to_string(),
                    );
                    raise_compile_exception_instance(&filename_str, details);
                    return MbValue::none();
                }
            }
        }
        _ => unreachable!("mode already validated"),
    };

    if let Some(err) = validate_compile_nonlocal_declarations(&ast) {
        let span = err
            .span()
            .unwrap_or_else(|| crate::source::Span::new(file_id, 0, 0));
        let details =
            compile_error_details_from_span(&source_map, span, "SyntaxError", err.to_string());
        raise_compile_exception_instance(&filename_str, details);
        return MbValue::none();
    }

    // ── optimize=2 strips docstrings from the compiled module/function/class
    // bodies (matches CPython's -OO / optimize=2 contract) ─────────────────
    let mut ast = ast;
    if optimize >= 2 {
        strip_ast_docstrings(&mut ast.stmts);
    }

    // ── Return CodeObject (R1) ──────────────────────────────────────────────
    MbValue::from_ptr(MbObject::new_code_object(
        source_str,
        filename_str,
        mode_str,
        ast,
    ))
}

/// Remove docstrings (a leading bare string-literal expression statement) from
/// `stmts` (a module/function/class body — a "doc-bearing" body) and
/// recursively from every nested function/class body found within, for
/// `compile(..., optimize=2)` (R7). Mirrors CPython's `-OO` docstring
/// stripping. Control-flow bodies (if/while/for/try/with/match) are not
/// doc-bearing themselves — a leading string there is an ordinary (unused)
/// expression statement, not a docstring — but are still walked so that any
/// function/class defs nested inside them get stripped too.
fn strip_ast_docstrings(stmts: &mut Vec<crate::source::Spanned<crate::parser::ast::Stmt>>) {
    use crate::parser::ast::{Expr, Stmt};

    // Strip this body's own leading docstring, if present.
    if let Some(first) = stmts.first() {
        if let Stmt::ExprStmt(e) = &first.node {
            if let Expr::StrLit(_) = &e.node {
                stmts.remove(0);
            }
        }
    }

    // Recurse to find nested doc-bearing bodies (function/class defs) and
    // walk (without stripping) generic control-flow bodies.
    for stmt in stmts.iter_mut() {
        strip_ast_docstrings_in_stmt(&mut stmt.node);
    }
}

/// Recurse into `stmt`'s nested statement bodies looking for function/class
/// defs to strip docstrings from (see `strip_ast_docstrings`).
fn strip_ast_docstrings_in_stmt(stmt: &mut crate::parser::ast::Stmt) {
    use crate::parser::ast::Stmt;

    // A generic (non-doc-bearing) nested body: walk each statement without
    // stripping the body's own leading string.
    fn walk(stmts: &mut [crate::source::Spanned<Stmt>]) {
        for s in stmts.iter_mut() {
            strip_ast_docstrings_in_stmt(&mut s.node);
        }
    }

    match stmt {
        Stmt::FnDef { body, .. } | Stmt::AsyncFnDef { body, .. } | Stmt::ClassDef { body, .. } => {
            strip_ast_docstrings(body);
        }
        Stmt::If {
            body,
            elif_clauses,
            else_body,
            ..
        } => {
            walk(body);
            for (_, clause_body) in elif_clauses.iter_mut() {
                walk(clause_body);
            }
            if let Some(eb) = else_body {
                walk(eb);
            }
        }
        Stmt::While {
            body, else_body, ..
        }
        | Stmt::For {
            body, else_body, ..
        }
        | Stmt::AsyncFor {
            body, else_body, ..
        } => {
            walk(body);
            if let Some(eb) = else_body {
                walk(eb);
            }
        }
        Stmt::Try {
            body,
            handlers,
            else_body,
            finally_body,
            ..
        } => {
            walk(body);
            for handler in handlers.iter_mut() {
                walk(&mut handler.body);
            }
            if let Some(eb) = else_body {
                walk(eb);
            }
            if let Some(fb) = finally_body {
                walk(fb);
            }
        }
        Stmt::With { body, .. } | Stmt::AsyncWith { body, .. } => {
            walk(body);
        }
        Stmt::Match { arms, .. } => {
            for arm in arms.iter_mut() {
                walk(&mut arm.body);
            }
        }
        _ => {}
    }
}

fn validate_compile_nonlocal_declarations(
    module: &crate::parser::ast::Module,
) -> Option<crate::error::MambaError> {
    use crate::parser::ast::Stmt;
    use std::collections::HashSet;

    fn function_bindings(
        params: &[crate::parser::ast::Param],
        body: &[crate::source::Spanned<Stmt>],
    ) -> HashSet<String> {
        let mut assigned = Vec::new();
        let mut declared = Vec::new();
        crate::resolve::pass::collect_assignment_targets(body, &mut assigned, &mut declared);
        crate::resolve::pass::collect_walrus_targets_in_stmts(body, &mut assigned);

        let mut bindings: HashSet<String> = params.iter().map(|p| p.name.clone()).collect();
        for name in assigned {
            if !declared.iter().any(|decl| decl == &name) {
                bindings.insert(name);
            }
        }
        bindings
    }

    fn visit(
        stmts: &[crate::source::Spanned<Stmt>],
        function_scopes: &mut Vec<HashSet<String>>,
    ) -> Option<crate::error::MambaError> {
        for stmt in stmts {
            match &stmt.node {
                Stmt::Nonlocal(names) => {
                    for name in names {
                        if !function_scopes
                            .iter()
                            .rev()
                            .any(|scope| scope.contains(name))
                        {
                            return Some(crate::error::MambaError::syntax(
                                stmt.span,
                                format!("no binding for nonlocal '{name}' found"),
                            ));
                        }
                    }
                }
                Stmt::FnDef { params, body, .. } | Stmt::AsyncFnDef { params, body, .. } => {
                    function_scopes.push(function_bindings(params, body));
                    if let Some(err) = visit(body, function_scopes) {
                        return Some(err);
                    }
                    function_scopes.pop();
                }
                Stmt::ClassDef { body, .. } => {
                    if let Some(err) = visit(body, function_scopes) {
                        return Some(err);
                    }
                }
                Stmt::If {
                    body,
                    elif_clauses,
                    else_body,
                    ..
                } => {
                    if let Some(err) = visit(body, function_scopes) {
                        return Some(err);
                    }
                    for (_, elif_body) in elif_clauses {
                        if let Some(err) = visit(elif_body, function_scopes) {
                            return Some(err);
                        }
                    }
                    if let Some(else_body) = else_body {
                        if let Some(err) = visit(else_body, function_scopes) {
                            return Some(err);
                        }
                    }
                }
                Stmt::While {
                    body, else_body, ..
                }
                | Stmt::For {
                    body, else_body, ..
                }
                | Stmt::AsyncFor {
                    body, else_body, ..
                } => {
                    if let Some(err) = visit(body, function_scopes) {
                        return Some(err);
                    }
                    if let Some(else_body) = else_body {
                        if let Some(err) = visit(else_body, function_scopes) {
                            return Some(err);
                        }
                    }
                }
                Stmt::With { body, .. } | Stmt::AsyncWith { body, .. } => {
                    if let Some(err) = visit(body, function_scopes) {
                        return Some(err);
                    }
                }
                Stmt::Try {
                    body,
                    handlers,
                    else_body,
                    finally_body,
                } => {
                    if let Some(err) = visit(body, function_scopes) {
                        return Some(err);
                    }
                    for handler in handlers {
                        if let Some(err) = visit(&handler.body, function_scopes) {
                            return Some(err);
                        }
                    }
                    if let Some(else_body) = else_body {
                        if let Some(err) = visit(else_body, function_scopes) {
                            return Some(err);
                        }
                    }
                    if let Some(finally_body) = finally_body {
                        if let Some(err) = visit(finally_body, function_scopes) {
                            return Some(err);
                        }
                    }
                }
                Stmt::Match { arms, .. } => {
                    for arm in arms {
                        if let Some(err) = visit(&arm.body, function_scopes) {
                            return Some(err);
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    visit(&module.stmts, &mut Vec::new())
}

/// Format a MambaError as a SyntaxError message with file/line/col (R4).
fn format_syntax_error(
    err: &crate::error::MambaError,
    source_map: &crate::source::SourceMap,
    _filename: &str,
) -> String {
    if let Some(span) = err.span() {
        let file = source_map.get_file(span.file);
        let (line, col) = file.line_col(span.start);
        format!("{} (line {} col {})", err, line, col)
    } else {
        format!("{}", err)
    }
}

/// globals() — return module global namespace as a dict.
///
// HANDWRITE-BEGIN gap="standardize:projects-mamba-src-runtime-builtins-rs" tracker="standardize-gap-projects-mamba-src-runtime-builtins-rs" reason="introspection-builtins (issue: enhancement-mamba-introspection-builtins-globals-locals-vars-dir)."
/// Reads from the runtime SymbolId → (name, type-tag) registry populated by
/// the driver / module loader before JIT entry. NaN-boxes raw values from
/// GLOBAL_ID_NAMESPACE per the recorded type tag and unions in any
/// user-defined functions tracked in MODULE_FUNC_INFO.
/// Frame-local introspection (CPython `globals()` returning the *enclosing*
/// module's namespace from inside a function) works here because mamba's
/// JIT shares one GLOBAL_ID_NAMESPACE per module — there is only ever one
/// "current module" namespace at runtime.
/// @spec .aw/tech-design/cclab-mamba/logic/introspection-builtins.md#globals_impl
pub fn mb_globals() -> MbValue {
    crate::runtime::closure::build_globals_dict()
}
// HANDWRITE-END

/// locals() — return current frame local namespace as a dict.
///
// HANDWRITE-BEGIN gap="standardize:projects-mamba-src-runtime-builtins-rs" tracker="standardize-gap-projects-mamba-src-runtime-builtins-rs" reason="introspection-builtins (issue: enhancement-mamba-introspection-builtins-globals-locals-vars-dir)."
/// Frame-local snapshot is not supported — mamba's JIT keeps locals in
/// VRegs without a frame-dict, so a true `locals()` would need a per-call
/// metadata side-channel. At module and class scope CPython treats
/// `locals()` as equivalent to `globals()`, which is what we return here
/// (best-effort match for the most common usage: pytest fixture
/// discovery, debugger probes at REPL/module level). Inside a function
/// the result is still the module globals — incorrect by CPython spec but
/// preferable to an empty dict for current callers. `vars()` zero-arg
/// routes here at codegen time (hir_to_mir.rs).
/// @spec .aw/tech-design/cclab-mamba/logic/introspection-builtins.md#locals_impl
pub fn mb_locals() -> MbValue {
    crate::runtime::closure::build_globals_dict()
}
// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_lookup_globals_miss_falls_back_to_builtins() {
        crate::runtime::module::mb_register_builtins();
        let globals = crate::runtime::dict_ops::mb_dict_new();
        let ctx = ExecContext {
            globals: Some(globals),
            ..ExecContext::default()
        };

        let dict_value = exec_lookup_name(&ctx, "dict").expect("dict builtin should resolve");
        assert_eq!(
            eval_str_value(mb_repr(dict_value)).as_deref(),
            Some("<class 'dict'>")
        );
        assert!(
            exec_lookup_name(&ctx, "__definitely_missing_exec_name__").is_none(),
            "unknown names should still miss"
        );
    }

    #[test]
    fn test_exec_function_annotations_capture_class_scope_values() {
        crate::runtime::module::mb_register_builtins();
        crate::runtime::exception::mb_clear_exception();

        let globals = crate::runtime::dict_ops::mb_dict_new();
        mb_exec_with_globals(
            MbValue::from_ptr(MbObject::new_str(
                "class C:\n    x = 1\n    def method[T](self, arg: x):\n        pass\n".to_string(),
            )),
            globals,
        );
        assert!(
            !crate::runtime::exception::mb_has_exception()
                .as_bool()
                .unwrap_or(false),
            "exec should complete without raising: {:?}",
            (
                crate::runtime::exception::current_exception_type(),
                crate::runtime::class::peek_last_raised_instance()
                    .and_then(crate::runtime::exception::get_exception_message_pub)
            )
        );

        let cls = crate::runtime::dict_ops::mb_dict_get(
            globals,
            MbValue::from_ptr(MbObject::new_str("C".to_string())),
            MbValue::none(),
        );
        let method = crate::runtime::class::mb_getattr(
            cls,
            MbValue::from_ptr(MbObject::new_str("method".to_string())),
        );
        let annotations = crate::runtime::class::mb_getattr(
            method,
            MbValue::from_ptr(MbObject::new_str("__annotations__".to_string())),
        );
        let arg = crate::runtime::dict_ops::mb_dict_get(
            annotations,
            MbValue::from_ptr(MbObject::new_str("arg".to_string())),
            MbValue::none(),
        );
        assert_eq!(arg.as_int(), Some(1));
    }

    #[test]
    fn test_exec_generic_fn_default_does_not_see_type_param() {
        crate::runtime::module::mb_register_builtins();
        crate::runtime::exception::mb_clear_exception();

        let globals = crate::runtime::dict_ops::mb_dict_new();
        mb_exec_with_globals(
            MbValue::from_ptr(MbObject::new_str(
                "def func[T](arg = list[T]()):\n    pass\n".to_string(),
            )),
            globals,
        );

        assert!(
            crate::runtime::exception::mb_has_exception()
                .as_bool()
                .unwrap_or(false),
            "exec should raise NameError for a generic-function default reading its type param",
        );
        assert_eq!(
            crate::runtime::exception::current_exception_type().as_deref(),
            Some("NameError")
        );
        crate::runtime::exception::mb_clear_exception();
        assert_eq!(
            crate::runtime::dict_ops::mb_dict_contains(
                globals,
                MbValue::from_ptr(MbObject::new_str("T".to_string())),
            )
            .as_bool(),
            Some(false),
            "failed generic-function default should not leak its type param"
        );
        assert_eq!(
            crate::runtime::dict_ops::mb_dict_contains(
                globals,
                MbValue::from_ptr(MbObject::new_str("func".to_string())),
            )
            .as_bool(),
            Some(false),
            "failed generic-function default should not bind the function"
        );
    }

    #[test]
    fn test_exec_generic_fn_annotation_sees_type_param() {
        crate::runtime::module::mb_register_builtins();
        crate::runtime::exception::mb_clear_exception();

        let globals = crate::runtime::dict_ops::mb_dict_new();
        mb_exec_with_globals(
            MbValue::from_ptr(MbObject::new_str(
                "def func[T](arg: T):\n    pass\n".to_string(),
            )),
            globals,
        );

        assert!(
            !crate::runtime::exception::mb_has_exception()
                .as_bool()
                .unwrap_or(false),
            "exec should let generic function annotations read their type params"
        );
        let func = crate::runtime::dict_ops::mb_dict_get(
            globals,
            MbValue::from_ptr(MbObject::new_str("func".to_string())),
            MbValue::none(),
        );
        let annotations = crate::runtime::class::mb_getattr(
            func,
            MbValue::from_ptr(MbObject::new_str("__annotations__".to_string())),
        );
        let arg = crate::runtime::dict_ops::mb_dict_get(
            annotations,
            MbValue::from_ptr(MbObject::new_str("arg".to_string())),
            MbValue::none(),
        );
        let type_params = crate::runtime::class::mb_getattr(
            func,
            MbValue::from_ptr(MbObject::new_str("__type_params__".to_string())),
        );
        let params = extract_items(type_params);
        assert_eq!(params.len(), 1);
        assert_eq!(arg.to_bits(), params[0].to_bits());
        let name = crate::runtime::class::mb_getattr(
            arg,
            MbValue::from_ptr(MbObject::new_str("__name__".to_string())),
        );
        let name_text = name.as_ptr().and_then(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::Str(value) => Some(value.clone()),
                _ => None,
            }
        });
        assert_eq!(name_text.as_deref(), Some("T"));
    }

    #[test]
    fn test_exec_generic_fn_decorator_does_not_see_type_param() {
        crate::runtime::module::mb_register_builtins();
        crate::runtime::exception::mb_clear_exception();

        let globals = crate::runtime::dict_ops::mb_dict_new();
        mb_exec_with_globals(
            MbValue::from_ptr(MbObject::new_str(
                "def my_decorator(a):\n    return lambda f: f\n@my_decorator(T)\ndef func[T]():\n    pass\n"
                    .to_string(),
            )),
            globals,
        );

        assert!(
            crate::runtime::exception::mb_has_exception()
                .as_bool()
                .unwrap_or(false),
            "exec should raise NameError for a generic-function decorator reading its type param",
        );
        assert_eq!(
            crate::runtime::exception::current_exception_type().as_deref(),
            Some("NameError")
        );
        crate::runtime::exception::mb_clear_exception();
        assert_eq!(
            crate::runtime::dict_ops::mb_dict_contains(
                globals,
                MbValue::from_ptr(MbObject::new_str("T".to_string())),
            )
            .as_bool(),
            Some(false),
            "failed decorated generic function should not leak its type param"
        );
        assert_eq!(
            crate::runtime::dict_ops::mb_dict_contains(
                globals,
                MbValue::from_ptr(MbObject::new_str("func".to_string())),
            )
            .as_bool(),
            Some(false),
            "failed decorated generic function should not bind the function"
        );
    }

    #[test]
    fn test_exec_decorated_fn_applies_to_function_value() {
        crate::runtime::module::mb_register_builtins();
        crate::runtime::exception::mb_clear_exception();

        let globals = crate::runtime::dict_ops::mb_dict_new();
        mb_exec_with_globals(
            MbValue::from_ptr(MbObject::new_str(
                "def deco(func):\n    return func\n@deco\ndef func():\n    return 1\n".to_string(),
            )),
            globals,
        );

        assert!(
            !crate::runtime::exception::mb_has_exception()
                .as_bool()
                .unwrap_or(false),
            "exec should apply decorators to the function object"
        );
        let func = crate::runtime::dict_ops::mb_dict_get(
            globals,
            MbValue::from_ptr(MbObject::new_str("func".to_string())),
            MbValue::none(),
        );
        assert!(
            !func.is_none(),
            "decorated function should stay bound in globals"
        );
        let result = mb_call_spread(func, MbValue::from_ptr(MbObject::new_list(vec![])));
        assert_eq!(result.as_int(), Some(1));
    }

    #[test]
    fn test_pep695_class_annotation_comprehension_syntax_error_classifier() {
        assert!(pep695_class_annotation_comprehension_syntax_error(
            "class C[T]:\n    T = \"class\"\n    def meth[U](x: (T for _ in (1,)), y: T):\n        pass\n"
        ));
        assert!(!pep695_class_annotation_comprehension_syntax_error(
            "def f(x: (T for _ in (1,))):\n    pass\n"
        ));
        assert!(!pep695_class_annotation_comprehension_syntax_error(
            "class C:\n    def meth(x: (T for _ in (1,))):\n        pass\n"
        ));
    }
}
