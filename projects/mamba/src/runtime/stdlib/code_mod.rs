use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::parser::ast;
use crate::source::span::{FileId, Spanned};
use rustc_hash::FxHashMap;
/// code module for Mamba (#1261, #20).
///
/// Real `InteractiveInterpreter` / `InteractiveConsole` classes over a small
/// AST-walking exec engine, plus `compile_command` backed by the real mamba
/// parser. The engine covers the interactive-source subset the conformance
/// fixtures exercise: assignments, expression statements, `def` + calls of
/// the defined functions, `return`, `if`, and int/float/str arithmetic over
/// names resolved against the interpreter's shared locals dict.
///
/// Completeness classification (codeop semantics):
/// - parse error at EOF                          → incomplete (None / push True)
/// - parse OK, last stmt compound, no final "\n" → incomplete (REPL blank-line rule)
/// - parse OK otherwise                          → complete
/// - any other parse error                       → SyntaxError
use std::collections::HashMap;

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Str(s) => Some(s.clone()),
            _ => None,
        }
    })
}

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn raise(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(new_str(exc), new_str(msg));
    MbValue::none()
}

/// Write to the live sys.stderr (respecting contextlib.redirect_stderr).
fn write_stderr(s: &str) {
    if !super::super::output::try_write_stderr_redirect(s) {
        eprint!("{s}");
    }
}

// ── Source classification ────────────────────────────────────────────────

enum SourceState {
    Complete(ast::Module),
    Incomplete,
    Error(String),
}

fn stmt_is_compound(stmt: &ast::Stmt) -> bool {
    matches!(
        stmt,
        ast::Stmt::FnDef { .. }
            | ast::Stmt::ClassDef { .. }
            | ast::Stmt::If { .. }
            | ast::Stmt::While { .. }
            | ast::Stmt::For { .. }
            | ast::Stmt::With { .. }
            | ast::Stmt::Try { .. }
            | ast::Stmt::Match { .. }
    )
}

fn classify_source(source: &str) -> SourceState {
    match crate::parser::parse(source, FileId::default()) {
        Ok(module) => {
            // REPL blank-line rule: a trailing compound statement only
            // completes once the user enters a blank line (the joined buffer
            // then ends with '\n').
            let last_compound = module
                .stmts
                .last()
                .map(|s| stmt_is_compound(&s.node))
                .unwrap_or(false);
            if last_compound && !source.ends_with('\n') {
                SourceState::Incomplete
            } else {
                SourceState::Complete(module)
            }
        }
        Err(err) => {
            let msg = err.to_string();
            if msg.contains("EOF") || msg.contains("end of input") {
                SourceState::Incomplete
            } else {
                SourceState::Error(msg)
            }
        }
    }
}

// ── Mini exec engine ─────────────────────────────────────────────────────

/// A Python-level error from the interpreted snippet: (exception type, message).
struct ExecErr(String, String);

impl ExecErr {
    fn new(ty: &str, msg: impl Into<String>) -> Self {
        ExecErr(ty.to_string(), msg.into())
    }
}

enum Flow {
    Normal,
    Return(MbValue),
}

/// Interpreted `def`s live here; the value stored in the namespace is an
/// Instance (class_name "function") carrying `__interp_id__`.
struct InterpFunc {
    params: Vec<String>,
    defaults: Vec<Option<MbValue>>,
    body: Vec<Spanned<ast::Stmt>>,
}

thread_local! {
    static INTERP_FUNCS: std::cell::RefCell<HashMap<i64, InterpFunc>> =
        std::cell::RefCell::new(HashMap::new());
    static NEXT_INTERP_ID: std::cell::Cell<i64> = const { std::cell::Cell::new(1) };
}

/// Execution environment: an optional function-call frame over the shared
/// globals dict (the interpreter's locals mapping).
struct Env<'a> {
    frame: Option<&'a mut FxHashMap<String, MbValue>>,
    globals: MbValue,
}

impl Env<'_> {
    fn get(&self, name: &str) -> Option<MbValue> {
        if let Some(frame) = &self.frame {
            if let Some(&v) = frame.get(name) {
                return Some(v);
            }
        }
        let key = new_str(name);
        let contains = super::super::dict_ops::mb_dict_contains(self.globals, key)
            .as_bool()
            .unwrap_or(false);
        if contains {
            Some(super::super::dict_ops::mb_dict_get(
                self.globals,
                key,
                MbValue::none(),
            ))
        } else {
            None
        }
    }

    fn set(&mut self, name: &str, value: MbValue) {
        if let Some(frame) = &mut self.frame {
            frame.insert(name.to_string(), value);
        } else {
            super::super::dict_ops::mb_dict_setitem(self.globals, new_str(name), value);
        }
    }
}

fn truthy(v: MbValue) -> bool {
    if let Some(b) = v.as_bool() {
        return b;
    }
    if let Some(i) = v.as_int() {
        return i != 0;
    }
    if let Some(f) = v.as_float() {
        return f != 0.0;
    }
    if v.is_none() {
        return false;
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => return !s.is_empty(),
                ObjData::List(l) => return !l.read().unwrap().is_empty(),
                ObjData::Tuple(t) => return !t.is_empty(),
                _ => {}
            }
        }
    }
    true
}

fn num_binop(op: &ast::BinOp, a: MbValue, b: MbValue) -> Result<MbValue, ExecErr> {
    use ast::BinOp::*;
    // String concat first.
    if let (Some(x), Some(y)) = (extract_str(a), extract_str(b)) {
        if matches!(op, Add) {
            return Ok(new_str(&format!("{x}{y}")));
        }
    }
    let as_f = |v: MbValue| v.as_float().or_else(|| v.as_int().map(|i| i as f64));
    let both_int = a.as_int().is_some() && b.as_int().is_some();
    if both_int {
        let (x, y) = (a.as_int().unwrap(), b.as_int().unwrap());
        return match op {
            Add => Ok(MbValue::from_int(x.wrapping_add(y))),
            Sub => Ok(MbValue::from_int(x.wrapping_sub(y))),
            Mul => Ok(MbValue::from_int(x.wrapping_mul(y))),
            Div => {
                if y == 0 {
                    Err(ExecErr::new("ZeroDivisionError", "division by zero"))
                } else {
                    Ok(MbValue::from_float(x as f64 / y as f64))
                }
            }
            FloorDiv => {
                if y == 0 {
                    Err(ExecErr::new(
                        "ZeroDivisionError",
                        "integer division or modulo by zero",
                    ))
                } else {
                    Ok(MbValue::from_int(x.div_euclid(y)))
                }
            }
            Mod => {
                if y == 0 {
                    Err(ExecErr::new(
                        "ZeroDivisionError",
                        "integer division or modulo by zero",
                    ))
                } else {
                    Ok(MbValue::from_int(x.rem_euclid(y)))
                }
            }
            Pow => {
                if y >= 0 {
                    Ok(MbValue::from_int(x.pow(y.min(63) as u32)))
                } else {
                    Ok(MbValue::from_float((x as f64).powf(y as f64)))
                }
            }
            Eq => Ok(MbValue::from_bool(x == y)),
            NotEq => Ok(MbValue::from_bool(x != y)),
            Lt => Ok(MbValue::from_bool(x < y)),
            Gt => Ok(MbValue::from_bool(x > y)),
            LtEq => Ok(MbValue::from_bool(x <= y)),
            GtEq => Ok(MbValue::from_bool(x >= y)),
            _ => Err(ExecErr::new("TypeError", "unsupported operand type(s)")),
        };
    }
    if let (Some(x), Some(y)) = (as_f(a), as_f(b)) {
        return match op {
            Add => Ok(MbValue::from_float(x + y)),
            Sub => Ok(MbValue::from_float(x - y)),
            Mul => Ok(MbValue::from_float(x * y)),
            Div => {
                if y == 0.0 {
                    Err(ExecErr::new("ZeroDivisionError", "float division by zero"))
                } else {
                    Ok(MbValue::from_float(x / y))
                }
            }
            FloorDiv => {
                if y == 0.0 {
                    Err(ExecErr::new(
                        "ZeroDivisionError",
                        "float floor division by zero",
                    ))
                } else {
                    Ok(MbValue::from_float((x / y).floor()))
                }
            }
            Mod => {
                if y == 0.0 {
                    Err(ExecErr::new("ZeroDivisionError", "float modulo"))
                } else {
                    Ok(MbValue::from_float(x.rem_euclid(y)))
                }
            }
            Pow => Ok(MbValue::from_float(x.powf(y))),
            Eq => Ok(MbValue::from_bool(x == y)),
            NotEq => Ok(MbValue::from_bool(x != y)),
            Lt => Ok(MbValue::from_bool(x < y)),
            Gt => Ok(MbValue::from_bool(x > y)),
            LtEq => Ok(MbValue::from_bool(x <= y)),
            GtEq => Ok(MbValue::from_bool(x >= y)),
            _ => Err(ExecErr::new("TypeError", "unsupported operand type(s)")),
        };
    }
    // Generic equality fallback for non-numeric operands.
    match op {
        Eq => Ok(super::super::builtins::mb_eq(a, b)),
        NotEq => Ok(MbValue::from_bool(
            super::super::builtins::mb_eq(a, b).as_bool() != Some(true),
        )),
        _ => Err(ExecErr::new("TypeError", "unsupported operand type(s)")),
    }
}

fn interp_func_id(v: MbValue) -> Option<i64> {
    v.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*ptr).data
        {
            if class_name == "function" {
                return fields
                    .read()
                    .unwrap()
                    .get("__interp_id__")
                    .and_then(|x| x.as_int());
            }
        }
        None
    })
}

fn eval_expr(expr: &Spanned<ast::Expr>, env: &mut Env) -> Result<MbValue, ExecErr> {
    match &expr.node {
        ast::Expr::IntLit(i) => Ok(MbValue::from_int(*i)),
        ast::Expr::BigIntLit(s) => Ok(super::super::bigint_ops::bigint_from_literal(s)),
        ast::Expr::FloatLit(f) => Ok(MbValue::from_float(*f)),
        ast::Expr::StrLit(s) => Ok(new_str(s)),
        ast::Expr::BoolLit(b) => Ok(MbValue::from_bool(*b)),
        ast::Expr::NoneLit => Ok(MbValue::none()),
        ast::Expr::Ident(name) => match name.as_str() {
            "True" => Ok(MbValue::from_bool(true)),
            "False" => Ok(MbValue::from_bool(false)),
            "None" => Ok(MbValue::none()),
            _ => env
                .get(name)
                .ok_or_else(|| ExecErr::new("NameError", format!("name '{name}' is not defined"))),
        },
        ast::Expr::BinOp { op, lhs, rhs } => {
            use ast::BinOp::*;
            match op {
                And => {
                    let l = eval_expr(lhs, env)?;
                    if !truthy(l) {
                        return Ok(l);
                    }
                    eval_expr(rhs, env)
                }
                Or => {
                    let l = eval_expr(lhs, env)?;
                    if truthy(l) {
                        return Ok(l);
                    }
                    eval_expr(rhs, env)
                }
                _ => {
                    let a = eval_expr(lhs, env)?;
                    let b = eval_expr(rhs, env)?;
                    num_binop(op, a, b)
                }
            }
        }
        ast::Expr::Call { func, args } => {
            let callee = eval_expr(func, env)?;
            let mut arg_vals: Vec<MbValue> = Vec::new();
            for a in args {
                match a {
                    ast::CallArg::Positional(e) => arg_vals.push(eval_expr(e, env)?),
                    _ => {
                        return Err(ExecErr::new(
                            "TypeError",
                            "interactive exec supports positional arguments only",
                        ))
                    }
                }
            }
            if let Some(id) = interp_func_id(callee) {
                return call_interp_func(id, &arg_vals, env.globals);
            }
            Err(ExecErr::new("TypeError", "object is not callable"))
        }
        ast::Expr::TupleLit(items) => {
            let mut vals = Vec::new();
            for e in items {
                vals.push(eval_expr(e, env)?);
            }
            Ok(MbValue::from_ptr(MbObject::new_tuple(vals)))
        }
        _ => Err(ExecErr::new(
            "TypeError",
            "unsupported expression in interactive exec",
        )),
    }
}

fn call_interp_func(id: i64, args: &[MbValue], globals: MbValue) -> Result<MbValue, ExecErr> {
    let (params, defaults, body) = INTERP_FUNCS
        .with(|m| {
            m.borrow()
                .get(&id)
                .map(|f| (f.params.clone(), f.defaults.clone(), f.body.clone()))
        })
        .ok_or_else(|| ExecErr::new("TypeError", "object is not callable"))?;
    if args.len() > params.len() {
        return Err(ExecErr::new(
            "TypeError",
            format!(
                "function takes {} arguments but {} were given",
                params.len(),
                args.len()
            ),
        ));
    }
    let mut frame: FxHashMap<String, MbValue> = FxHashMap::default();
    for (i, p) in params.iter().enumerate() {
        let v = if i < args.len() {
            args[i]
        } else if let Some(Some(d)) = defaults.get(i) {
            *d
        } else {
            return Err(ExecErr::new(
                "TypeError",
                format!("missing required argument: '{p}'"),
            ));
        };
        frame.insert(p.clone(), v);
    }
    let mut env = Env {
        frame: Some(&mut frame),
        globals,
    };
    for stmt in &body {
        match exec_stmt(stmt, &mut env)? {
            Flow::Return(v) => return Ok(v),
            Flow::Normal => {}
        }
    }
    Ok(MbValue::none())
}

fn exec_stmt(stmt: &Spanned<ast::Stmt>, env: &mut Env) -> Result<Flow, ExecErr> {
    match &stmt.node {
        ast::Stmt::Assign { target, value } => {
            let v = eval_expr(value, env)?;
            match &target.node {
                ast::Expr::Ident(name) => {
                    env.set(name, v);
                    Ok(Flow::Normal)
                }
                _ => Err(ExecErr::new(
                    "TypeError",
                    "unsupported assignment target in interactive exec",
                )),
            }
        }
        ast::Stmt::ExprStmt(e) => {
            eval_expr(e, env)?;
            Ok(Flow::Normal)
        }
        ast::Stmt::Return(value) => {
            let v = match value {
                Some(e) => eval_expr(e, env)?,
                None => MbValue::none(),
            };
            Ok(Flow::Return(v))
        }
        ast::Stmt::FnDef {
            name, params, body, ..
        } => {
            let mut pnames = Vec::new();
            let mut defaults = Vec::new();
            for p in params {
                pnames.push(p.name.clone());
                let d = match &p.default {
                    Some(e) => Some(eval_expr(e, env)?),
                    None => None,
                };
                defaults.push(d);
            }
            let id = NEXT_INTERP_ID.with(|c| {
                let v = c.get();
                c.set(v + 1);
                v
            });
            INTERP_FUNCS.with(|m| {
                m.borrow_mut().insert(
                    id,
                    InterpFunc {
                        params: pnames,
                        defaults,
                        body: body.clone(),
                    },
                );
            });
            let mut fields = FxHashMap::default();
            fields.insert("__interp_id__".to_string(), MbValue::from_int(id));
            fields.insert("__name__".to_string(), new_str(name));
            let obj = Box::new(MbObject {
                header: MbObjectHeader {
                    rc: std::sync::atomic::AtomicU32::new(1),
                    kind: ObjKind::Instance,
                },
                data: ObjData::Instance {
                    class_name: "function".to_string(),
                    fields: super::super::rc::MbRwLock::new(fields),
                },
            });
            env.set(name, MbValue::from_ptr(Box::into_raw(obj)));
            Ok(Flow::Normal)
        }
        ast::Stmt::Pass => Ok(Flow::Normal),
        _ => exec_if_like(stmt, env),
    }
}

/// `if` handling split out so exec_stmt stays readable.
fn exec_if_like(stmt: &Spanned<ast::Stmt>, env: &mut Env) -> Result<Flow, ExecErr> {
    if let ast::Stmt::If {
        condition,
        body,
        elif_clauses,
        else_body,
    } = &stmt.node
    {
        if truthy(eval_expr(condition, env)?) {
            return exec_block(body, env);
        }
        for (cond, clause_body) in elif_clauses {
            if truthy(eval_expr(cond, env)?) {
                return exec_block(clause_body, env);
            }
        }
        if let Some(else_body) = else_body {
            return exec_block(else_body, env);
        }
        return Ok(Flow::Normal);
    }
    Err(ExecErr::new(
        "TypeError",
        "unsupported statement in interactive exec",
    ))
}

fn exec_block(body: &[Spanned<ast::Stmt>], env: &mut Env) -> Result<Flow, ExecErr> {
    for stmt in body {
        match exec_stmt(stmt, env)? {
            Flow::Return(v) => return Ok(Flow::Return(v)),
            Flow::Normal => {}
        }
    }
    Ok(Flow::Normal)
}

/// Execute parsed top-level statements against the shared locals dict.
/// Interactive ('single') semantics: a top-level expression statement whose
/// value is not None prints its repr to stdout (sys.displayhook).
fn exec_module(module: &ast::Module, globals: MbValue) -> Result<(), ExecErr> {
    let mut env = Env {
        frame: None,
        globals,
    };
    for stmt in &module.stmts {
        if let ast::Stmt::ExprStmt(e) = &stmt.node {
            let v = eval_expr(e, &mut env)?;
            if !v.is_none() {
                let repr = extract_str(super::super::builtins::mb_repr(v)).unwrap_or_default();
                let line = format!("{repr}\n");
                if !super::super::output::write_captured(&line) {
                    print!("{line}");
                }
            }
            continue;
        }
        exec_stmt(stmt, &mut env)?;
    }
    Ok(())
}

// ── Interpreter / Console object model ───────────────────────────────────

fn make_instance(class_name: &str, fields: FxHashMap<String, MbValue>) -> MbValue {
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: class_name.to_string(),
            fields: super::super::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

fn instance_field(inst: MbValue, name: &str) -> Option<MbValue> {
    inst.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(name).copied()
        } else {
            None
        }
    })
}

fn locals_dict(self_v: MbValue) -> MbValue {
    match instance_field(self_v, "locals") {
        Some(d) if !d.is_none() => d,
        _ => MbValue::from_ptr(MbObject::new_dict()),
    }
}

/// Method-call args arrive packed as a list (kwargs as a trailing dict, which
/// the code-module methods don't use). Unpack positionals.
fn method_args(args: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = args.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                return lock.read().unwrap().iter().copied().collect();
            }
        }
    }
    if args.is_none() {
        Vec::new()
    } else {
        vec![args]
    }
}

fn require_str(v: MbValue, what: &str) -> Result<String, MbValue> {
    extract_str(v).ok_or_else(|| raise("TypeError", &format!("{what} must be str")))
}

/// Core runsource logic shared by runsource / push / runcode.
/// Returns true = more input needed, false = complete (ran or
/// errored-and-reported).
fn runsource_impl(self_v: MbValue, source: &str, filename: &str) -> bool {
    match classify_source(source) {
        SourceState::Incomplete => true,
        SourceState::Error(msg) => {
            write_stderr(&format!(
                "  File \"{filename}\", line 1\nSyntaxError: {msg}\n"
            ));
            false
        }
        SourceState::Complete(module) => {
            let globals = locals_dict(self_v);
            if let Err(ExecErr(ty, msg)) = exec_module(&module, globals) {
                write_stderr(&format!(
                    "Traceback (most recent call last):\n  File \"{filename}\", line 1, in <module>\n{ty}: {msg}\n"
                ));
            }
            false
        }
    }
}

// ── Native methods ───────────────────────────────────────────────────────

unsafe extern "C" fn ii_runsource(self_v: MbValue, args: MbValue) -> MbValue {
    let items = method_args(args);
    let source = match require_str(
        items.first().copied().unwrap_or_else(MbValue::none),
        "source",
    ) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let filename = items
        .get(1)
        .copied()
        .and_then(extract_str)
        .unwrap_or_else(|| "<input>".to_string());
    MbValue::from_bool(runsource_impl(self_v, &source, &filename))
}

unsafe extern "C" fn ii_runcode(self_v: MbValue, args: MbValue) -> MbValue {
    let items = method_args(args);
    let code = items.first().copied().unwrap_or_else(MbValue::none);
    let is_code = code
        .as_ptr()
        .map(|p| matches!(&(*p).data, ObjData::Instance { class_name, .. } if class_name == "code"))
        .unwrap_or(false);
    if !is_code {
        return raise("TypeError", "runcode() argument must be a code object");
    }
    if let Some(src) = instance_field(code, "_source").and_then(extract_str) {
        let globals = locals_dict(self_v);
        if let SourceState::Complete(module) = classify_source(&src) {
            if let Err(ExecErr(ty, msg)) = exec_module(&module, globals) {
                write_stderr(&format!(
                    "Traceback (most recent call last):\n  File \"<input>\", line 1, in <module>\n{ty}: {msg}\n"
                ));
            }
        }
    }
    MbValue::none()
}

unsafe extern "C" fn ii_write(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = method_args(args);
    let data = match require_str(items.first().copied().unwrap_or_else(MbValue::none), "data") {
        Ok(s) => s,
        Err(e) => return e,
    };
    write_stderr(&data);
    MbValue::none()
}

unsafe extern "C" fn ii_showsyntaxerror(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = method_args(args);
    if let Some(filename) = items.first().copied() {
        // A trailing kwargs dict (e.g. source=...) is tolerated.
        let is_kwargs_dict = filename
            .as_ptr()
            .map(|p| matches!(&(*p).data, ObjData::Dict(_)))
            .unwrap_or(false);
        if !filename.is_none() && !is_kwargs_dict && extract_str(filename).is_none() {
            return raise("TypeError", "filename must be str or None");
        }
    }
    write_stderr("SyntaxError: invalid syntax\n");
    MbValue::none()
}

unsafe extern "C" fn ii_showtraceback(_self_v: MbValue, _args: MbValue) -> MbValue {
    write_stderr("Traceback (most recent call last):\n");
    MbValue::none()
}

unsafe extern "C" fn ic_push(self_v: MbValue, args: MbValue) -> MbValue {
    let items = method_args(args);
    let line = match require_str(items.first().copied().unwrap_or_else(MbValue::none), "line") {
        Ok(s) => s,
        Err(e) => return e,
    };
    // Append to the buffer and try the joined source.
    let buffer = instance_field(self_v, "buffer")
        .unwrap_or_else(|| MbValue::from_ptr(MbObject::new_list(vec![])));
    let joined = {
        if let Some(ptr) = buffer.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut buf = lock.write().unwrap();
                buf.push(new_str(&line));
                buf.iter()
                    .map(|v| extract_str(*v).unwrap_or_default())
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                line.clone()
            }
        } else {
            line.clone()
        }
    };
    let more = runsource_impl(self_v, &joined, "<input>");
    if !more {
        if let Some(ptr) = buffer.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.write().unwrap().clear();
            }
        }
    }
    MbValue::from_bool(more)
}

unsafe extern "C" fn ic_resetbuffer(self_v: MbValue, _args: MbValue) -> MbValue {
    if let Some(buffer) = instance_field(self_v, "buffer") {
        if let Some(ptr) = buffer.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.write().unwrap().clear();
            }
        }
    }
    MbValue::none()
}

unsafe extern "C" fn ic_raw_input(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = method_args(args);
    if let Some(prompt) = items.first().copied() {
        if !prompt.is_none() && extract_str(prompt).is_none() {
            return raise("TypeError", "prompt must be str");
        }
        if let Some(p) = extract_str(prompt) {
            write_stderr(&p);
        }
    }
    let mut line = String::new();
    match std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut line) {
        Ok(0) => raise("EOFError", "EOF when reading a line"),
        Ok(_) => {
            if line.ends_with('\n') {
                line.pop();
            }
            new_str(&line)
        }
        Err(_) => raise("EOFError", "EOF when reading a line"),
    }
}

fn banner_type_ok(banner: MbValue) -> bool {
    banner.is_none() || extract_str(banner).is_some()
}

unsafe extern "C" fn ic_interact(self_v: MbValue, args: MbValue) -> MbValue {
    let items = method_args(args);
    if let Some(banner) = items.first().copied() {
        let is_kwargs_dict = banner
            .as_ptr()
            .map(|p| matches!(&(*p).data, ObjData::Dict(_)))
            .unwrap_or(false);
        if !is_kwargs_dict && !banner_type_ok(banner) {
            return raise("TypeError", "banner must be str or None");
        }
        if let Some(b) = extract_str(banner) {
            write_stderr(&format!("{b}\n"));
        }
    }
    // Drive the console from stdin until EOF.
    loop {
        let mut line = String::new();
        match std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                if line.ends_with('\n') {
                    line.pop();
                }
                let push_args = MbValue::from_ptr(MbObject::new_list(vec![new_str(&line)]));
                ic_push(self_v, push_args);
            }
        }
    }
    MbValue::none()
}

// ── Constructors / module functions ──────────────────────────────────────

unsafe extern "C" fn dispatch_interactive_interpreter(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let locals = if nargs >= 1 {
        let arg0 = *args_ptr;
        if arg0.is_none() {
            MbValue::from_ptr(MbObject::new_dict())
        } else {
            super::super::rc::retain_if_ptr(arg0);
            arg0
        }
    } else {
        MbValue::from_ptr(MbObject::new_dict())
    };
    let mut fields = FxHashMap::default();
    fields.insert("locals".to_string(), locals);
    make_instance("InteractiveInterpreter", fields)
}

unsafe extern "C" fn dispatch_interactive_console(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let locals = if nargs >= 1 {
        let arg0 = *args_ptr;
        if arg0.is_none() {
            MbValue::from_ptr(MbObject::new_dict())
        } else {
            super::super::rc::retain_if_ptr(arg0);
            arg0
        }
    } else {
        MbValue::from_ptr(MbObject::new_dict())
    };
    let mut fields = FxHashMap::default();
    fields.insert("locals".to_string(), locals);
    fields.insert(
        "buffer".to_string(),
        MbValue::from_ptr(MbObject::new_list(vec![])),
    );
    make_instance("InteractiveConsole", fields)
}

unsafe extern "C" fn dispatch_interact(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args: &[MbValue] = if nargs == 0 {
        &[]
    } else {
        std::slice::from_raw_parts(args_ptr, nargs)
    };
    if let Some(&banner) = args.first() {
        let is_kwargs_dict = banner
            .as_ptr()
            .map(|p| matches!(&(*p).data, ObjData::Dict(_)))
            .unwrap_or(false);
        if !is_kwargs_dict && !banner_type_ok(banner) {
            return raise("TypeError", "banner must be str or None");
        }
    }
    let console = dispatch_interactive_console(std::ptr::null(), 0);
    let interact_args = MbValue::from_ptr(MbObject::new_list(args.to_vec()));
    ic_interact(console, interact_args)
}

unsafe extern "C" fn dispatch_compile_command(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return raise(
            "TypeError",
            "compile_command() missing required argument: 'source'",
        );
    }
    let args = std::slice::from_raw_parts(args_ptr, nargs);
    let source = match require_str(args[0], "source") {
        Ok(s) => s,
        Err(e) => return e,
    };
    let filename = args
        .get(1)
        .copied()
        .and_then(extract_str)
        .unwrap_or_else(|| "<input>".to_string());
    match classify_source(&source) {
        SourceState::Incomplete => MbValue::none(),
        SourceState::Error(msg) => raise("SyntaxError", &msg),
        SourceState::Complete(_) => {
            super::super::class::make_module_code_object(&filename, &source)
        }
    }
}

// ── Registration ─────────────────────────────────────────────────────────

fn register_code_classes() {
    use std::collections::HashMap as Map;
    let var = |addr: usize| {
        super::super::module::register_variadic_func(addr as u64);
        MbValue::from_func(addr)
    };
    let interp_methods: Vec<(&str, usize)> = vec![
        ("runsource", ii_runsource as *const () as usize),
        ("runcode", ii_runcode as *const () as usize),
        ("write", ii_write as *const () as usize),
        ("showsyntaxerror", ii_showsyntaxerror as *const () as usize),
        ("showtraceback", ii_showtraceback as *const () as usize),
    ];
    let console_methods: Vec<(&str, usize)> = vec![
        ("push", ic_push as *const () as usize),
        ("resetbuffer", ic_resetbuffer as *const () as usize),
        ("raw_input", ic_raw_input as *const () as usize),
        ("interact", ic_interact as *const () as usize),
    ];
    let mut mi: Map<String, MbValue> = Map::new();
    for (name, addr) in &interp_methods {
        mi.insert((*name).to_string(), var(*addr));
    }
    super::super::class::mb_class_register("InteractiveInterpreter", vec![], mi);
    let mut mc: Map<String, MbValue> = Map::new();
    for (name, addr) in interp_methods.iter().chain(console_methods.iter()) {
        mc.insert((*name).to_string(), var(*addr));
    }
    super::super::class::mb_class_register(
        "InteractiveConsole",
        vec!["InteractiveInterpreter".to_string()],
        mc,
    );
}

/// Register the code module.
pub fn register() {
    register_code_classes();

    let mut attrs = HashMap::new();

    let addr_ii = dispatch_interactive_interpreter as *const () as usize;
    let addr_ic = dispatch_interactive_console as *const () as usize;
    let addr_i = dispatch_interact as *const () as usize;
    let addr_cc = dispatch_compile_command as *const () as usize;

    attrs.insert("InteractiveInterpreter".into(), MbValue::from_func(addr_ii));
    attrs.insert("InteractiveConsole".into(), MbValue::from_func(addr_ic));
    attrs.insert("interact".into(), MbValue::from_func(addr_i));
    attrs.insert("compile_command".into(), MbValue::from_func(addr_cc));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for addr in [addr_ii, addr_ic, addr_i, addr_cc] {
            set.insert(addr as u64);
        }
    });
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        map.insert(addr_ii as u64, "InteractiveInterpreter".to_string());
        map.insert(addr_ic as u64, "InteractiveConsole".to_string());
    });

    super::register_module("code", attrs);
}
