/// HIR → MIR lowering (#278).
///
/// Converts HIR (tree-based, high-level) into MIR (SSA-based, CFG).
/// Each HirFunction becomes a MirBody with basic blocks, virtual registers,
/// and explicit control flow.
use crate::hir::*;
use crate::mir::*;
use crate::resolve::{SymbolId, SymbolTable, VariableClass};
use crate::types::{Ty, TypeContext, TypeId};
use std::collections::{HashMap, HashSet};

/// Decorator kind applied to a class method. Used during class registration
/// to wrap the raw function pointer in a property / classmethod / staticmethod
/// descriptor so that `mb_getattr` can dispatch correctly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodDecorKind {
    None,
    Property,
    ClassMethod,
    StaticMethod,
    CachedProperty,
}

type MethodEntry = (
    String,
    SymbolId,
    MethodDecorKind,
    Option<SymbolId>,
    Option<SymbolId>,
    Vec<&'static str>,
    // Generic runtime decorator to apply to the (already-wrapped) method
    // value, e.g. a cross-class `@Base.prop.setter`. None for plain or
    // descriptor-folded methods. (#82)
    Option<HirExpr>,
);

type PendingClassRegistration = (
    String,
    SymbolId,
    Vec<String>,
    Option<NamedTupleBaseSpec>,
    Vec<MethodEntry>,
    Vec<String>,
    Option<String>,
    Option<Vec<String>>,
    Vec<(String, HirExpr)>,
);

fn method_decorator_marker_attr_name(name: &str) -> Option<&'static str> {
    match name {
        "override" => Some("__override__"),
        "final" => Some("__final__"),
        _ => None,
    }
}

/// Mapping from Python builtin name to mb_* runtime extern name.
fn builtin_extern_map() -> HashMap<&'static str, &'static str> {
    [
        ("print", "mb_print"),
        ("len", "mb_len"),
        ("int", "mb_int"),
        ("float", "mb_float"),
        ("bool", "mb_bool"),
        ("str", "mb_str"),
        ("abs", "mb_abs"),
        ("type", "mb_type"),
        ("range", "mb_range"),
        ("slice", "mb_slice"),
        ("complex", "mb_complex"),
        ("breakpoint", "mb_breakpoint"),
        ("memoryview", "mb_memoryview"),
        ("__import__", "mb_dunder_import"),
        ("min", "mb_min"),
        ("max", "mb_max"),
        ("sum", "mb_sum"),
        ("sorted", "mb_sorted"),
        ("repr", "mb_repr"),
        ("hash", "mb_hash"),
        ("id", "mb_id"),
        ("input", "mb_input"),
        ("chr", "mb_chr"),
        ("ord", "mb_ord"),
        ("isinstance", "mb_isinstance"),
        ("issubclass", "mb_issubclass"),
        ("callable", "mb_callable"),
        ("hasattr", "mb_hasattr"),
        ("getattr", "mb_getattr"),
        ("setattr", "mb_setattr"),
        ("delattr", "mb_delattr"),
        ("iter", "mb_iter"),
        ("next", "mb_next_raise"),
        ("reversed", "mb_reversed"),
        ("enumerate", "mb_enumerate"),
        ("zip", "mb_zip"),
        ("map", "mb_map"),
        ("filter", "mb_filter"),
        ("any", "mb_any"),
        ("all", "mb_all"),
        ("hex", "mb_hex"),
        ("oct", "mb_oct"),
        ("bin", "mb_bin"),
        ("format", "mb_format"),
        ("vars", "mb_vars"),
        ("dir", "mb_dir"),
        // HANDWRITE-BEGIN gap="standardize:projects-mamba-src-lower-hir-to-mir-rs" tracker="standardize-gap-projects-mamba-src-lower-hir-to-mir-rs" reason="introspection-builtins (issue: enhancement-mamba-introspection-builtins-globals-locals-vars-dir)."
        // Wires `globals()` / `locals()` into the lowerer so the names resolve.
        // Runtime impl currently returns an empty dict; full module-namespace
        // tracking requires a sym_names runtime registry — separate follow-up.
        ("globals", "mb_globals"),
        ("locals", "mb_locals"),
        // HANDWRITE-END
        // #1565 — eval/exec/compile builtin name registration.
        ("eval", "mb_eval"),
        ("exec", "mb_exec"),
        ("compile", "mb_compile"),
        ("round", "mb_round"),
        ("pow", "mb_pow"),
        ("divmod", "mb_divmod"),
        ("super", "mb_super"),
        ("frozenset", "mb_frozenset_new"),
        ("list", "mb_list_from_iterable"),
        ("tuple", "mb_tuple_from_iterable"),
        ("set", "mb_set_from_iterable"),
        ("dict", "mb_dict_from_pairs"),
        ("bytes", "mb_bytes_new_checked"),
        ("bytearray", "mb_bytearray_new_checked"),
        ("ascii", "mb_ascii"),
        ("open", "mb_open"),
        // PEP 695 desugaring intrinsics (lower::pep695): runtime TypeVar /
        // TypeAliasType construction.
        ("__mb_pep695_typevar__", "mb_pep695_typevar"),
        ("__mb_pep695_type_alias__", "mb_pep695_type_alias"),
    ]
    .into_iter()
    .collect()
}

/// Extract a function's docstring: the string-literal value of a leading
/// `HirStmt::Expr` whose expression is `HirExpr::StrLit`. Returns None if the
/// body is empty or doesn't start with a bare string literal.
fn extract_leading_docstring(body: &[HirStmt]) -> Option<String> {
    match body.first()? {
        HirStmt::Expr {
            expr: HirExpr::StrLit(s, _),
            ..
        } => Some(s.clone()),
        _ => None,
    }
}

/// Collapse function-body redefinitions so each `SymbolId` is compiled once.
///
/// A module-scope `def f(...)` that is later redefined (`def f(...)` again)
/// rebinds the *same* resolver `SymbolId`, so both bodies arrive in
/// `bodies` with an identical `name.0`. Codegen names every body
/// `_mb_<name.0>` and exports it, so two same-symbol bodies trip Cranelift's
/// "Duplicate definition of identifier" at `define_function` time and the
/// whole module fails to compile.
///
/// Python binds the name to the *last* `def` that executes, and every call
/// site is resolved statically by `SymbolId` (`MirInst::Call { func }` /
/// `MirConst::FuncRef`), so keeping the last body per symbol matches the
/// dominant "last def wins" semantics. Relative order is preserved for the
/// kept bodies (notably `__main__`, whose `name.0` is `u32::MAX`).
///
/// This cannot regress an already-passing fixture: any module containing a
/// redefinition currently fails the duplicate-define check at codegen, so no
/// passing fixture reaches this path with a collision to collapse.
fn dedup_bodies_keep_last(bodies: Vec<crate::mir::MirBody>) -> Vec<crate::mir::MirBody> {
    let mut last_index: HashMap<u32, usize> = HashMap::new();
    for (i, b) in bodies.iter().enumerate() {
        last_index.insert(b.name.0, i);
    }
    bodies
        .into_iter()
        .enumerate()
        .filter(|(i, b)| last_index.get(&b.name.0) == Some(i))
        .map(|(_, b)| b)
        .collect()
}

/// Register the SymbolId of every body whose return type is `any`/`object` — a
/// guaranteed already-boxed MbValue returned in the integer register. The
/// dynamic-call `rebox` (mb_call1_val / mb_call0 / mb_call_spread) re-boxes raw
/// unboxed ints (int fast-path returns, which lack a NaN-prefix) into MbValues;
/// but a `float` MbValue also lacks the prefix, so an any-returning callee that
/// returns a float (e.g. `lambda v: v*2.0` used as a map/filter callback) would
/// be mis-boxed as a giant int. Marking these addresses lets `rebox` pass their
/// result through untouched. Int/Bool returns stay unregistered and keep the
/// raw→box behavior; Float returns use the F64/xmm0 ABI and are out of scope
/// here. Mirrors the VARIADIC_SYMBOL_IDS / register_variadic_symbol pattern.
fn register_boxed_return_bodies(bodies: &[crate::mir::MirBody], tcx: &TypeContext) {
    use crate::mir::{MirInst, Terminator};
    use crate::types::Ty;
    for b in bodies {
        // Classify by the type of the VALUE the body actually returns, NOT the
        // function's declared return_ty: a decorated `-> int` function can carry
        // return_ty=Any yet still return a raw unboxed int (its body's `a+b` is
        // Int-typed) — that MUST stay reboxed. We register skip-rebox only when
        // every return value is provably NOT a raw primitive (Int/Bool), i.e. a
        // float or an already-boxed MbValue, which rebox would otherwise mis-box.
        let mut vreg_ty: std::collections::HashMap<u32, TypeId> = std::collections::HashMap::new();
        let mut copy_src: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
        for (vr, ty) in &b.params {
            vreg_ty.insert(vr.0, *ty);
        }
        for blk in &b.blocks {
            for inst in &blk.stmts {
                match inst {
                    MirInst::BinOp { dest, ty, .. }
                    | MirInst::CheckedAdd { dest, ty, .. }
                    | MirInst::CheckedSub { dest, ty, .. }
                    | MirInst::CheckedMul { dest, ty, .. }
                    | MirInst::UnaryOp { dest, ty, .. }
                    | MirInst::LoadConst { dest, ty, .. }
                    | MirInst::GetAttr { dest, ty, .. }
                    | MirInst::GetItem { dest, ty, .. }
                    | MirInst::MakeList { dest, ty, .. }
                    | MirInst::MakeDict { dest, ty, .. }
                    | MirInst::MakeTuple { dest, ty, .. }
                    | MirInst::LoadGlobal { dest, ty, .. }
                    | MirInst::LoadCell { dest, ty, .. }
                    | MirInst::MakeCell { dest, ty, .. }
                    | MirInst::LoadCapture { dest, ty, .. } => {
                        vreg_ty.insert(dest.0, *ty);
                    }
                    MirInst::Call {
                        dest: Some(d), ty, ..
                    }
                    | MirInst::CallExtern {
                        dest: Some(d), ty, ..
                    } => {
                        vreg_ty.insert(d.0, *ty);
                    }
                    MirInst::Copy { dest, source } => {
                        copy_src.insert(dest.0, source.0);
                    }
                    _ => {}
                }
            }
        }
        let resolve = |mut v: u32| -> Option<TypeId> {
            for _ in 0..64 {
                if let Some(t) = vreg_ty.get(&v) {
                    return Some(*t);
                }
                match copy_src.get(&v) {
                    Some(&s) => v = s,
                    None => return None,
                }
            }
            None
        };
        let mut returns_value = false;
        let mut all_boxed = true;
        for blk in &b.blocks {
            if let Terminator::Return(Some(vr)) = &blk.terminator {
                returns_value = true;
                match resolve(vr.0).map(|t| tcx.get(t)) {
                    // Provably a raw primitive, or undeterminable → keep reboxing.
                    Some(Ty::Int) | Some(Ty::Bool) | None => {
                        all_boxed = false;
                        break;
                    }
                    _ => {}
                }
            }
        }
        if returns_value && all_boxed {
            crate::runtime::module::register_boxed_return_symbol(b.name.0);
        }
    }
}

/// Compute code-object introspection metadata for a function (CORE #3):
/// `(co_argcount, co_varnames)`. `co_argcount` is the number of positional
/// parameters, excluding any trailing `*args` / `**kwargs` (CPython counts
/// those in `co_varnames` but not `co_argcount`). `co_varnames` is the
/// parameter names in declaration order. `resolve` maps each param SymbolId to
/// its source name; unresolved params fall back to a positional placeholder so
/// the varnames length still matches the parameter count.
fn func_code_metadata(
    func: &HirFunction,
    resolve: impl Fn(SymbolId) -> Option<String>,
) -> (i64, Vec<String>) {
    let mut varnames: Vec<String> = func
        .params
        .iter()
        .enumerate()
        .map(|(i, (sym, _))| resolve(*sym).unwrap_or_else(|| format!("arg{i}")))
        .collect();
    // argcount excludes the trailing *args / **kwargs params (which are the
    // last one or two parameters when present).
    let mut excluded = 0usize;
    if func.has_kwargs {
        excluded += 1;
    }
    if func.has_star_args {
        excluded += 1;
    }
    let argcount = varnames.len().saturating_sub(excluded) as i64;
    // CPython's co_varnames appends body locals after the parameters, in
    // first-binding order. Collect Let/Assign/For/With binding targets.
    let mut seen: std::collections::HashSet<String> = varnames.iter().cloned().collect();
    collect_local_names(&func.body, &resolve, &mut varnames, &mut seen);
    (argcount, varnames)
}

/// Append the names bound by local statements (in first-binding order) to
/// `out`, skipping names already present. Mirrors CPython's locals section of
/// co_varnames: plain assignments, loop vars, with-targets, except-aliases.
fn collect_local_names(
    body: &[HirStmt],
    resolve: &impl Fn(SymbolId) -> Option<String>,
    out: &mut Vec<String>,
    seen: &mut std::collections::HashSet<String>,
) {
    fn push(
        sym: SymbolId,
        resolve: &impl Fn(SymbolId) -> Option<String>,
        out: &mut Vec<String>,
        seen: &mut std::collections::HashSet<String>,
    ) {
        if let Some(name) = resolve(sym) {
            if !name.is_empty() && seen.insert(name.clone()) {
                out.push(name);
            }
        }
    }
    fn lvalue(
        lv: &crate::hir::HirLValue,
        resolve: &impl Fn(SymbolId) -> Option<String>,
        out: &mut Vec<String>,
        seen: &mut std::collections::HashSet<String>,
    ) {
        match lv {
            crate::hir::HirLValue::Var(sym) => push(*sym, resolve, out, seen),
            crate::hir::HirLValue::Unpack { targets, .. } => {
                for t in targets {
                    lvalue(t, resolve, out, seen);
                }
            }
            _ => {}
        }
    }
    for stmt in body {
        match stmt {
            HirStmt::Let { target, .. } => push(*target, resolve, out, seen),
            HirStmt::Assign { target, .. } => lvalue(target, resolve, out, seen),
            HirStmt::For {
                var,
                body,
                else_body,
                ..
            } => {
                push(*var, resolve, out, seen);
                collect_local_names(body, resolve, out, seen);
                collect_local_names(else_body, resolve, out, seen);
            }
            HirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                collect_local_names(then_body, resolve, out, seen);
                collect_local_names(else_body, resolve, out, seen);
            }
            HirStmt::While {
                body, else_body, ..
            } => {
                collect_local_names(body, resolve, out, seen);
                collect_local_names(else_body, resolve, out, seen);
            }
            HirStmt::Try {
                body,
                handlers,
                else_body,
                finally_body,
                ..
            } => {
                collect_local_names(body, resolve, out, seen);
                for h in handlers {
                    collect_local_names(&h.body, resolve, out, seen);
                }
                collect_local_names(else_body, resolve, out, seen);
                collect_local_names(finally_body, resolve, out, seen);
            }
            HirStmt::With { items, body, .. } => {
                for (_, target) in items {
                    if let Some(sym) = target {
                        push(*sym, resolve, out, seen);
                    }
                }
                collect_local_names(body, resolve, out, seen);
            }
            _ => {}
        }
    }
}

/// Lower a complete HIR module to a MIR module.
pub fn lower_hir_to_mir(hir: &HirModule, tcx: &TypeContext) -> MirModule {
    let mut lowerer = HirToMir::new(tcx);
    // Populate sym_types for nested pattern capture unboxing (#827).
    lowerer.sym_types = hir.sym_types.clone();
    lowerer.sym_names = hir.sym_names.clone();
    lowerer.module_annotations = hir.module_annotations.clone();
    // Populate user_func_param_types so MirInst::Call sites can selectively box
    // primitive args destined for Any/object-typed parameters (#827 R8).
    // Also populate user_func_return_tys for iter(callable, sentinel) thunk generation.
    for func in &hir.functions {
        let param_types: Vec<TypeId> = func.params.iter().map(|(_, ty)| *ty).collect();
        lowerer
            .user_func_param_types
            .insert(func.name.0, param_types);
        lowerer
            .user_func_return_tys
            .insert(func.name.0, func.return_ty);
        lowerer
            .user_func_variadic_info
            .insert(func.name.0, (func.has_star_args, func.has_kwargs));
        if let Some(name) = hir.sym_names.get(&func.name) {
            lowerer.user_func_names.insert(func.name.0, name.clone());
        }
        if let Some(doc) = extract_leading_docstring(&func.body) {
            lowerer.user_func_docs.insert(func.name.0, doc);
        }
        let (argcount, varnames) = func_code_metadata(func, |sym| hir.sym_names.get(&sym).cloned());
        lowerer.user_func_argcounts.insert(func.name.0, argcount);
        lowerer.user_func_varnames.insert(func.name.0, varnames);
    }
    lowerer.user_func_sigs = hir.func_sigs.clone();
    for func in &hir.functions {
        if !func.decorators.is_empty() {
            lowerer
                .pending_decorators
                .push((func.name, func.decorators.clone()));
            lowerer.decorated_func_syms.insert(func.name.0);
            lowerer
                .decorated_func_return_tys
                .insert(func.name.0, func.return_ty);
        }
        let body = lowerer.lower_function(func);
        lowerer.bodies.push(body);
    }
    // Top-level statements go into an implicit __main__ function
    if !hir.top_level.is_empty() {
        let main_body = lowerer.lower_top_level(&hir.top_level);
        lowerer.bodies.push(main_body);
    }
    let bodies = dedup_bodies_keep_last(lowerer.bodies);
    register_boxed_return_bodies(&bodies, tcx);
    MirModule {
        bodies,
        externs: Vec::new(),
    }
}

/// Lower with symbol table for builtin resolution.
pub fn lower_hir_to_mir_with_symbols(
    hir: &HirModule,
    tcx: &TypeContext,
    symbols: &SymbolTable,
) -> MirModule {
    lower_hir_to_mir_with_symbols_src(hir, tcx, symbols, None)
}

/// Like `lower_hir_to_mir_with_symbols`, additionally threading the module
/// source `(filename, text)` so function metadata can include real source
/// locations (co_firstlineno / co_filename). `None` keeps the old behavior.
pub fn lower_hir_to_mir_with_symbols_src(
    hir: &HirModule,
    tcx: &TypeContext,
    symbols: &SymbolTable,
    src: Option<(&str, &str)>,
) -> MirModule {
    let user_funcs: HashSet<u32> = hir.functions.iter().map(|f| f.name.0).collect();
    let extern_map = builtin_extern_map();
    let mut builtin_syms: HashMap<u32, String> = HashMap::new();
    for (&py_name, &mb_name) in &extern_map {
        if let Some(sym_id) = symbols.lookup(py_name) {
            if !user_funcs.contains(&sym_id.0) {
                builtin_syms.insert(sym_id.0, mb_name.to_string());
            }
        }
    }
    // Collect class symbols (exception types etc.) so Var references emit string constants
    let exception_names = [
        "BaseException",
        "SystemExit",
        "KeyboardInterrupt",
        "GeneratorExit",
        "Exception",
        "StopIteration",
        "StopAsyncIteration",
        "ArithmeticError",
        "ZeroDivisionError",
        "OverflowError",
        "FloatingPointError",
        "LookupError",
        "IndexError",
        "KeyError",
        "OSError",
        "IOError",
        "FileNotFoundError",
        "PermissionError",
        "FileExistsError",
        "TypeError",
        "ValueError",
        "AttributeError",
        "NameError",
        "RuntimeError",
        "RecursionError",
        "NotImplementedError",
        "ImportError",
        "ModuleNotFoundError",
        "SyntaxError",
        "IndentationError",
        "UnicodeError",
        "UnicodeDecodeError",
        "UnicodeEncodeError",
        "AssertionError",
        "BufferError",
        "EOFError",
        "MemoryError",
        "ConnectionError",
        "ConnectionResetError",
        "ConnectionAbortedError",
        "ConnectionRefusedError",
        "BrokenPipeError",
        "IsADirectoryError",
        "NotADirectoryError",
        "InterruptedError",
        "ProcessLookupError",
        "ChildProcessError",
        "BlockingIOError",
        "ReferenceError",
        "TimeoutError",
        "ExceptionGroup",
        "BaseExceptionGroup",
        "Warning",
        "UserWarning",
        "DeprecationWarning",
        "PendingDeprecationWarning",
        "SyntaxWarning",
        "RuntimeWarning",
        "FutureWarning",
        "ImportWarning",
        "UnicodeWarning",
        "BytesWarning",
        "ResourceWarning",
        "EncodingWarning",
    ];
    let mut class_syms: HashMap<u32, String> = HashMap::new();
    for name in &exception_names {
        if let Some(sym_id) = symbols.lookup(name) {
            if !user_funcs.contains(&sym_id.0) {
                class_syms.insert(sym_id.0, name.to_string());
            }
        }
    }
    // Also add built-in type names so isinstance/issubclass args emit string constants.
    let type_names = [
        "int",
        "float",
        "str",
        "bool",
        "list",
        "dict",
        "set",
        "tuple",
        "bytes",
        "bytearray",
        "frozenset",
        "type",
        "object",
        // Descriptor types: `isinstance(x, property)` etc. compare against
        // the runtime's "__property__"/"__staticmethod__"/"__classmethod__"
        // wrapper instances via the alias arms in mb_isinstance.
        "property",
        "staticmethod",
        "classmethod",
    ];
    for name in &type_names {
        if let Some(sym_id) = symbols.lookup(name) {
            if !user_funcs.contains(&sym_id.0) {
                class_syms.insert(sym_id.0, name.to_string());
            }
        }
    }
    let mut lowerer = HirToMir::new_with_builtins(tcx, user_funcs, builtin_syms);
    lowerer.class_syms = class_syms;
    lowerer.symbol_table = Some(symbols);
    // Source-location metadata: line-start offsets for span→line conversion
    // plus per-def first lines (co_firstlineno / co_filename priming).
    if let Some((filename, source)) = src {
        let line_starts: Vec<u32> = std::iter::once(0)
            .chain(source.match_indices('\n').map(|(i, _)| (i + 1) as u32))
            .collect();
        let line_of = |offset: u32| -> u32 { line_starts.partition_point(|&s| s <= offset) as u32 };
        for func in &hir.functions {
            if func.span.end > 0 {
                lowerer
                    .user_func_lines
                    .insert(func.name.0, line_of(func.span.start));
            }
        }
        for cls in &hir.classes {
            for m in &cls.methods {
                if m.span.end > 0 {
                    lowerer
                        .user_func_lines
                        .insert(m.name.0, line_of(m.span.start));
                }
            }
        }
        lowerer.src_line_starts = Some(line_starts);
        lowerer.src_filename = Some(filename.to_string());
    }
    // Populate sym_types so emit_pattern_test can unbox nested capture bindings (#827).
    lowerer.sym_types = hir.sym_types.clone();
    lowerer.sym_names = hir.sym_names.clone();
    lowerer.module_annotations = hir.module_annotations.clone();
    // Populate user_func_param_types so MirInst::Call sites can selectively box
    // primitive args destined for Any/object-typed parameters (#827 R8).
    // Also populate user_func_return_tys for iter(callable, sentinel) thunk generation.
    for func in &hir.functions {
        let param_types: Vec<TypeId> = func.params.iter().map(|(_, ty)| *ty).collect();
        lowerer
            .user_func_param_types
            .insert(func.name.0, param_types);
        lowerer
            .user_func_return_tys
            .insert(func.name.0, func.return_ty);
        lowerer
            .user_func_variadic_info
            .insert(func.name.0, (func.has_star_args, func.has_kwargs));
    }
    for cls in &hir.classes {
        for method in &cls.methods {
            let param_types: Vec<TypeId> = method.params.iter().map(|(_, ty)| *ty).collect();
            lowerer
                .user_func_param_types
                .insert(method.name.0, param_types);
            lowerer
                .user_func_variadic_info
                .insert(method.name.0, (method.has_star_args, method.has_kwargs));
        }
    }

    // Build a reverse lookup from SymbolId → name using the symbol table.
    // This is more reliable than hir.sym_names which only covers local names.
    let sym_name_lookup = |sym: SymbolId| -> Option<String> {
        hir.sym_names.get(&sym).cloned().or_else(|| {
            // Synthetic symbol IDs (>= 1_000_000) are pattern-binding locals allocated
            // in ast_to_hir. They are never in the symbol table, so calling
            // symbols.get_symbol would panic with "index out of bounds" (#827).
            if sym.0 >= 1_000_000 {
                return None;
            }
            // Try symbol table (always valid for symbols resolved by checker)
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                symbols.get_symbol(sym).name.clone()
            }))
            .ok()
        })
    };

    // Pre-pass: register every class name up front so method bodies that
    // reference a later-defined class (e.g. `A.__iter__` returning `B()`
    // where `B` is declared below `A`) resolve the callee as a class
    // constructor at lowering time, not as an unresolved Var.
    for cls in &hir.classes {
        let class_name =
            sym_name_lookup(cls.name).unwrap_or_else(|| format!("class_{}", cls.name.0));
        lowerer.class_syms.insert(cls.name.0, class_name);
        lowerer.user_class_syms.insert(cls.name.0);
    }

    // Capture user-defined function names so the FUNC_NAMES registry can be
    // primed at module-init (top-level `def`s never flow through
    // FuncDefPlaceholder, which would otherwise register them).
    for func in &hir.functions {
        if let Some(name) = sym_name_lookup(func.name) {
            lowerer.user_func_names.insert(func.name.0, name);
        }
        if let Some(doc) = extract_leading_docstring(&func.body) {
            lowerer.user_func_docs.insert(func.name.0, doc);
        }
        let (argcount, varnames) = func_code_metadata(func, &sym_name_lookup);
        lowerer.user_func_argcounts.insert(func.name.0, argcount);
        lowerer.user_func_varnames.insert(func.name.0, varnames);
    }
    // Methods need the same __code__ introspection priming so
    // `Cls.m.__code__.co_argcount` / `.co_varnames` resolve and
    // mb_func_is_registered(Cls.m) is true. Keyed by FuncRef(method_sym) — the
    // value the class table stores for an undecorated method — so it matches the
    // value `getattr(Cls, "m")` returns. Introspection maps only; call lowering
    // is unaffected.
    for cls in &hir.classes {
        for func in &cls.methods {
            if let Some(name) = sym_name_lookup(func.name) {
                lowerer.user_func_names.insert(func.name.0, name);
            }
            let (argcount, varnames) = func_code_metadata(func, &sym_name_lookup);
            lowerer.user_func_argcounts.insert(func.name.0, argcount);
            lowerer.user_func_varnames.insert(func.name.0, varnames);
        }
    }
    lowerer.user_func_sigs = hir.func_sigs.clone();

    // Process user-defined classes: register class names and compile methods
    for cls in &hir.classes {
        let class_name =
            sym_name_lookup(cls.name).unwrap_or_else(|| format!("class_{}", cls.name.0));

        // Introspection docs: class-body docstring + per-method docstrings
        // (inspect.getdoc / __doc__). Method docs ride the existing
        // FUNC_DOCS priming loop in lower_top_level.
        if let Some(doc) = &cls.doc {
            lowerer
                .pending_class_docs
                .push((class_name.clone(), doc.clone()));
        }
        for m in &cls.methods {
            if let Some(doc) = extract_leading_docstring(&m.body) {
                lowerer.user_func_docs.insert(m.name.0, doc);
            }
        }

        // Resolve all base names for multiple inheritance (P1 OOP conformance).
        let all_base_names: Vec<String> = if !cls.all_bases.is_empty() {
            cls.all_bases
                .iter()
                .filter_map(|b| sym_name_lookup(*b))
                .collect()
        } else {
            cls.base
                .and_then(|b| sym_name_lookup(b))
                .into_iter()
                .collect()
        };
        // Detect method decorator kinds: property / classmethod / staticmethod.
        // The runtime wraps these via mb_property_new / mb_classmethod_new /
        // mb_staticmethod_new so descriptor dispatch works through mb_getattr.
        //
        // Also handle `@<name>.setter` / `.getter` / `.deleter` decorators: these
        // re-define a method with the same name as an existing @property, and
        // should be combined with the existing property (not registered as a
        // separate entry). We collapse them here so the per-property entry has
        // an optional setter/deleter symbol.
        let methods: Vec<MethodEntry> = {
            // Per-method: (name, sym, base_kind, is_setter_for, is_deleter_for, marker_attrs)
            struct Raw {
                sym: SymbolId,
                kind: MethodDecorKind,
                setter_target: Option<String>,
                deleter_target: Option<String>,
                marker_attrs: Vec<&'static str>,
                // A `@<expr>.setter/.deleter/.getter` whose `<expr>` is NOT a
                // bare sibling-property name (e.g. cross-class `@Base.x.setter`).
                // This is a generic runtime decorator, applied to the method
                // value rather than folded into an in-class property. (#82)
                generic_decorator: Option<HirExpr>,
            }
            impl Raw {
                fn new(sym: SymbolId, marker_attrs: Vec<&'static str>) -> Self {
                    Self {
                        sym,
                        kind: MethodDecorKind::None,
                        setter_target: None,
                        deleter_target: None,
                        marker_attrs,
                        generic_decorator: None,
                    }
                }
            }
            // Collect raw per-method info in declaration order.
            let mut raw: Vec<(String, Raw)> = Vec::new();
            for m in &cls.methods {
                let name = sym_name_lookup(m.name)
                    .unwrap_or_else(|| format!("method_{}", m.name.0));
                let marker_attrs = m.decorators
                    .iter()
                    .filter_map(|dec| match dec {
                        HirExpr::Var(dec_sym, _) => sym_name_lookup(*dec_sym)
                            .and_then(|name| method_decorator_marker_attr_name(&name)),
                        HirExpr::Attr { attr, .. } => {
                            method_decorator_marker_attr_name(attr)
                        }
                        _ => None,
                    })
                    .collect();
                let mut r = Raw::new(m.name, marker_attrs);
                for dec in &m.decorators {
                    match dec {
                        HirExpr::Var(dec_sym, _) => {
                            if let Some(dec_name) = sym_name_lookup(*dec_sym) {
                                match dec_name.as_str() {
                                    "property" => {
                                        r.kind = MethodDecorKind::Property;
                                    }
                                    "classmethod" => {
                                        r.kind = MethodDecorKind::ClassMethod;
                                    }
                                    "staticmethod" => {
                                        r.kind = MethodDecorKind::StaticMethod;
                                    }
                                    "cached_property" => {
                                        r.kind = MethodDecorKind::CachedProperty;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        HirExpr::Attr { object, attr, .. } => {
                            if let HirExpr::Var(obj_sym, _) = object.as_ref() {
                                let obj_name = sym_name_lookup(*obj_sym);
                                // `@enum.property` — the enum module's property
                                // descriptor behaves like the builtin property for
                                // member attribute access. Treat it as Property.
                                if attr == "property"
                                    && obj_name.as_deref() == Some("enum")
                                {
                                    r.kind = MethodDecorKind::Property;
                                } else if attr == "cached_property"
                                    && obj_name.as_deref() == Some("functools")
                                {
                                    // `@functools.cached_property` — same descriptor
                                    // as the bare `cached_property` import.
                                    r.kind = MethodDecorKind::CachedProperty;
                                } else if let Some(prop_name) = obj_name {
                                    // `@<name>.setter` / `.deleter` — combine with
                                    // the existing property of that name.
                                    if attr == "setter" {
                                        r.setter_target = Some(prop_name);
                                    } else if attr == "deleter" {
                                        r.deleter_target = Some(prop_name);
                                    }
                                }
                            } else if matches!(attr.as_str(), "setter" | "deleter" | "getter") {
                                // `@<expr>.setter/.deleter/.getter` where `<expr>`
                                // is not a bare sibling-property Var — e.g. the
                                // cross-class `@Base.x.setter` form. CPython
                                // evaluates the bound descriptor method and calls
                                // it on the function, yielding a NEW property
                                // sharing the base's accessors. Apply it as a
                                // generic runtime decorator. (#82)
                                r.generic_decorator = Some(dec.clone());
                            }
                        }
                        _ => {}
                    }
                }
                raw.push((name, r));
            }
            // Second pass: fold setters/deleters into their target property.
            let mut out: Vec<MethodEntry> = Vec::new();
            for (name, r) in &raw {
                if (r.setter_target.is_some() || r.deleter_target.is_some())
                    && r.generic_decorator.is_none()
                {
                    // Skip — the target property entry will reference this sym.
                    // (A generic cross-class decorator is emitted on its own.)
                    continue;
                }
                let mut setter_sym = None;
                let mut deleter_sym = None;
                for (_, other) in &raw {
                    if other.generic_decorator.is_some() {
                        continue;
                    }
                    if other.setter_target.as_deref() == Some(name.as_str()) {
                        setter_sym = Some(other.sym);
                    }
                    if other.deleter_target.as_deref() == Some(name.as_str()) {
                        deleter_sym = Some(other.sym);
                    }
                }
                out.push((
                    name.clone(),
                    r.sym,
                    r.kind,
                    setter_sym,
                    deleter_sym,
                    r.marker_attrs.clone(),
                    r.generic_decorator.clone(),
                ));
            }
            out
        };
        // Determine __match_args__ for PEP 634 positional class patterns (#827).
        // Explicit `__match_args__ = (...)` in the class body takes priority;
        // otherwise synthesize from __init__ params (skip 'self').
        let match_args: Vec<String> = if let Some(ref explicit) = cls.explicit_match_args {
            explicit.clone()
        } else {
            cls.methods
                .iter()
                .find(|m| sym_name_lookup(m.name).as_deref() == Some("__init__"))
                .map(|init| {
                    init.params
                        .iter()
                        .skip(1) // skip 'self'
                        .filter_map(|(sym, _)| sym_name_lookup(*sym))
                        .collect()
                })
                .unwrap_or_default()
        };
        // #82: a class with a cross-class chained property decorator
        // (`@Base.x.setter`) must register at its textual ClassDefPlaceholder
        // (not eagerly), so the base class's property is already set when the
        // decorator reads it. Mark it so the eager `None` registration pass
        // skips it and the placeholder's `Some(sym)` pass handles it.
        if methods.iter().any(|m| m.6.is_some()) {
            lowerer.classes_needing_textual_registration.insert(cls.name.0);
        }
        lowerer.pending_classes.push((
            class_name.clone(),
            cls.name,
            all_base_names,
            cls.namedtuple_base.clone(),
            methods,
            match_args,
            cls.metaclass.clone(),
            cls.slots.clone(),
            cls.class_kwargs.clone(),
        ));
        if !cls.runtime_base_exprs.is_empty() {
            lowerer.pending_runtime_class_bases.push((
                class_name.clone(),
                cls.name,
                cls.runtime_base_exprs.clone(),
            ));
        }
        // P2-R3: Store class-level attribute assignments for emission at the
        // class's ClassDefPlaceholder (its textual position in the module).
        for (attr_name, val_expr) in &cls.class_attr_assigns {
            lowerer.pending_class_attrs.push((
                class_name.clone(),
                cls.name,
                attr_name.clone(),
                val_expr.clone(),
            ));
        }
        // Store class decorators for application after registration.
        if !cls.decorators.is_empty() {
            lowerer.pending_class_decorators.push((
                class_name.clone(),
                cls.name,
                cls.decorators.clone(),
            ));
        }
        // PEP 557: stash ordered dataclass field facts; emitted at the
        // ClassDefPlaceholder right before the decorator call runs.
        if !cls.dataclass_fields.is_empty() {
            lowerer.pending_dataclass_fields.push((
                class_name.clone(),
                cls.name,
                cls.dataclass_fields.clone(),
            ));
        }

        // abc: collect names of methods decorated with `@abc.abstractmethod`
        // (or `abstractproperty`/`abstractclassmethod`/`abstractstaticmethod`),
        // whether referenced as `abc.abstractmethod` (Attr) or imported bare
        // (`from abc import abstractmethod` → Var).
        let is_abstract_decor_name = |n: &str| {
            matches!(
                n,
                "abstractmethod"
                    | "abstractproperty"
                    | "abstractclassmethod"
                    | "abstractstaticmethod"
            )
        };
        let mut abstract_method_names: Vec<String> = Vec::new();
        for m in &cls.methods {
            let is_abstract = m.decorators.iter().any(|dec| match dec {
                HirExpr::Attr { attr, .. } => is_abstract_decor_name(attr),
                HirExpr::Var(sym, _) => sym_name_lookup(*sym)
                    .as_deref()
                    .map_or(false, is_abstract_decor_name),
                _ => false,
            });
            if is_abstract {
                if let Some(mname) = sym_name_lookup(m.name) {
                    abstract_method_names.push(mname);
                }
            }
        }
        if !abstract_method_names.is_empty() {
            lowerer
                .pending_abstract_methods
                .push((class_name.clone(), abstract_method_names));
        }

        // Compile each method as a separate function
        let self_sym = cls
            .methods
            .first()
            .and_then(|m| m.params.first().map(|(s, _)| *s));
        for method in &cls.methods {
            let method_self_sym = method.params.first().map(|(s, _)| *s).or(self_sym);
            if let Some(ss) = method_self_sym {
                lowerer.current_class_ctx = Some((class_name.clone(), ss));
            }
            // R4 P1: Mark class methods so return values are NaN-boxed for dynamic dispatch.
            lowerer.is_class_method = true;
            let body = lowerer.lower_function(method);
            lowerer.bodies.push(body);
            lowerer.current_class_ctx = None;
            lowerer.is_class_method = false;
        }
    }

    for func in &hir.functions {
        if !func.decorators.is_empty() {
            lowerer
                .pending_decorators
                .push((func.name, func.decorators.clone()));
            lowerer.decorated_func_syms.insert(func.name.0);
            lowerer
                .decorated_func_return_tys
                .insert(func.name.0, func.return_ty);
        }
        let body = lowerer.lower_function(func);
        lowerer.bodies.push(body);
    }
    // Always emit a __main__ body when there are classes or top_level stmts
    if !hir.top_level.is_empty() || !hir.classes.is_empty() {
        let main_body = lowerer.lower_top_level(&hir.top_level);
        lowerer.bodies.push(main_body);
    }
    let bodies = dedup_bodies_keep_last(lowerer.bodies);
    register_boxed_return_bodies(&bodies, tcx);
    MirModule {
        bodies,
        externs: Vec::new(),
    }
}

/// REPL-aware lowering: includes accumulated functions from previous
/// iterations, restores globals, saves all top-level variables,
/// and returns the last expression value for echo.
/// REPL lowering result: (MirModule, new_globals, has_expression_echo).
pub fn lower_hir_to_mir_repl(
    hir: &HirModule,
    tcx: &TypeContext,
    prev_globals: &[String],
    extra_functions: &[HirFunction],
) -> (MirModule, Vec<String>, bool) {
    let mut lowerer = HirToMir::new(tcx);
    // Populate sym_types for nested pattern capture unboxing (#827).
    lowerer.sym_types = hir.sym_types.clone();
    lowerer.sym_names = hir.sym_names.clone();
    // Populate user_funcs so call-site dispatch correctly routes accumulated and current
    // session functions through MirInst::Call rather than dynamic mb_call* dispatch.
    for func in extra_functions {
        lowerer.user_funcs.insert(func.name.0);
    }
    for func in &hir.functions {
        lowerer.user_funcs.insert(func.name.0);
    }
    // Populate user_func_param_types so MirInst::Call sites can selectively box
    // primitive args destined for Any/object-typed parameters (#827 R8).
    for func in extra_functions {
        let param_types: Vec<TypeId> = func.params.iter().map(|(_, ty)| *ty).collect();
        lowerer
            .user_func_param_types
            .insert(func.name.0, param_types);
        lowerer
            .user_func_variadic_info
            .insert(func.name.0, (func.has_star_args, func.has_kwargs));
    }
    for func in &hir.functions {
        let param_types: Vec<TypeId> = func.params.iter().map(|(_, ty)| *ty).collect();
        lowerer
            .user_func_param_types
            .insert(func.name.0, param_types);
        lowerer
            .user_func_variadic_info
            .insert(func.name.0, (func.has_star_args, func.has_kwargs));
    }
    // Compile accumulated functions from previous iterations
    for func in extra_functions {
        let body = lowerer.lower_function(func);
        lowerer.bodies.push(body);
    }
    // Compile functions from current iteration
    for func in &hir.functions {
        let body = lowerer.lower_function(func);
        lowerer.bodies.push(body);
    }
    // Always emit a __main__ body (even if top_level is empty)
    let (body, new_globals, has_echo) =
        lowerer.lower_top_level_repl(&hir.top_level, &hir.sym_names, &hir.sym_types, prev_globals);
    lowerer.bodies.push(body);
    let bodies = dedup_bodies_keep_last(lowerer.bodies);
    register_boxed_return_bodies(&bodies, tcx);
    let mir = MirModule {
        bodies,
        externs: Vec::new(),
    };
    (mir, new_globals, has_echo)
}

struct HirToMir<'a> {
    tcx: &'a TypeContext,
    bodies: Vec<MirBody>,
    next_vreg: u32,
    next_block: u32,
    blocks: Vec<BasicBlock>,
    current_stmts: Vec<MirInst>,
    sym_to_vreg: HashMap<SymbolId, VReg>,
    /// Block IDs for break/continue targets
    loop_exit: Option<BlockId>,
    loop_header: Option<BlockId>,
    /// The block ID for the currently open block.
    current_block_id: Option<BlockId>,
    /// VReg holding coroutine handle for async functions.
    async_coro_vreg: Option<VReg>,
    /// User-defined function SymbolIds (to distinguish from builtins).
    user_funcs: HashSet<u32>,
    /// SymbolId.0 → mb_* extern name for builtin calls.
    builtin_syms: HashMap<u32, String>,
    /// SymbolId.0 → class name for exception types and user classes.
    class_syms: HashMap<u32, String>,
    /// VReg of the caught exception inside an except handler body (for implicit chaining).
    active_except_vreg: Option<VReg>,
    /// Classes to register at the start of top-level code.
    /// (class_name, class_symbol_id, all_base_names, namedtuple_base, methods, match_args, metaclass, slots, class_kwargs)
    pending_classes: Vec<PendingClassRegistration>,
    /// SymbolId.0 of classes whose registration must be emitted at their
    /// textual ClassDefPlaceholder rather than eagerly, because a method
    /// carries a cross-class chained property decorator (`@Base.x.setter`)
    /// that reads a base property only set at the base's textual position. (#82)
    classes_needing_textual_registration: HashSet<u32>,
    /// Class base expressions that must be evaluated at the class statement's
    /// textual runtime position, e.g. `class Derived(base):` inside a loop.
    pending_runtime_class_bases: Vec<(String, SymbolId, Vec<HirExpr>)>,
    /// P2-R3: Class-level attribute assignments to emit after class registration.
    /// (class_name, class_symbol_id, attr_name, value_expr)
    /// Emitted at the class's ClassDefPlaceholder position (textual order) so
    /// initializer expressions like `X = enum.auto()` see imports/bindings
    /// established by preceding statements (#1686 motivation).
    pending_class_attrs: Vec<(String, SymbolId, String, HirExpr)>,
    /// abc: per-class names of methods decorated `@abc.abstractmethod` (and the
    /// abstract{property,classmethod,staticmethod} variants). Emitted after
    /// `mb_class_define_multi` so the runtime can compute `__abstractmethods__`
    /// and block instantiation of still-abstract classes.
    pending_abstract_methods: Vec<(String, Vec<String>)>,
    /// Class decorator applications: (class_name, class_symbol_id, decorators).
    /// Applied after class registration + class attrs in lower_top_level.
    pending_class_decorators: Vec<(String, SymbolId, Vec<HirExpr>)>,
    /// PEP 557: per-class ordered dataclass field facts
    /// (class_name, class_symbol_id, [(field_name, annotation_repr, default)]).
    /// Emitted at the ClassDefPlaceholder position immediately BEFORE the
    /// decorator call so `@dataclass` sees recorded facts (and default exprs
    /// can reference imports bound above the class, mirroring #1686).
    pending_dataclass_fields: Vec<(String, SymbolId, Vec<(String, String, Option<HirExpr>)>)>,
    /// SymbolId.0 set for user-defined classes (need instance-based raise).
    user_class_syms: HashSet<u32>,
    /// Current class context for method lowering (class_name, self_sym).
    /// Set when lowering class methods so super() can be resolved.
    current_class_ctx: Option<(String, SymbolId)>,
    /// True when lowering a generator body function.
    /// Causes bare `return` to emit NaN-boxed None and boxes return values.
    is_gen_body: bool,
    /// Stack of (handler_block, finally_block) for active try blocks.
    /// Used by generator yield to emit post-yield exception checks that
    /// jump to the appropriate handler when throw()/close() injects an exception.
    try_handler_stack: Vec<(BlockId, BlockId)>,
    /// Stack of active `with` block context manager vregs, in depth order.
    /// Used when `raise` terminates inside a `with`: each pending context
    /// manager's __exit__ must be called before the function returns.
    with_ctx_stack: Vec<VReg>,
    /// Stack of active `with` blocks' exit sequences: (exit_block, try_depth).
    /// When a method/function call inside a `with` body raises, control must
    /// transfer to that `with`'s `__exit__` sequence (so suppression /
    /// re-raise runs) instead of returning straight out of the function. The
    /// recorded `try_depth` is `try_handler_stack.len()` when the `with` was
    /// entered; if a `try` was pushed *after* this `with` (current depth is
    /// greater), that inner `try` handles the exception first, so this frame
    /// is skipped. The innermost frame whose `try_depth == current depth` is
    /// the active `with` exit target.
    with_exit_stack: Vec<(BlockId, usize)>,
    /// Stack of finally bodies for active try blocks (parallel to try_handler_stack).
    /// Used to inline finally code before early exits (return/break/continue).
    finally_body_stack: Vec<Vec<crate::hir::HirStmt>>,
    /// Symbol table for variable classification (global/nonlocal/cell/free).
    /// None when created without symbols (basic lowering).
    symbol_table: Option<&'a SymbolTable>,
    /// SymbolId → TypeId for pattern-binding captures (#827 nested unboxing).
    /// Populated from HirModule.sym_types at the lowering entry points.
    sym_types: HashMap<SymbolId, TypeId>,
    /// SymbolId → name for synthesizing inline `locals()` snapshots inside
    /// function bodies. Populated from HirModule.sym_names at the lowering
    /// entry points. The lowerer prefers `symbol_table.get_symbol(id).name`
    /// for canonical syms (low IDs); this map covers lowerer-introduced
    /// synthetic syms (≥ 1_000_000).
    // HANDWRITE-BEGIN gap="standardize:projects-mamba-src-lower-hir-to-mir-rs" tracker="standardize-gap-projects-mamba-src-lower-hir-to-mir-rs" reason="introspection-builtins (issue: enhancement-mamba-introspection-builtins-globals-locals-vars-dir)."
    sym_names: HashMap<SymbolId, String>,
    /// HANDWRITE-END
    /// Return type of the currently-lowering function.
    /// Used in Return lowering to unbox `any`-typed values when function declares int/bool/float.
    current_return_ty: TypeId,
    /// Counter for allocating synthetic lambda SymbolIds (starts at 4_000_000).
    next_lambda_id: u32,
    /// Pending decorator applications: (func_sym, decorators_in_order).
    /// Emitted at the start of top-level code, after class registrations.
    pending_decorators: Vec<(SymbolId, Vec<HirExpr>)>,
    /// SymbolId.0 set for functions that have decorators.
    /// Call sites for these functions use dynamic dispatch via LoadGlobal.
    decorated_func_syms: HashSet<u32>,
    /// SymbolId.0 → return TypeId for decorated functions.
    /// Used to box return values after dynamic dispatch at call sites.
    decorated_func_return_tys: HashMap<u32, TypeId>,
    /// Synthetic SymbolIds (1M+) that are nonlocal-shared (Cell) for the current function.
    /// These must use global storage (StoreGlobal/LoadGlobal) so outer and inner functions
    /// share the same variable slot regardless of stack frames.
    cell_override: HashSet<u32>,
    /// SymbolId.0 → ordered parameter TypeIds for each user-defined function.
    /// Used at MirInst::Call sites to selectively box primitive arguments when the
    /// callee declares the parameter as Any/object, so match-subject comparisons via
    /// mb_eq receive uniform NaN-boxed MbValues (#827 R8).
    user_func_param_types: HashMap<u32, Vec<TypeId>>,
    /// SymbolId.0 → return TypeId for each user-defined function.
    /// Used by iter(callable, sentinel) lowering to detect primitive-returning callables
    /// that need a boxing thunk so mb_call0 receives properly NaN-boxed MbValues.
    user_func_return_tys: HashMap<u32, TypeId>,
    /// SymbolId.0 → (has_star_args, has_kwargs) for each user-defined function.
    /// Used at call sites to decide how to pack variadic positional and keyword arguments.
    user_func_variadic_info: HashMap<u32, (bool, bool)>,
    /// SymbolId.0 → declared name for each user-defined function.
    /// Top-level `def f(): ...` is hoisted into HirModule.functions and never
    /// flows through `HirStmt::FuncDefPlaceholder`, so the FUNC_NAMES registry
    /// must be primed at module-init from this map (so `f.__name__` works).
    user_func_names: HashMap<u32, String>,
    /// SymbolId.0 → docstring for each top-level user-defined function whose
    /// body starts with a bare string literal. Primed at module-init via
    /// `mb_func_set_doc` so `f.__doc__` returns the docstring (None otherwise).
    user_func_docs: HashMap<u32, String>,
    /// SymbolId.0 → positional argument count (`co_argcount`) for each
    /// user-defined function. Excludes `*args` / `**kwargs`. Primed at
    /// module-init via `mb_func_set_argcount` so
    /// `f.__code__.co_argcount` returns the real signature arity (CORE #3).
    user_func_argcounts: HashMap<u32, i64>,
    /// SymbolId.0 → parameter names (`co_varnames`) in declaration order for
    /// each user-defined function. Primed at module-init via
    /// `mb_func_set_varnames` so `f.__code__.co_varnames` returns the params
    /// (CORE #3).
    user_func_varnames: HashMap<u32, Vec<String>>,
    /// SymbolId.0 → declared signature metadata (names/kinds/defaults/
    /// annotations + return annotation) for each user-defined `def`. Primed
    /// at module-init via `mb_func_set_params` / `mb_func_set_retanno` so
    /// `inspect.signature(f)` reflects the real declaration.
    user_func_sigs: HashMap<u32, crate::hir::HirFuncSig>,
    /// SymbolId.0 → first source line for each user-defined `def`. Primed at
    /// module-init via `mb_func_set_srcinfo` (co_firstlineno). Only populated
    /// when the entry point received the module source.
    user_func_lines: HashMap<u32, u32>,
    /// Byte offsets of line starts in the module source (for span→line
    /// conversion of lambdas during expression lowering). None when the entry
    /// point didn't receive the source.
    src_line_starts: Option<Vec<u32>>,
    /// Module source filename (co_filename), when known.
    src_filename: Option<String>,
    /// (class_name, docstring) pairs primed at module-init via
    /// `mb_class_set_doc` so `inspect.getdoc(Cls)` works.
    pending_class_docs: Vec<(String, String)>,
    /// Module-scope variable annotations `(name, type_repr)` in source order.
    /// Primed at module-init into the `__annotations__` dict (PEP 526).
    module_annotations: Vec<(String, String)>,
    /// True when lowering module-level (top-level) statements.
    /// Local variable assignments at module scope also emit StoreGlobal so
    /// functions can read them back via LoadGlobal when there is no `global`
    /// declaration (implicit global read — valid Python but untracked by the
    /// resolver which leaves such variables as VariableClass::Local).
    in_module_scope: bool,
    /// R4 P1: True when lowering a class method body.
    /// Forces return values to be NaN-boxed so mb_call_method (dynamic dispatch)
    /// receives proper MbValues instead of raw primitives.
    is_class_method: bool,
}

impl<'a> HirToMir<'a> {
    fn new(tcx: &'a TypeContext) -> Self {
        let int_ty = tcx.int();
        Self {
            tcx,
            bodies: Vec::new(),
            next_vreg: 0,
            next_block: 0,
            blocks: Vec::new(),
            current_stmts: Vec::new(),
            sym_to_vreg: HashMap::new(),
            loop_exit: None,
            loop_header: None,
            current_block_id: None,
            async_coro_vreg: None,
            user_funcs: HashSet::new(),
            builtin_syms: HashMap::new(),
            class_syms: HashMap::new(),
            active_except_vreg: None,
            pending_classes: Vec::new(),
            classes_needing_textual_registration: HashSet::new(),
            pending_runtime_class_bases: Vec::new(),
            pending_class_attrs: Vec::new(),
            pending_abstract_methods: Vec::new(),
            pending_class_decorators: Vec::new(),
            pending_dataclass_fields: Vec::new(),
            user_class_syms: HashSet::new(),
            current_class_ctx: None,
            is_gen_body: false,
            try_handler_stack: Vec::new(),
            with_ctx_stack: Vec::new(),
            with_exit_stack: Vec::new(),
            finally_body_stack: Vec::new(),
            symbol_table: None,
            sym_types: HashMap::new(),
            sym_names: HashMap::new(),
            current_return_ty: int_ty,
            next_lambda_id: 0,
            pending_decorators: Vec::new(),
            decorated_func_syms: HashSet::new(),
            decorated_func_return_tys: HashMap::new(),
            cell_override: HashSet::new(),
            user_func_param_types: HashMap::new(),
            user_func_return_tys: HashMap::new(),
            user_func_variadic_info: HashMap::new(),
            user_func_names: HashMap::new(),
            user_func_docs: HashMap::new(),
            user_func_argcounts: HashMap::new(),
            user_func_varnames: HashMap::new(),
            user_func_sigs: HashMap::new(),
            user_func_lines: HashMap::new(),
            src_line_starts: None,
            src_filename: None,
            pending_class_docs: Vec::new(),
            module_annotations: Vec::new(),
            in_module_scope: false,
            is_class_method: false,
        }
    }

    fn new_with_builtins(
        tcx: &'a TypeContext,
        user_funcs: HashSet<u32>,
        builtin_syms: HashMap<u32, String>,
    ) -> Self {
        let int_ty = tcx.int();
        Self {
            tcx,
            bodies: Vec::new(),
            next_vreg: 0,
            next_block: 0,
            blocks: Vec::new(),
            current_stmts: Vec::new(),
            sym_to_vreg: HashMap::new(),
            loop_exit: None,
            loop_header: None,
            current_block_id: None,
            async_coro_vreg: None,
            user_funcs,
            builtin_syms,
            class_syms: HashMap::new(),
            active_except_vreg: None,
            pending_classes: Vec::new(),
            classes_needing_textual_registration: HashSet::new(),
            pending_runtime_class_bases: Vec::new(),
            pending_class_attrs: Vec::new(),
            pending_abstract_methods: Vec::new(),
            pending_class_decorators: Vec::new(),
            pending_dataclass_fields: Vec::new(),
            user_class_syms: HashSet::new(),
            current_class_ctx: None,
            is_gen_body: false,
            try_handler_stack: Vec::new(),
            with_ctx_stack: Vec::new(),
            with_exit_stack: Vec::new(),
            finally_body_stack: Vec::new(),
            symbol_table: None,
            sym_types: HashMap::new(),
            sym_names: HashMap::new(),
            current_return_ty: int_ty,
            next_lambda_id: 0,
            pending_decorators: Vec::new(),
            decorated_func_syms: HashSet::new(),
            decorated_func_return_tys: HashMap::new(),
            cell_override: HashSet::new(),
            user_func_param_types: HashMap::new(),
            user_func_return_tys: HashMap::new(),
            user_func_variadic_info: HashMap::new(),
            user_func_names: HashMap::new(),
            user_func_docs: HashMap::new(),
            user_func_argcounts: HashMap::new(),
            user_func_varnames: HashMap::new(),
            user_func_sigs: HashMap::new(),
            user_func_lines: HashMap::new(),
            src_line_starts: None,
            src_filename: None,
            pending_class_docs: Vec::new(),
            module_annotations: Vec::new(),
            in_module_scope: false,
            is_class_method: false,
        }
    }

    fn fresh_vreg(&mut self) -> VReg {
        let v = VReg(self.next_vreg);
        self.next_vreg += 1;
        v
    }

    fn fresh_block(&mut self) -> BlockId {
        let b = BlockId(self.next_block);
        self.next_block += 1;
        b
    }

    fn reset(&mut self) {
        self.next_vreg = 0;
        self.next_block = 0;
        self.blocks.clear();
        self.current_stmts.clear();
        self.sym_to_vreg.clear();
        self.loop_exit = None;
        self.loop_header = None;
        self.current_block_id = None;
        self.async_coro_vreg = None;
        self.is_gen_body = false;
        self.try_handler_stack.clear();
        self.finally_body_stack.clear();
        self.in_module_scope = false;
    }

    /// `*args` is a tuple in Python, but every call path packs the extra
    /// positional arguments into a list (some paths, e.g. atexit-forwarded
    /// calls, already pass a tuple). Emit a `mb_star_args_to_tuple` at function
    /// entry so the body observes `type(args) is tuple` and tuple immutability,
    /// matching CPython. The incoming param vreg keeps the ABI's value
    /// (callers are unchanged); the body reads the converted tuple via
    /// sym_to_vreg. `*args` is `any`-typed, so body operations already route
    /// through polymorphic runtime dispatch and work unchanged on a tuple.
    fn emit_star_args_to_tuple(&mut self, func: &HirFunction, any_ty: TypeId) {
        if let Some(star_pos) = func.star_param_pos {
            if let Some((star_sym, _)) = func.params.get(star_pos) {
                if let Some(&packed_vreg) = self.sym_to_vreg.get(star_sym) {
                    let tuple_vreg = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(tuple_vreg),
                        name: "mb_star_args_to_tuple".to_string(),
                        args: vec![packed_vreg],
                        ty: any_ty,
                    });
                    self.sym_to_vreg.insert(*star_sym, tuple_vreg);
                }
            }
        }
    }

    fn lower_function(&mut self, func: &HirFunction) -> MirBody {
        if func.is_async {
            return self.lower_async_function(func);
        }
        if func.is_generator {
            return self.lower_generator_function(func);
        }

        // Register variadic (*args) functions so the JIT can identify them by address.
        if func.has_star_args {
            crate::runtime::module::register_variadic_symbol(func.name.0);
        }
        if func.has_kwargs {
            crate::runtime::module::register_kwargs_symbol(func.name.0);
        }

        self.reset();
        let entry = self.fresh_block();
        self.current_block_id = Some(entry);

        // Track current function return type so Return lowering can unbox any→int (#827).
        self.current_return_ty = func.return_ty;

        // Populate cell_override from the function's captures (nonlocal-shared symbols).
        // These synthetic SymbolIds (1M+) must use global storage for consistency across
        // outer (Cell) and inner (Free/nonlocal) function frames.
        self.cell_override = func.captures.iter().map(|s| s.0).collect();

        // Decorated functions are dispatched dynamically through `mb_call1_val` /
        // `mb_call_spread`, which pass NaN-boxed MbValues. To keep a single ABI
        // for the decorated call path, compile decorated function params as
        // `any` (MbValue-sized) and unbox to the inferred primitive type at
        // entry. This allows `@lru_cache`-style identity decorators to work
        // without corrupting primitive args.
        let is_decorated = self.decorated_func_syms.contains(&func.name.0);
        let any_ty = self.tcx.any();
        let int_ty = self.tcx.int();
        let bool_ty = self.tcx.bool();
        let float_ty = self.tcx.float();

        // Allocate vregs for parameters. For decorated functions, the incoming
        // vreg is `any` and we add an unbox instruction to populate a second
        // vreg with the primitive type that `sym_to_vreg` points at.
        let params: Vec<(VReg, TypeId)> = func
            .params
            .iter()
            .map(|(sym, orig_ty)| {
                if is_decorated
                    && (*orig_ty == int_ty || *orig_ty == bool_ty || *orig_ty == float_ty)
                {
                    let any_vreg = self.fresh_vreg();
                    let typed_vreg = self.fresh_vreg();
                    let (unbox_name, dest_ty) = if *orig_ty == int_ty {
                        ("mb_unbox_int", int_ty)
                    } else if *orig_ty == bool_ty {
                        ("mb_unbox_bool", bool_ty)
                    } else {
                        ("mb_unbox_float", float_ty)
                    };
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(typed_vreg),
                        name: unbox_name.to_string(),
                        args: vec![any_vreg],
                        ty: dest_ty,
                    });
                    self.sym_to_vreg.insert(*sym, typed_vreg);
                    (any_vreg, any_ty)
                } else {
                    let vreg = self.fresh_vreg();
                    self.sym_to_vreg.insert(*sym, vreg);
                    (vreg, *orig_ty)
                }
            })
            .collect();

        self.emit_star_args_to_tuple(func, any_ty);

        // Store parameters to global storage if they are cell variables (captured by inner
        // functions via implicit or explicit nonlocal). This ensures that when an inner
        // function reads LoadGlobal(param_sym), it gets the value passed at call time.
        // Parameters use raw-int convention (int_ty → i64), so we NaN-box them first so
        // that LoadGlobal + dynamic dispatch (mb_call0/mb_call1_val) can treat the stored
        // value as a proper MbValue.
        for (sym, ty) in &func.params {
            if self.cell_override.contains(&sym.0) {
                let vreg = *self.sym_to_vreg.get(sym).expect("param just inserted");
                let boxed = self.box_operand(vreg, *ty);
                self.current_stmts.push(MirInst::StoreGlobal {
                    name: *sym,
                    value: boxed,
                });
            }
        }

        // Lower body
        for stmt in &func.body {
            self.lower_stmt(stmt);
        }

        // Finish any open block with an implicit Return(None)
        if self.current_block_id.is_some() {
            self.finish_block(Terminator::Return(None));
        }

        MirBody {
            name: func.name,
            params,
            return_ty: func.return_ty,
            blocks: std::mem::take(&mut self.blocks),
        }
    }

    /// Lower generator function into wrapper + body.
    ///
    /// Generates TWO MirBodies (like async functions):
    /// 1. Body (`fn_N_gen`): the actual generator body. Yield points call
    ///    `mb_generator_yield_value`. Runs in a separate thread.
    /// 2. Wrapper (`fn_N`): creates generator, stores args, returns handle.
    fn lower_generator_function(&mut self, func: &HirFunction) -> MirBody {
        let int_ty = self.tcx.int();
        let none_ty = self.tcx.none();
        let bool_ty = self.tcx.bool();
        let float_ty = self.tcx.float();
        let any_ty = self.tcx.any();

        // Synthetic SymbolId for the body function
        let body_sym = SymbolId(func.name.0.wrapping_add(3_000_000));

        // ── Phase 1: Generate body function ──
        self.reset();
        self.is_gen_body = true; // Enable return-value boxing for generator bodies
        let entry = self.fresh_block();
        self.current_block_id = Some(entry);

        // Body params arrive through `call_body_fn`, which transmutes the body to
        // `extern "C" fn(i64, ..) -> i64` and passes each stored arg as raw
        // `MbValue::to_bits()` (always a NaN-boxed i64). Declaring a Float param as
        // F64 here would make Cranelift expect it in an FP register, while the i64
        // trampoline ABI delivers it in an integer register — a register-class
        // mismatch that leaks raw IEEE-754 bits (or crashes). For an explicitly
        // *annotated* primitive param, mirror the decorated path: declare it `any`
        // (NaN-boxed i64) and unbox to the original primitive at entry. (Unannotated
        // generator params already arrive as `any` from ast_to_hir, so they fall
        // through unchanged and the body uses NaN-aware runtime dispatch.) The
        // wrapper boxes each arg before `mb_generator_store_arg`, so the value the
        // body unboxes here is always a proper NaN-boxed MbValue.
        let body_params: Vec<(VReg, TypeId)> = func
            .params
            .iter()
            .map(|(sym, ty)| {
                if *ty == int_ty || *ty == bool_ty || *ty == float_ty {
                    let any_vreg = self.fresh_vreg();
                    let typed_vreg = self.fresh_vreg();
                    let (unbox_name, dest_ty) = if *ty == int_ty {
                        ("mb_unbox_int", int_ty)
                    } else if *ty == bool_ty {
                        ("mb_unbox_bool", bool_ty)
                    } else {
                        ("mb_unbox_float", float_ty)
                    };
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(typed_vreg),
                        name: unbox_name.to_string(),
                        args: vec![any_vreg],
                        ty: dest_ty,
                    });
                    self.sym_to_vreg.insert(*sym, typed_vreg);
                    (any_vreg, any_ty)
                } else {
                    let vreg = self.fresh_vreg();
                    self.sym_to_vreg.insert(*sym, vreg);
                    (vreg, *ty)
                }
            })
            .collect();

        self.emit_star_args_to_tuple(func, any_ty);

        // Lower the actual function body (yield becomes mb_generator_yield_value)
        for stmt in &func.body {
            self.lower_stmt(stmt);
        }

        // Implicit return: generator body returns None (StopIteration with no value)
        if self.current_block_id.is_some() {
            let none_vreg = self.emit_none();
            self.finish_block(Terminator::Return(Some(none_vreg)));
        }

        let body_mir = MirBody {
            name: body_sym,
            params: body_params.clone(),
            return_ty: int_ty,
            blocks: std::mem::take(&mut self.blocks),
        };
        self.bodies.push(body_mir);

        // ── Phase 2: Generate wrapper (constructor) function ──
        self.reset();
        let entry = self.fresh_block();
        self.current_block_id = Some(entry);

        // Wrapper has the original function params
        let params: Vec<(VReg, TypeId)> = func
            .params
            .iter()
            .map(|(sym, ty)| {
                let vreg = self.fresh_vreg();
                self.sym_to_vreg.insert(*sym, vreg);
                (vreg, *ty)
            })
            .collect();

        // Create generator: mb_generator_create(name, body_fn_addr)
        let name_vreg = self.fresh_vreg();
        self.current_stmts.push(MirInst::LoadConst {
            dest: name_vreg,
            value: MirConst::Str(format!("gen_{}", func.name.0)),
            ty: self.tcx.str(),
        });
        let body_fn_ptr = self.fresh_vreg();
        self.current_stmts.push(MirInst::LoadConst {
            dest: body_fn_ptr,
            value: MirConst::FuncRef(body_sym),
            ty: int_ty,
        });
        let gen_handle = self.fresh_vreg();
        self.current_stmts.push(MirInst::CallExtern {
            dest: Some(gen_handle),
            name: "mb_generator_create".to_string(),
            args: vec![name_vreg, body_fn_ptr],
            ty: int_ty,
        });

        // Store function args for deferred thread spawn. NaN-box each primitive
        // arg first so the stored value is always a proper MbValue: `call_body_fn`
        // delivers these to the body verbatim (as i64 bits), and the body unboxes
        // Int/Bool/Float params back to their declared primitive at entry. Without
        // boxing here, a Float param would be stored as raw IEEE-754 bits and the
        // body would unbox garbage.
        for (vreg, ty) in &params {
            let boxed = self.box_operand(*vreg, *ty);
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_generator_store_arg".to_string(),
                args: vec![gen_handle, boxed],
                ty: none_ty,
            });
        }

        // Return generator handle
        self.finish_block(Terminator::Return(Some(gen_handle)));

        MirBody {
            name: func.name,
            params,
            return_ty: int_ty,
            blocks: std::mem::take(&mut self.blocks),
        }
    }

    /// Lower async function into wrapper + body (#313 R1).
    ///
    /// Generates TWO MirBodies:
    /// 1. Body (`fn_N_step`): reads args from coroutine locals, executes body,
    ///    completes coroutine with result. Pushed into `self.bodies`.
    /// 2. Wrapper (`fn_N`): creates coroutine, stores args as locals,
    ///    calls body function, returns coroutine handle. Returned from this fn.
    fn lower_async_function(&mut self, func: &HirFunction) -> MirBody {
        let int_ty = self.tcx.int();
        let none_ty = self.tcx.none();

        // Synthetic SymbolId for the body function (step function)
        let body_sym = SymbolId(func.name.0.wrapping_add(2_000_000));

        // ── Phase 1: Generate body function ──
        self.reset();
        let entry = self.fresh_block();
        self.current_block_id = Some(entry);

        // Body takes a single param: coroutine handle
        let handle_vreg = self.fresh_vreg();

        // Read original function args from coroutine locals.
        // NB: coroutine locals are always NaN-boxed MbValue (i64), so the
        // result type of mb_coroutine_get_local is int_ty regardless of the
        // parameter's declared type. Using *ty here would cause Cranelift
        // to declare the VReg as F64 for float params, mismatching the
        // actual I64 return from the extern call.
        for (i, (sym, _ty)) in func.params.iter().enumerate() {
            let idx_vreg = self.fresh_vreg();
            self.current_stmts.push(MirInst::LoadConst {
                dest: idx_vreg,
                value: MirConst::Int(i as i64),
                ty: int_ty,
            });
            let arg_vreg = self.fresh_vreg();
            self.current_stmts.push(MirInst::CallExtern {
                dest: Some(arg_vreg),
                name: "mb_coroutine_get_local".to_string(),
                args: vec![handle_vreg, idx_vreg],
                ty: int_ty,
            });
            self.sym_to_vreg.insert(*sym, arg_vreg);
        }

        let any_ty = self.tcx.any();
        self.emit_star_args_to_tuple(func, any_ty);

        // Track coroutine handle for return wrapping
        self.async_coro_vreg = Some(handle_vreg);

        // Lower the actual function body
        for stmt in &func.body {
            self.lower_stmt(stmt);
        }

        // Implicit return: complete coroutine with None
        if self.current_block_id.is_some() {
            let none_vreg = self.fresh_vreg();
            self.current_stmts.push(MirInst::LoadConst {
                dest: none_vreg,
                value: MirConst::None,
                ty: none_ty,
            });
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_coroutine_complete".to_string(),
                args: vec![handle_vreg, none_vreg],
                ty: none_ty,
            });
            self.finish_block(Terminator::Return(Some(handle_vreg)));
        }

        let body_mir = MirBody {
            name: body_sym,
            params: vec![(handle_vreg, int_ty)],
            return_ty: int_ty,
            blocks: std::mem::take(&mut self.blocks),
        };
        self.bodies.push(body_mir);

        // ── Phase 2: Generate wrapper function ──
        self.reset();
        let entry = self.fresh_block();
        self.current_block_id = Some(entry);

        // Wrapper has the original function params
        let params: Vec<(VReg, TypeId)> = func
            .params
            .iter()
            .map(|(sym, ty)| {
                let vreg = self.fresh_vreg();
                self.sym_to_vreg.insert(*sym, vreg);
                (vreg, *ty)
            })
            .collect();

        // Create coroutine: mb_coroutine_new(name, empty_list)
        let name_vreg = self.fresh_vreg();
        self.current_stmts.push(MirInst::LoadConst {
            dest: name_vreg,
            value: MirConst::Str(format!("fn_{}", func.name.0)),
            ty: self.tcx.str(),
        });
        let locals_vreg = self.fresh_vreg();
        self.current_stmts.push(MirInst::MakeList {
            dest: locals_vreg,
            elements: Vec::new(),
            ty: self.tcx.any(),
        });
        let coro_handle = self.fresh_vreg();
        self.current_stmts.push(MirInst::CallExtern {
            dest: Some(coro_handle),
            name: "mb_coroutine_new".to_string(),
            args: vec![name_vreg, locals_vreg],
            ty: int_ty,
        });

        // Store function args as coroutine locals
        for (i, (vreg, _)) in params.iter().enumerate() {
            let idx_vreg = self.fresh_vreg();
            self.current_stmts.push(MirInst::LoadConst {
                dest: idx_vreg,
                value: MirConst::Int(i as i64),
                ty: int_ty,
            });
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_coroutine_set_local".to_string(),
                args: vec![coro_handle, idx_vreg, *vreg],
                ty: none_ty,
            });
        }

        // Register body function pointer for deferred execution (#313 R1)
        let body_fn_ptr = self.fresh_vreg();
        self.current_stmts.push(MirInst::LoadConst {
            dest: body_fn_ptr,
            value: MirConst::FuncRef(body_sym),
            ty: int_ty,
        });
        self.current_stmts.push(MirInst::CallExtern {
            dest: None,
            name: "mb_coroutine_set_body".to_string(),
            args: vec![coro_handle, body_fn_ptr],
            ty: none_ty,
        });

        // Return coroutine handle — body NOT called here, deferred to mb_coroutine_step
        self.finish_block(Terminator::Return(Some(coro_handle)));

        MirBody {
            name: func.name,
            params,
            return_ty: int_ty,
            blocks: std::mem::take(&mut self.blocks),
        }
    }

    fn lower_top_level(&mut self, stmts: &[HirStmt]) -> MirBody {
        self.reset();
        // Mark module scope so Local variable assignments also emit StoreGlobal,
        // making them accessible to functions that read them without a `global` decl.
        self.in_module_scope = true;
        let entry = self.fresh_block();
        self.current_block_id = Some(entry);

        // Ensure stdlib modules are registered in this thread before any mb_import calls.
        // MODULES is thread-local, so it must be populated in the JIT execution thread.
        self.current_stmts.push(MirInst::CallExtern {
            dest: None,
            name: "mb_register_builtins".to_string(),
            args: vec![],
            ty: self.tcx.none(),
        });

        // Prime the FUNC_NAMES registry for every user-defined function so
        // `f.__name__` works on top-level `def`s (which are hoisted into
        // HirModule.functions and never flow through `FuncDefPlaceholder`).
        let any_ty = self.tcx.any();
        let func_name_pairs: Vec<(SymbolId, String)> = self
            .user_func_names
            .iter()
            .map(|(sid, name)| (SymbolId(*sid), name.clone()))
            .collect();
        for (func_sym, fname) in &func_name_pairs {
            let fn_vreg = self.fresh_vreg();
            self.current_stmts.push(MirInst::LoadConst {
                dest: fn_vreg,
                value: MirConst::FuncRef(*func_sym),
                ty: any_ty,
            });
            let name_vreg = self.emit_str_const(fname);
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_func_set_name".to_string(),
                args: vec![fn_vreg, name_vreg],
                ty: self.tcx.none(),
            });
        }

        // Prime FUNC_DOCS for top-level defs whose body starts with a string
        // literal so `f.__doc__` returns the docstring (None otherwise).
        let func_doc_pairs: Vec<(SymbolId, String)> = self
            .user_func_docs
            .iter()
            .map(|(sid, doc)| (SymbolId(*sid), doc.clone()))
            .collect();
        for (func_sym, fdoc) in &func_doc_pairs {
            let fn_vreg = self.fresh_vreg();
            self.current_stmts.push(MirInst::LoadConst {
                dest: fn_vreg,
                value: MirConst::FuncRef(*func_sym),
                ty: any_ty,
            });
            let doc_vreg = self.emit_str_const(fdoc);
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_func_set_doc".to_string(),
                args: vec![fn_vreg, doc_vreg],
                ty: self.tcx.none(),
            });
        }

        // Prime code-object introspection metadata (CORE #3) for every
        // top-level def so `f.__code__.co_argcount` / `.co_varnames` return the
        // real compiled signature. argcount is an int const; varnames is a list
        // of string consts (mb_func_set_varnames stores them as a tuple).
        let func_argcount_pairs: Vec<(SymbolId, i64)> = self
            .user_func_argcounts
            .iter()
            .map(|(sid, n)| (SymbolId(*sid), *n))
            .collect();
        for (func_sym, argcount) in &func_argcount_pairs {
            let fn_vreg = self.fresh_vreg();
            self.current_stmts.push(MirInst::LoadConst {
                dest: fn_vreg,
                value: MirConst::FuncRef(*func_sym),
                ty: any_ty,
            });
            let n_vreg = self.fresh_vreg();
            self.current_stmts.push(MirInst::LoadConst {
                dest: n_vreg,
                value: MirConst::Int(*argcount),
                ty: self.tcx.int(),
            });
            // Box the raw native int into a NaN-boxed MbValue so the extern
            // receives a tagged integer (matching mb_func_set_argcount's
            // `argcount.as_int()` read).
            let n_boxed = self.box_operand(n_vreg, self.tcx.int());
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_func_set_argcount".to_string(),
                args: vec![fn_vreg, n_boxed],
                ty: self.tcx.none(),
            });
        }

        let func_varname_pairs: Vec<(SymbolId, Vec<String>)> = self
            .user_func_varnames
            .iter()
            .map(|(sid, names)| (SymbolId(*sid), names.clone()))
            .collect();
        for (func_sym, names) in &func_varname_pairs {
            let fn_vreg = self.fresh_vreg();
            self.current_stmts.push(MirInst::LoadConst {
                dest: fn_vreg,
                value: MirConst::FuncRef(*func_sym),
                ty: any_ty,
            });
            let name_vregs: Vec<VReg> = names.iter().map(|n| self.emit_str_const(n)).collect();
            let names_vreg = self.fresh_vreg();
            self.current_stmts.push(MirInst::MakeList {
                dest: names_vreg,
                elements: name_vregs,
                ty: any_ty,
            });
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_func_set_varnames".to_string(),
                args: vec![fn_vreg, names_vreg],
                ty: self.tcx.none(),
            });
        }

        // Prime FUNC_PARAMS / FUNC_RET_ANNOS so `inspect.signature(f)` (and
        // getfullargspec) reflect the declared signature. Each parameter is
        // packed as a (name, kind, has_default, default, annotation) tuple;
        // kind is the CPython Parameter ordinal (0 POSITIONAL_ONLY ..
        // 4 VAR_KEYWORD).
        let func_sig_pairs: Vec<(SymbolId, crate::hir::HirFuncSig)> = self
            .user_func_sigs
            .iter()
            .map(|(sid, sig)| (SymbolId(*sid), sig.clone()))
            .collect();
        for (func_sym, sig) in &func_sig_pairs {
            let fn_vreg = self.fresh_vreg();
            self.current_stmts.push(MirInst::LoadConst {
                dest: fn_vreg,
                value: MirConst::FuncRef(*func_sym),
                ty: any_ty,
            });
            let mut param_vregs: Vec<VReg> = Vec::new();
            for p in &sig.params {
                let name_vreg = self.emit_str_const(&p.name);
                let kind_raw = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest: kind_raw,
                    value: MirConst::Int(p.kind as i64),
                    ty: self.tcx.int(),
                });
                let kind_vreg = self.box_operand(kind_raw, self.tcx.int());
                let has_default = p.default.is_some() || p.default_opaque;
                let hd_raw = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest: hd_raw,
                    value: MirConst::Int(if has_default { 1 } else { 0 }),
                    ty: self.tcx.int(),
                });
                let hd_vreg = self.box_operand(hd_raw, self.tcx.int());
                let def_vreg = match &p.default {
                    Some(crate::hir::HirSigDefault::Int(v)) => {
                        let raw = self.fresh_vreg();
                        self.current_stmts.push(MirInst::LoadConst {
                            dest: raw,
                            value: MirConst::Int(*v),
                            ty: self.tcx.int(),
                        });
                        self.box_operand(raw, self.tcx.int())
                    }
                    Some(crate::hir::HirSigDefault::Float(v)) => {
                        let raw = self.fresh_vreg();
                        self.current_stmts.push(MirInst::LoadConst {
                            dest: raw,
                            value: MirConst::Float(*v),
                            ty: self.tcx.float(),
                        });
                        self.box_operand(raw, self.tcx.float())
                    }
                    Some(crate::hir::HirSigDefault::Str(s)) => self.emit_str_const(s),
                    Some(crate::hir::HirSigDefault::Bool(b)) => {
                        let raw = self.fresh_vreg();
                        self.current_stmts.push(MirInst::LoadConst {
                            dest: raw,
                            value: MirConst::Bool(*b),
                            ty: self.tcx.bool(),
                        });
                        self.box_operand(raw, self.tcx.bool())
                    }
                    Some(crate::hir::HirSigDefault::None) | None => self.emit_none(),
                };
                let anno_vreg = match &p.annotation {
                    Some(s) => self.emit_str_const(s),
                    None => self.emit_none(),
                };
                let tup_vreg = self.fresh_vreg();
                self.current_stmts.push(MirInst::MakeTuple {
                    dest: tup_vreg,
                    elements: vec![name_vreg, kind_vreg, hd_vreg, def_vreg, anno_vreg],
                    ty: any_ty,
                });
                param_vregs.push(tup_vreg);
            }
            let params_vreg = self.fresh_vreg();
            self.current_stmts.push(MirInst::MakeList {
                dest: params_vreg,
                elements: param_vregs,
                ty: any_ty,
            });
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_func_set_params".to_string(),
                args: vec![fn_vreg, params_vreg],
                ty: self.tcx.none(),
            });
            if let Some(ret_anno) = &sig.return_annotation {
                let ra_vreg = self.emit_str_const(ret_anno);
                self.current_stmts.push(MirInst::CallExtern {
                    dest: None,
                    name: "mb_func_set_retanno".to_string(),
                    args: vec![fn_vreg, ra_vreg],
                    ty: self.tcx.none(),
                });
            }
        }

        // Prime FUNC_LINES / FUNC_FILES (co_firstlineno / co_filename) for
        // every def whose source location was resolved at the entry point.
        let func_line_pairs: Vec<(SymbolId, u32)> = self
            .user_func_lines
            .iter()
            .map(|(sid, line)| (SymbolId(*sid), *line))
            .collect();
        if !func_line_pairs.is_empty() {
            let filename = self.src_filename.clone().unwrap_or_default();
            for (func_sym, line) in &func_line_pairs {
                let fn_vreg = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest: fn_vreg,
                    value: MirConst::FuncRef(*func_sym),
                    ty: any_ty,
                });
                let line_raw = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest: line_raw,
                    value: MirConst::Int(*line as i64),
                    ty: self.tcx.int(),
                });
                let line_vreg = self.box_operand(line_raw, self.tcx.int());
                let file_vreg = self.emit_str_const(&filename);
                self.current_stmts.push(MirInst::CallExtern {
                    dest: None,
                    name: "mb_func_set_srcinfo".to_string(),
                    args: vec![fn_vreg, line_vreg, file_vreg],
                    ty: self.tcx.none(),
                });
            }
        }

        // Module `__annotations__` (PEP 526). CPython always exposes a module
        // `__annotations__` dict; annotated module-scope names (`x: int`,
        // `x: int = v`) record their key here. Create the dict and StoreGlobal
        // it under the `__annotations__` symbol, then populate each key.
        if let Some(ann_sym) = self
            .symbol_table
            .and_then(|st| st.lookup("__annotations__"))
        {
            let any_ty = self.tcx.any();
            let dict_vreg = self.fresh_vreg();
            self.current_stmts.push(MirInst::MakeDict {
                dest: dict_vreg,
                keys: vec![],
                values: vec![],
                ty: any_ty,
            });
            let annotations = self.module_annotations.clone();
            for (name, type_repr) in &annotations {
                let key_vreg = self.emit_str_const(name);
                let val_vreg = self.emit_str_const(type_repr);
                self.current_stmts.push(MirInst::CallExtern {
                    dest: None,
                    name: "mb_dict_setitem".to_string(),
                    args: vec![dict_vreg, key_vreg, val_vreg],
                    ty: self.tcx.none(),
                });
            }
            self.current_stmts.push(MirInst::StoreGlobal {
                name: ann_sym,
                value: dict_vreg,
            });
            self.sym_to_vreg.insert(ann_sym, dict_vreg);
        }

        // Emit class registrations that do not need runtime class keyword
        // evaluation. Classes with class kwargs are emitted at their
        // ClassDefPlaceholder so preceding top-level bindings are visible.
        self.emit_pending_class_registrations(None);

        // Prime CLASS_DOCS so `inspect.getdoc(Cls)` / `Cls.__doc__` see the
        // class-body docstring.
        let class_doc_pairs = std::mem::take(&mut self.pending_class_docs);
        for (cls_name, doc) in &class_doc_pairs {
            let name_vreg = self.emit_str_const(cls_name);
            let doc_vreg = self.emit_str_const(doc);
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_class_set_doc".to_string(),
                args: vec![name_vreg, doc_vreg],
                ty: self.tcx.none(),
            });
        }

        // P2-R3: Class-level attribute assignments are NOT emitted here.
        // Initializer expressions like `X = enum.auto()` or `X = math.floor(2.5)`
        // reference imports bound by `import` statements that live in `stmts`
        // and have not yet been lowered — evaluating them before the main stmt
        // loop would resolve those names against an empty module (same defect
        // class as the #1686/#1690 decorator-ordering bug). Each class emits a
        // `HirStmt::ClassDefPlaceholder` at its textual position (ast_to_hir),
        // and `lower_stmt` consumes `pending_class_attrs` there.

        // Note: pending_class_decorators is intentionally NOT consumed here.
        // Class decorator expressions can reference imported symbols (e.g.
        // `@unittest.skipUnless(...)`), which are bound by `import` statements
        // that live in `stmts` and have not yet been lowered. Emitting the
        // decorator calls before the main stmt loop runs would resolve those
        // names against an empty module — `unittest` would be `None` and
        // attribute access would crash. Instead, apply class decorators
        // *after* the main stmt loop below, when all module imports and
        // top-level bindings have run. (#1686)

        // Decorator applications are emitted inline via HirStmt::FuncDefPlaceholder.
        // pending_decorators is consumed by lower_stmt as placeholders are encountered;
        // do NOT clear it here — the placeholders need to pull from it in source order.

        // If the last top-level statement is a bare function call (e.g. `f()` at module
        // scope in a JIT fixture), capture its return value so __main__ can return it.
        // This lets fixture files with class definitions expose `f()` results through
        // the __main__ entry point (#827).
        // Note: only function-call expressions are captured; other bare expressions
        // (list/dict literals, etc.) are emitted normally and __main__ returns 0.
        let n = stmts.len();
        let mut last_expr_vreg: Option<VReg> = None;
        let mut last_expr_ty: Option<TypeId> = None;
        for (i, stmt) in stmts.iter().enumerate() {
            if i + 1 == n {
                if let HirStmt::Expr { ref expr, .. } = stmt {
                    if let HirExpr::Call { ty: call_ty, .. } = expr {
                        let val = self.lower_expr(expr);
                        last_expr_vreg = Some(val);
                        last_expr_ty = Some(*call_ty);
                        continue;
                    }
                }
            }
            self.lower_stmt(stmt);
        }

        // P2-R3 fallback: drain any class-attr assignments whose class never
        // produced a ClassDefPlaceholder (defensive; top-level classes always
        // emit one when they carry attr assigns).
        self.emit_class_attrs_for(None);

        // (#1690) Class decorators are no longer applied here. Each decorated
        // class emits a `HirStmt::ClassDefPlaceholder` at its textual
        // position (see ast_to_hir.rs), and `lower_stmt` consumes
        // `pending_class_decorators` when it lowers that placeholder. This
        // preserves Python semantics: decorators run *after* preceding
        // imports/bindings (#1686 motivation) AND *before* subsequent
        // statements that consume the decorated class.

        // When the last top-level expression is a typed-primitive call
        // (e.g. `f()` where f returns int), propagate that type as
        // __main__'s return type so the JIT entry caller — which expects
        // `extern "C" fn() -> i64` — receives the raw unboxed value.
        // Otherwise emit_internal_call's callsite-mismatch path boxes the
        // typed-int result through mb_box_int and the entry caller sees a
        // NaN-boxed pointer instead of a raw int.
        //
        // The function may legitimately return either form: literal `42`
        // arrives raw; `IfExpr` / `getattr` paths arrive NaN-boxed because
        // hir_to_mir uniformly NaN-boxes branch results. Route through
        // `mb_unbox_*_if_boxed` so both forms collapse to the raw primitive
        // the entry-caller expects.
        let return_ty = match last_expr_ty {
            Some(ty) if matches!(self.tcx.get(ty), Ty::Int | Ty::Bool | Ty::Float) => ty,
            _ => self.tcx.none(),
        };
        if let (Some(vreg), Some(ty)) = (last_expr_vreg, last_expr_ty) {
            let unbox_name = match self.tcx.get(ty) {
                Ty::Int => Some("mb_unbox_int_if_boxed"),
                Ty::Bool => Some("mb_unbox_bool_if_boxed"),
                Ty::Float => Some("mb_unbox_float_if_boxed"),
                _ => None,
            };
            if let Some(name) = unbox_name {
                let unboxed = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(unboxed),
                    name: name.to_string(),
                    args: vec![vreg],
                    ty,
                });
                last_expr_vreg = Some(unboxed);
            }
        }

        if self.current_block_id.is_some() {
            let term = if let Some(vreg) = last_expr_vreg {
                Terminator::Return(Some(vreg))
            } else {
                Terminator::Return(None)
            };
            self.finish_block(term);
        }

        MirBody {
            name: SymbolId(u32::MAX), // sentinel for __main__
            params: Vec::new(),
            return_ty,
            blocks: std::mem::take(&mut self.blocks),
        }
    }

    /// REPL-aware top-level lowering.
    /// - Restores known globals at start via mb_global_get
    /// - Saves all variables at end via mb_global_set
    /// - Returns last expression value for REPL echo
    /// Returns (MirBody, new_globals, has_expression_echo).
    fn lower_top_level_repl(
        &mut self,
        stmts: &[HirStmt],
        sym_names: &std::collections::HashMap<SymbolId, String>,
        sym_types: &std::collections::HashMap<SymbolId, TypeId>,
        prev_globals: &[String],
    ) -> (MirBody, Vec<String>, bool) {
        self.reset();
        let entry = self.fresh_block();
        self.current_block_id = Some(entry);

        // Build reverse map: name → SymbolId from sym_names
        let name_to_sym: std::collections::HashMap<&str, SymbolId> = sym_names
            .iter()
            .map(|(&id, name)| (name.as_str(), id))
            .collect();

        // Initialize __name__ = "__main__" for REPL sessions (#1133).
        // REPL is an entry-point context; __name__ must equal "__main__" per CPython.
        // Eagerly emit StoreGlobal so LoadGlobal reads work for REPL Var references
        // (REPL doesn't use in_module_scope, so Var falls through to LoadGlobal).
        if let Some(&name_sym) = name_to_sym.get("__name__") {
            let main_vreg = self.emit_str_const("__main__");
            self.current_stmts.push(MirInst::StoreGlobal {
                name: name_sym,
                value: main_vreg,
            });
            self.sym_to_vreg.insert(name_sym, main_vreg);
        }

        // Restore previous globals via mb_global_get_id(sym_id)
        // Use the actual type from sym_types so arithmetic stays primitive
        for name in prev_globals {
            if let Some(&sym_id) = name_to_sym.get(name.as_str()) {
                let var_ty = sym_types
                    .get(&sym_id)
                    .copied()
                    .unwrap_or_else(|| self.tcx.any());
                let id_vreg = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest: id_vreg,
                    value: MirConst::Int(sym_id.0 as i64),
                    ty: self.tcx.int(),
                });
                let result = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(result),
                    name: "mb_global_get_id".to_string(),
                    args: vec![id_vreg],
                    ty: var_ty,
                });
                self.sym_to_vreg.insert(sym_id, result);
            }
        }

        // Lower all statements except possibly the last expression
        let mut last_expr_vreg: Option<VReg> = None;
        let n = stmts.len();
        for (i, stmt) in stmts.iter().enumerate() {
            if i == n - 1 {
                // If last statement is a bare expression, capture its value
                // for REPL echo (R1 acceptance: `x` should print 10)
                if let HirStmt::Expr { ref expr, .. } = stmt {
                    let val = self.lower_expr(expr);
                    last_expr_vreg = Some(val);
                    continue;
                }
            }
            self.lower_stmt(stmt);
        }

        // Collect all top-level variable names from sym_names
        let mut new_globals = Vec::new();
        let save_pairs: Vec<(SymbolId, String, VReg)> = self
            .sym_to_vreg
            .iter()
            .filter_map(|(sym_id, &vreg)| {
                sym_names.get(sym_id).map(|name| {
                    new_globals.push(name.clone());
                    (*sym_id, name.clone(), vreg)
                })
            })
            .collect();

        // Save all variables: mb_global_set_id(sym_id, vreg)
        if self.current_block_id.is_some() {
            for (sym_id, _name, vreg) in &save_pairs {
                let id_vreg = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest: id_vreg,
                    value: MirConst::Int(sym_id.0 as i64),
                    ty: self.tcx.int(),
                });
                self.current_stmts.push(MirInst::CallExtern {
                    dest: None,
                    name: "mb_global_set_id".to_string(),
                    args: vec![id_vreg, *vreg],
                    ty: self.tcx.none(),
                });
            }
            // Return last expression value for REPL echo, or None
            self.finish_block(Terminator::Return(last_expr_vreg));
        }

        let has_echo = last_expr_vreg.is_some();
        let body = MirBody {
            name: SymbolId(u32::MAX),
            params: Vec::new(),
            return_ty: self.tcx.any(),
            blocks: std::mem::take(&mut self.blocks),
        };
        (body, new_globals, has_echo)
    }

    fn lower_delete_lvalue(&mut self, target: &HirLValue) {
        match target {
            HirLValue::Attr { object, attr } => {
                let obj = self.lower_expr(object);
                let attr_vreg = self.emit_str_const(attr);
                self.current_stmts.push(MirInst::CallExtern {
                    dest: None,
                    name: "mb_delattr".to_string(),
                    args: vec![obj, attr_vreg],
                    ty: self.tcx.none(),
                });
            }
            HirLValue::Index { object, index } => {
                let obj = self.lower_expr(object);
                let idx_raw = self.lower_expr(index);
                let idx = self.box_operand(idx_raw, index.ty());
                self.current_stmts.push(MirInst::CallExtern {
                    dest: None,
                    name: "mb_obj_delitem".to_string(),
                    args: vec![obj, idx],
                    ty: self.tcx.none(),
                });
            }
            HirLValue::Var(sym_id) => {
                if let Some(&vreg) = self.sym_to_vreg.get(sym_id) {
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: None,
                        name: "mb_del_var".to_string(),
                        args: vec![vreg],
                        ty: self.tcx.none(),
                    });
                }
                if self.in_module_scope {
                    self.current_stmts.push(MirInst::DeleteGlobal { name: *sym_id });
                }
                self.sym_to_vreg.remove(sym_id);
            }
            HirLValue::Unpack { targets, .. } => {
                for target in targets {
                    self.lower_delete_lvalue(target);
                }
            }
        }
    }

    fn lower_stmt(&mut self, stmt: &HirStmt) {
        match stmt {
            HirStmt::Let { target, value, .. } => {
                let val_ty = value.ty();
                let val = self.lower_expr(value);
                // Always allocate a fresh VReg to avoid aliasing the
                // source.  Without this, `temp: int = b` shares the
                // same VReg as b, and a later `b = …` corrupts temp.
                let dest = self.fresh_vreg();
                self.current_stmts.push(MirInst::Copy { dest, source: val });
                self.sym_to_vreg.insert(*target, dest);
                // If this is a nonlocal-shared (Cell) variable, also store to global storage
                // so inner functions can observe mutations via LoadGlobal.
                if self.cell_override.contains(&target.0) {
                    // NaN-box primitives so LoadGlobal in the inner function sees a proper
                    // MbValue. Matches the param cell-store path — keeps the storage
                    // convention uniform across param / Let / Assign entry points.
                    let boxed = self.box_operand(dest, val_ty);
                    self.current_stmts.push(MirInst::StoreGlobal {
                        name: *target,
                        value: boxed,
                    });
                } else if self.in_module_scope {
                    // At module scope, mirror the Local assignment to global
                    // storage so functions reading via LoadGlobal get a
                    // proper NaN-boxed MbValue. Without boxing primitives,
                    // `v = f()` (int) stored raw bits that LoadGlobal then
                    // read back as an MbValue — float garbage on print.
                    let boxed = self.box_operand(dest, val_ty);
                    self.current_stmts.push(MirInst::StoreGlobal {
                        name: *target,
                        value: boxed,
                    });
                }
            }
            HirStmt::Assign { target, value, .. } => {
                let val_ty = value.ty();
                let val = self.lower_expr(value);
                match target {
                    HirLValue::Var(sym) => {
                        // Check cell_override first: synthetic (1M+) nonlocal-shared symbols
                        // always use global storage regardless of symbol_table classification.
                        if self.cell_override.contains(&sym.0) {
                            // NaN-box primitive values so inner functions see a proper MbValue
                            // via LoadGlobal. Matches the param cell-store path (line 684) so
                            // the storage convention is uniform regardless of whether the cell
                            // was first written by a param or an assignment.
                            let boxed = self.box_operand(val, val_ty);
                            self.current_stmts.push(MirInst::StoreGlobal {
                                name: *sym,
                                value: boxed,
                            });
                        } else {
                            let var_class = self
                                .symbol_table
                                .map(|st| st.get_var_class(*sym))
                                .unwrap_or(VariableClass::Local);
                            if var_class == VariableClass::Global {
                                self.current_stmts.push(MirInst::StoreGlobal {
                                    name: *sym,
                                    value: val,
                                });
                            } else if var_class == VariableClass::Cell {
                                // Cell variables are captured by inner functions — use global
                                // storage so mutations are visible to inner function reads.
                                self.current_stmts.push(MirInst::StoreGlobal {
                                    name: *sym,
                                    value: val,
                                });
                            } else if var_class == VariableClass::Free {
                                // Free variables write back through the outer Cell symbol.
                                let outer_sym =
                                    self.symbol_table.and_then(|st| st.get_nonlocal_outer(*sym));
                                if let Some(outer) = outer_sym {
                                    self.current_stmts.push(MirInst::StoreGlobal {
                                        name: outer,
                                        value: val,
                                    });
                                } else if let Some(&orig_vreg) = self.sym_to_vreg.get(sym) {
                                    self.current_stmts.push(MirInst::Copy {
                                        dest: orig_vreg,
                                        source: val,
                                    });
                                } else {
                                    self.sym_to_vreg.insert(*sym, val);
                                }
                            } else if let Some(&orig_vreg) = self.sym_to_vreg.get(sym) {
                                // Write back to the ORIGINAL VReg so Cranelift's
                                // SSA builder inserts proper phi nodes across
                                // loop back-edges.
                                self.current_stmts.push(MirInst::Copy {
                                    dest: orig_vreg,
                                    source: val,
                                });
                                if self.in_module_scope {
                                    let boxed = self.box_operand(orig_vreg, val_ty);
                                    self.current_stmts.push(MirInst::StoreGlobal {
                                        name: *sym,
                                        value: boxed,
                                    });
                                }
                            } else {
                                // First assignment — treat as definition.
                                // Fresh VReg + Copy (prevents aliasing the source;
                                // matches the HirStmt::Let path).
                                let dest = self.fresh_vreg();
                                self.current_stmts.push(MirInst::Copy { dest, source: val });
                                self.sym_to_vreg.insert(*sym, dest);
                                if self.in_module_scope {
                                    let boxed = self.box_operand(dest, val_ty);
                                    self.current_stmts.push(MirInst::StoreGlobal {
                                        name: *sym,
                                        value: boxed,
                                    });
                                }
                            }
                        } // close cell_override else branch
                    }
                    HirLValue::Attr { object, attr } => {
                        let obj = self.lower_expr(object);
                        // Box primitives for runtime setattr (instance fields store MbValues)
                        let boxed_val = self.box_operand(val, val_ty);
                        self.current_stmts.push(MirInst::SetAttr {
                            object: obj,
                            attr: attr.clone(),
                            value: boxed_val,
                        });
                    }
                    HirLValue::Index { object, index } => {
                        let obj = self.lower_expr(object);
                        let idx_ty = index.ty();
                        let idx = self.lower_expr(index);
                        // Box primitives for runtime setitem (dict/list store MbValues)
                        let boxed_idx = self.box_operand(idx, idx_ty);
                        let boxed_val = self.box_operand(val, val_ty);
                        self.current_stmts.push(MirInst::SetItem {
                            object: obj,
                            index: boxed_idx,
                            value: boxed_val,
                        });
                    }
                    HirLValue::Unpack {
                        targets,
                        star_index,
                    } => {
                        self.lower_unpack_assign(val, targets, *star_index);
                    }
                }
            }
            HirStmt::Return { value, .. } => {
                if let Some(coro_handle) = self.async_coro_vreg {
                    // Async function: complete coroutine with return value.
                    // Box the value so mb_run_until_complete returns a proper
                    // NaN-boxed MbValue rather than a raw primitive.
                    let ret_val = if let Some(v) = value {
                        let raw = self.lower_expr(v);
                        self.box_operand(raw, v.ty())
                    } else {
                        self.emit_none()
                    };
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: None,
                        name: "mb_coroutine_complete".to_string(),
                        args: vec![coro_handle, ret_val],
                        ty: self.tcx.none(),
                    });
                    self.finish_block(Terminator::Return(Some(coro_handle)));
                } else if self.is_gen_body {
                    // Generator body: box the return value so it arrives as
                    // a proper NaN-boxed MbValue through the channel.
                    let ret_vreg = if let Some(v) = value {
                        let raw = self.lower_expr(v);
                        self.box_operand(raw, v.ty())
                    } else {
                        self.emit_none()
                    };
                    self.finish_block(Terminator::Return(Some(ret_vreg)));
                } else if self.is_class_method {
                    // R4 P1: Class method body — box the return value so
                    // mb_call_method (dynamic dispatch) receives a proper
                    // NaN-boxed MbValue instead of a raw primitive.
                    let ret_vreg = if let Some(v) = value {
                        let raw = self.lower_expr(v);
                        self.box_operand(raw, v.ty())
                    } else {
                        self.emit_none()
                    };
                    // `return <expr>` where <expr> raised must route through the
                    // enclosing try handler (same as the plain-function path
                    // below). Without this, a method's `return d[k]` inside a
                    // nested try escaped the except clause. Also inline any
                    // pending finally / with-exit before returning.
                    if let Some(&(handler_block, _)) = self.try_handler_stack.last() {
                        let has_exc = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(has_exc),
                            name: "mb_has_exception".to_string(),
                            args: vec![],
                            ty: self.tcx.bool(),
                        });
                        let return_block = self.fresh_block();
                        self.finish_block(Terminator::Branch {
                            cond: has_exc,
                            then_block: handler_block,
                            else_block: return_block,
                        });
                        self.start_block(return_block);
                    }
                    if !self.finally_body_stack.is_empty() {
                        let pending_finally: Vec<Vec<crate::hir::HirStmt>> = self
                            .finally_body_stack
                            .iter()
                            .rev()
                            .filter(|f| !f.is_empty())
                            .cloned()
                            .collect();
                        if !pending_finally.is_empty() {
                            let handler_count = self.try_handler_stack.len();
                            for _ in 0..handler_count {
                                self.emit_extern_call(None, "mb_pop_handler");
                            }
                            let saved_finally = std::mem::take(&mut self.finally_body_stack);
                            let saved_try = std::mem::take(&mut self.try_handler_stack);
                            for finally_stmts in &pending_finally {
                                for s in finally_stmts {
                                    self.lower_stmt(s);
                                }
                            }
                            self.finally_body_stack = saved_finally;
                            self.try_handler_stack = saved_try;
                        }
                    }
                    if !self.with_ctx_stack.is_empty()
                        && self.finally_body_stack.is_empty()
                        && self.try_handler_stack.is_empty()
                    {
                        let none_vreg = self.emit_none();
                        let ctxs: Vec<VReg> =
                            self.with_ctx_stack.iter().rev().copied().collect();
                        for ctx in ctxs {
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: None,
                                name: "mb_context_exit".to_string(),
                                args: vec![ctx, none_vreg],
                                ty: self.tcx.any(),
                            });
                        }
                    }
                    self.finish_block(Terminator::Return(Some(ret_vreg)));
                } else {
                    let ret_vreg = value.as_ref().map(|v| {
                        self.lower_expr(v)
                        // Do NOT unbox the return value based on the declared
                        // return type.  The declared type may be int_ty (the
                        // default for unannotated functions) while the runtime
                        // value is a NaN-boxed string, list, etc.  Unboxing
                        // would destroy non-int NaN-boxed values (mb_unbox_int
                        // on a string pointer → 0).
                        // Instead, callers use mb_box_int which already guards
                        // against double-boxing: NaN-boxed values pass through
                        // unchanged, raw ints get properly NaN-boxed.
                    });
                    // `return <expr>` where <expr> raised — route through the
                    // enclosing try handler instead of returning so try/except
                    // catches the exception. Without this, `return int("abc")`
                    // inside a try silently returns None and the exception
                    // falls out past the except clause.
                    if let Some(&(handler_block, _)) = self.try_handler_stack.last() {
                        let has_exc = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(has_exc),
                            name: "mb_has_exception".to_string(),
                            args: vec![],
                            ty: self.tcx.bool(),
                        });
                        let return_block = self.fresh_block();
                        self.finish_block(Terminator::Branch {
                            cond: has_exc,
                            then_block: handler_block,
                            else_block: return_block,
                        });
                        self.start_block(return_block);
                    }
                    // If inside try blocks with non-empty finally bodies, inline them
                    // before returning so `finally` always runs on early exit.
                    if !self.finally_body_stack.is_empty() {
                        let pending_finally: Vec<Vec<crate::hir::HirStmt>> = self
                            .finally_body_stack
                            .iter()
                            .rev()
                            .filter(|f| !f.is_empty())
                            .cloned()
                            .collect();
                        if !pending_finally.is_empty() {
                            // Pop exception handlers for all try blocks we're exiting.
                            let handler_count = self.try_handler_stack.len();
                            for _ in 0..handler_count {
                                self.emit_extern_call(None, "mb_pop_handler");
                            }
                            // Temporarily clear stacks to prevent infinite recursion if
                            // the finally body itself contains a return statement.
                            let saved_finally = std::mem::take(&mut self.finally_body_stack);
                            let saved_try = std::mem::take(&mut self.try_handler_stack);
                            for finally_stmts in &pending_finally {
                                for s in finally_stmts {
                                    self.lower_stmt(s);
                                }
                            }
                            self.finally_body_stack = saved_finally;
                            self.try_handler_stack = saved_try;
                        }
                    }
                    // A `return` inside a `with` must still run each context
                    // manager's __exit__ before leaving the function (CPython:
                    // files closed / locks released on early return). Emit the
                    // exits inline in reverse (LIFO). Conservatively scoped to
                    // the no-enclosing-try/finally case so the __exit__/finally
                    // ordering can't be gotten wrong; the exception path already
                    // routes raises to the with's exit block.
                    if !self.with_ctx_stack.is_empty()
                        && self.finally_body_stack.is_empty()
                        && self.try_handler_stack.is_empty()
                    {
                        let none_vreg = self.emit_none();
                        let ctxs: Vec<VReg> = self.with_ctx_stack.iter().rev().copied().collect();
                        for ctx in ctxs {
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: None,
                                name: "mb_context_exit".to_string(),
                                args: vec![ctx, none_vreg],
                                ty: self.tcx.any(),
                            });
                        }
                    }
                    self.finish_block(Terminator::Return(ret_vreg));
                }
                // Start a dead block for any unreachable code after return
                let dead = self.fresh_block();
                self.start_block(dead);
            }
            HirStmt::Expr { expr, .. } => {
                let _ = self.lower_expr(expr);
            }
            HirStmt::If {
                cond,
                then_body,
                else_body,
                ..
            } => {
                self.lower_if(cond, then_body, else_body);
            }
            HirStmt::While {
                cond,
                body,
                else_body,
                ..
            } => {
                self.lower_while(cond, body, else_body);
            }
            HirStmt::For {
                var,
                iter,
                body,
                else_body,
                ..
            } => {
                self.lower_for(*var, iter, body, else_body);
            }
            HirStmt::Break { .. } => {
                if let Some(exit) = self.loop_exit {
                    // If inside try blocks with finally bodies, inline them before break.
                    if !self.finally_body_stack.is_empty() {
                        let pending_finally: Vec<Vec<crate::hir::HirStmt>> = self
                            .finally_body_stack
                            .iter()
                            .rev()
                            .filter(|f| !f.is_empty())
                            .cloned()
                            .collect();
                        if !pending_finally.is_empty() {
                            let handler_count = self.try_handler_stack.len();
                            for _ in 0..handler_count {
                                self.emit_extern_call(None, "mb_pop_handler");
                            }
                            let saved_finally = std::mem::take(&mut self.finally_body_stack);
                            let saved_try = std::mem::take(&mut self.try_handler_stack);
                            for finally_stmts in &pending_finally {
                                for s in finally_stmts {
                                    self.lower_stmt(s);
                                }
                            }
                            self.finally_body_stack = saved_finally;
                            self.try_handler_stack = saved_try;
                        }
                    }
                    self.finish_block(Terminator::Goto(exit));
                    let dead = self.fresh_block();
                    self.start_block(dead);
                }
            }
            HirStmt::Continue { .. } => {
                if let Some(header) = self.loop_header {
                    // If inside try blocks with finally bodies, inline them before continue.
                    if !self.finally_body_stack.is_empty() {
                        let pending_finally: Vec<Vec<crate::hir::HirStmt>> = self
                            .finally_body_stack
                            .iter()
                            .rev()
                            .filter(|f| !f.is_empty())
                            .cloned()
                            .collect();
                        if !pending_finally.is_empty() {
                            let handler_count = self.try_handler_stack.len();
                            for _ in 0..handler_count {
                                self.emit_extern_call(None, "mb_pop_handler");
                            }
                            let saved_finally = std::mem::take(&mut self.finally_body_stack);
                            let saved_try = std::mem::take(&mut self.try_handler_stack);
                            for finally_stmts in &pending_finally {
                                for s in finally_stmts {
                                    self.lower_stmt(s);
                                }
                            }
                            self.finally_body_stack = saved_finally;
                            self.try_handler_stack = saved_try;
                        }
                    }
                    self.finish_block(Terminator::Goto(header));
                    let dead = self.fresh_block();
                    self.start_block(dead);
                }
            }
            HirStmt::Try {
                body,
                handlers,
                else_body,
                finally_body,
                ..
            } => {
                let has_star = handlers.iter().any(|h| h.is_star);
                let handler_block = self.fresh_block();
                let finally_block = self.fresh_block();
                let merge_block = self.fresh_block();
                // Push exception handler via runtime
                let catch_all = handlers.iter().any(|h| h.exc_type.is_none());
                let catch_all_vreg = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest: catch_all_vreg,
                    value: MirConst::Bool(catch_all || has_star),
                    ty: self.tcx.bool(),
                });
                self.current_stmts.push(MirInst::CallExtern {
                    dest: None,
                    name: "mb_push_handler".to_string(),
                    args: vec![catch_all_vreg],
                    ty: self.tcx.none(),
                });
                // Try body — after each statement, check for pending exceptions
                // so that exceptions propagate at statement boundaries (matching Python).
                self.try_handler_stack.push((handler_block, finally_block));
                self.finally_body_stack.push(finally_body.clone());
                for s in body {
                    self.lower_stmt(s);
                    self.emit_try_exception_guard();
                }
                self.try_handler_stack.pop();
                self.finally_body_stack.pop();
                self.emit_extern_call(None, "mb_pop_handler");
                // Check for exception
                let exc_check = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(exc_check),
                    name: "mb_has_exception".to_string(),
                    args: Vec::new(),
                    ty: self.tcx.bool(),
                });
                let no_exc_target = if !else_body.is_empty() {
                    self.fresh_block()
                } else {
                    finally_block
                };
                self.finish_block(Terminator::Branch {
                    cond: exc_check,
                    then_block: handler_block,
                    else_block: no_exc_target,
                });
                // Else block (only when no exception)
                if !else_body.is_empty() {
                    self.start_block(no_exc_target);
                    for s in else_body {
                        self.lower_stmt(s);
                    }
                    self.finish_block(Terminator::Goto(finally_block));
                }
                // Handler block: catch exception, match against handlers
                self.start_block(handler_block);
                let caught_exc = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(caught_exc),
                    name: "mb_catch_exception_instance".to_string(),
                    args: Vec::new(),
                    ty: self.tcx.any(),
                });

                if has_star {
                    // except* lowering (PEP 654):
                    // The caught exception is an ExceptionGroup. For each handler,
                    // split the group by type, run the handler body with the matched
                    // sub-group, then continue with the rest.
                    let mut current_group = caught_exc;
                    for h in handlers.iter() {
                        if let Some(ref exc_type_expr) = h.exc_type {
                            let type_vreg = self.lower_expr(exc_type_expr);
                            // Split: (matched_group_or_none, rest_group_or_none)
                            let split_result = self.fresh_vreg();
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(split_result),
                                name: "mb_except_star".to_string(),
                                args: vec![current_group, type_vreg],
                                ty: self.tcx.any(),
                            });
                            // matched = tuple[0]
                            let idx0_raw = self.emit_int_const(0);
                            let idx0 = self.box_operand(idx0_raw, self.tcx.int());
                            let matched = self.fresh_vreg();
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(matched),
                                name: "mb_tuple_getitem".to_string(),
                                args: vec![split_result, idx0],
                                ty: self.tcx.any(),
                            });
                            // rest = tuple[1]
                            let idx1_raw = self.emit_int_const(1);
                            let idx1 = self.box_operand(idx1_raw, self.tcx.int());
                            let rest = self.fresh_vreg();
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(rest),
                                name: "mb_tuple_getitem".to_string(),
                                args: vec![split_result, idx1],
                                ty: self.tcx.any(),
                            });
                            // Check if matched is not None
                            let is_not_none = self.fresh_vreg();
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(is_not_none),
                                name: "mb_is_not_none".to_string(),
                                args: vec![matched],
                                ty: self.tcx.bool(),
                            });
                            let body_block = self.fresh_block();
                            let skip_block = self.fresh_block();
                            self.finish_block(Terminator::Branch {
                                cond: is_not_none,
                                then_block: body_block,
                                else_block: skip_block,
                            });
                            // Handler body block
                            self.start_block(body_block);
                            if let Some(name_sym) = h.name {
                                self.sym_to_vreg.insert(name_sym, matched);
                            }
                            let prev_active = self.active_except_vreg;
                            self.active_except_vreg = Some(matched);
                            for s in &h.body {
                                self.lower_stmt(s);
                            }
                            self.active_except_vreg = prev_active;
                            self.finish_block(Terminator::Goto(skip_block));
                            // Continue with the rest
                            self.start_block(skip_block);
                            current_group = rest;
                        }
                    }
                    // After all handlers, if rest is not None, re-raise
                    let rest_not_none = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(rest_not_none),
                        name: "mb_is_not_none".to_string(),
                        args: vec![current_group],
                        ty: self.tcx.bool(),
                    });
                    let reraise_block = self.fresh_block();
                    self.finish_block(Terminator::Branch {
                        cond: rest_not_none,
                        then_block: reraise_block,
                        else_block: finally_block,
                    });
                    self.start_block(reraise_block);
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: None,
                        name: "mb_reraise".to_string(),
                        args: vec![current_group],
                        ty: self.tcx.none(),
                    });
                    self.finish_block(Terminator::Goto(finally_block));
                } else {
                    // Regular except lowering
                    let mut next_handler_blocks = Vec::new();
                    for (i, h) in handlers.iter().enumerate() {
                        if i > 0 {
                            let blk = next_handler_blocks.last().copied().unwrap();
                            self.start_block(blk);
                        }
                        if let Some(ref exc_type_expr) = h.exc_type {
                            let type_vreg = self.lower_expr(exc_type_expr);
                            let match_result = self.fresh_vreg();
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(match_result),
                                name: "mb_exception_matches".to_string(),
                                args: vec![caught_exc, type_vreg],
                                ty: self.tcx.bool(),
                            });
                            let handler_body_block = self.fresh_block();
                            let next_check = if i + 1 < handlers.len() {
                                let nb = self.fresh_block();
                                next_handler_blocks.push(nb);
                                nb
                            } else {
                                let reraise_block = self.fresh_block();
                                let saved_stmts = std::mem::take(&mut self.current_stmts);
                                let saved_block = self.current_block_id;
                                self.start_block(reraise_block);
                                self.current_stmts.push(MirInst::CallExtern {
                                    dest: None,
                                    name: "mb_reraise".to_string(),
                                    args: vec![caught_exc],
                                    ty: self.tcx.none(),
                                });
                                self.finish_block(Terminator::Goto(finally_block));
                                self.current_block_id = saved_block;
                                self.current_stmts = saved_stmts;
                                reraise_block
                            };
                            self.finish_block(Terminator::Branch {
                                cond: match_result,
                                then_block: handler_body_block,
                                else_block: next_check,
                            });
                            self.start_block(handler_body_block);
                        }
                        if let Some(name_sym) = h.name {
                            self.sym_to_vreg.insert(name_sym, caught_exc);
                        }
                        let prev_active = self.active_except_vreg;
                        self.active_except_vreg = Some(caught_exc);
                        for s in &h.body {
                            self.lower_stmt(s);
                        }
                        self.active_except_vreg = prev_active;
                        self.finish_block(Terminator::Goto(finally_block));
                    }
                    if handlers.is_empty() {
                        // try/finally with no except: re-raise after finally
                        // so the exception propagates to the caller.
                        let reraise_finally = self.fresh_block();
                        self.finish_block(Terminator::Goto(reraise_finally));
                        self.start_block(reraise_finally);
                        for s in finally_body.iter() {
                            self.lower_stmt(s);
                        }
                        // Re-raise the caught exception
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: None,
                            name: "mb_reraise".to_string(),
                            args: vec![caught_exc],
                            ty: self.tcx.none(),
                        });
                        self.finish_block(Terminator::Goto(merge_block));
                    }
                }
                // Finally block (normal path — no exception)
                self.start_block(finally_block);
                for s in finally_body {
                    self.lower_stmt(s);
                }
                self.finish_block(Terminator::Goto(merge_block));
                self.start_block(merge_block);
                // except* can leave a reraised unmatched sub-group in CURRENT_EXCEPTION
                // after merge. Regular except paths either clear it via mb_catch or rely
                // on the enclosing try body's per-stmt guard to propagate, so we only
                // need the extra check for except*. Keeping this narrow avoids inflating
                // JIT code size past cranelift's ±64MB branch range on exception-heavy
                // modules.
                if has_star {
                    self.emit_exception_propagate();
                }
            }
            HirStmt::Raise { value, from, .. } => {
                let has_context = self.active_except_vreg.is_some();
                let has_from = from.is_some();
                // Emit the mb_raise* runtime call. After the call, control flow MUST
                // transfer immediately (no subsequent statement in the same function
                // may execute). We emit the call, then a terminator that either jumps
                // to an enclosing try handler or returns from the function.
                let mut raise_emitted = false;
                if let Some(value_expr) = value {
                    // Pattern: raise ExcType(msg)
                    if let HirExpr::Call { func, args, .. } = value_expr {
                        if let HirExpr::Var(sym, _) = func.as_ref() {
                            if let Some(class_name) = self.class_syms.get(&sym.0).cloned() {
                                let is_user_class = self.user_class_syms.contains(&sym.0);
                                if is_user_class {
                                    // User-defined class: create instance, call __init__, raise
                                    let type_vreg = self.emit_str_const(&class_name);
                                    let arg_vregs: Vec<VReg> = args
                                        .iter()
                                        .map(|a| {
                                            let v = self.lower_expr(a);
                                            self.box_operand(v, a.ty())
                                        })
                                        .collect();
                                    let args_list = self.fresh_vreg();
                                    self.current_stmts.push(MirInst::MakeList {
                                        dest: args_list,
                                        elements: arg_vregs,
                                        ty: self.tcx.any(),
                                    });
                                    let instance = self.fresh_vreg();
                                    self.current_stmts.push(MirInst::CallExtern {
                                        dest: Some(instance),
                                        name: "mb_instance_new_with_init".to_string(),
                                        args: vec![type_vreg, args_list],
                                        ty: self.tcx.any(),
                                    });
                                    // Choose raise function based on from/context
                                    if has_from && has_context {
                                        let from_val = {
                                            let v = self.lower_expr(from.as_ref().unwrap());
                                            self.box_operand(v, from.as_ref().unwrap().ty())
                                        };
                                        self.current_stmts.push(MirInst::CallExtern {
                                            dest: None,
                                            name: "mb_raise_instance_from_with_context".to_string(),
                                            args: vec![
                                                instance,
                                                from_val,
                                                self.active_except_vreg.unwrap(),
                                            ],
                                            ty: self.tcx.none(),
                                        });
                                    } else if has_from {
                                        let from_val = {
                                            let v = self.lower_expr(from.as_ref().unwrap());
                                            self.box_operand(v, from.as_ref().unwrap().ty())
                                        };
                                        self.current_stmts.push(MirInst::CallExtern {
                                            dest: None,
                                            name: "mb_raise_instance_from".to_string(),
                                            args: vec![instance, from_val],
                                            ty: self.tcx.none(),
                                        });
                                    } else if has_context {
                                        self.current_stmts.push(MirInst::CallExtern {
                                            dest: None,
                                            name: "mb_raise_instance_with_context".to_string(),
                                            args: vec![instance, self.active_except_vreg.unwrap()],
                                            ty: self.tcx.none(),
                                        });
                                    } else {
                                        self.current_stmts.push(MirInst::CallExtern {
                                            dest: None,
                                            name: "mb_raise_instance".to_string(),
                                            args: vec![instance],
                                            ty: self.tcx.none(),
                                        });
                                    }
                                    raise_emitted = true;
                                } else if class_name == "ExceptionGroup"
                                    || class_name == "BaseExceptionGroup" {
                                    // raise (Base)ExceptionGroup(...): pass ALL
                                    // positional args so the constructor can
                                    // validate the arity and argument types
                                    // (CPython raises TypeError/ValueError on bad
                                    // shape) before raising the group.
                                    let boxed: Vec<VReg> = args.iter().map(|a| {
                                        let v = self.lower_expr(a);
                                        self.box_operand(v, a.ty())
                                    }).collect();
                                    let args_list = self.fresh_vreg();
                                    self.current_stmts.push(MirInst::MakeList {
                                        dest: args_list,
                                        elements: boxed,
                                        ty: self.tcx.any(),
                                    });
                                    let cn_vreg = self.emit_str_const(&class_name);
                                    self.current_stmts.push(MirInst::CallExtern {
                                        dest: None,
                                        name: "mb_exception_group_construct_and_raise".to_string(),
                                        args: vec![args_list, cn_vreg],
                                        ty: self.tcx.none(),
                                    });
                                    raise_emitted = true;
                                } else if args.len() > 1 && !has_from {
                                    // Built-in exception with multiple args: use
                                    // mb_exception_new_with_args + mb_raise_instance so
                                    // e.args preserves all constructor arguments.
                                    let type_vreg = self.emit_str_const(&class_name);
                                    let arg_vregs: Vec<VReg> = args
                                        .iter()
                                        .map(|a| {
                                            let v = self.lower_expr(a);
                                            self.box_operand(v, a.ty())
                                        })
                                        .collect();
                                    let args_list = self.fresh_vreg();
                                    self.current_stmts.push(MirInst::MakeList {
                                        dest: args_list,
                                        elements: arg_vregs,
                                        ty: self.tcx.any(),
                                    });
                                    let instance = self.fresh_vreg();
                                    self.current_stmts.push(MirInst::CallExtern {
                                        dest: Some(instance),
                                        name: "mb_exception_new_with_args".to_string(),
                                        args: vec![type_vreg, args_list],
                                        ty: self.tcx.any(),
                                    });
                                    if has_context {
                                        self.current_stmts.push(MirInst::CallExtern {
                                            dest: None,
                                            name: "mb_raise_instance_with_context".to_string(),
                                            args: vec![instance, self.active_except_vreg.unwrap()],
                                            ty: self.tcx.none(),
                                        });
                                    } else {
                                        self.current_stmts.push(MirInst::CallExtern {
                                            dest: None,
                                            name: "mb_raise_instance".to_string(),
                                            args: vec![instance],
                                            ty: self.tcx.none(),
                                        });
                                    }
                                    raise_emitted = true;
                                } else {
                                    // Built-in exception type: use existing mb_raise path
                                    // (0 or 1 args, or raise-from which needs chaining support)
                                    let type_vreg = self.emit_str_const(&class_name);
                                    let msg_vreg = if let Some(arg) = args.first() {
                                        let v = self.lower_expr(arg);
                                        self.box_operand(v, arg.ty())
                                    } else {
                                        self.emit_str_const("")
                                    };
                                    // Choose raise function based on from/context
                                    let (raise_fn, mut raise_args) = if has_from && has_context {
                                        ("mb_raise_from_with_context", vec![type_vreg, msg_vreg])
                                    } else if has_from {
                                        ("mb_raise_from", vec![type_vreg, msg_vreg])
                                    } else if has_context {
                                        ("mb_raise_with_context", vec![type_vreg, msg_vreg])
                                    } else {
                                        ("mb_raise", vec![type_vreg, msg_vreg])
                                    };
                                    if let Some(from_expr) = from {
                                        let from_val = self.lower_expr(from_expr);
                                        let boxed = self.box_operand(from_val, from_expr.ty());
                                        raise_args.push(boxed);
                                    }
                                    if has_context {
                                        raise_args.push(self.active_except_vreg.unwrap());
                                    }
                                    self.current_stmts.push(MirInst::CallExtern {
                                        dest: None,
                                        name: raise_fn.to_string(),
                                        args: raise_args,
                                        ty: self.tcx.none(),
                                    });
                                    raise_emitted = true;
                                }
                            }
                        }
                    }
                    if !raise_emitted {
                        // Pattern: raise ExcType (bare, no call)
                        if let HirExpr::Var(sym, _) = value_expr {
                            if let Some(class_name) = self.class_syms.get(&sym.0).cloned() {
                                let type_vreg = self.emit_str_const(&class_name);
                                let msg_vreg = self.emit_str_const("");
                                let (raise_fn, mut raise_args) = if has_context {
                                    ("mb_raise_with_context", vec![type_vreg, msg_vreg])
                                } else {
                                    ("mb_raise", vec![type_vreg, msg_vreg])
                                };
                                if has_context {
                                    raise_args.push(self.active_except_vreg.unwrap());
                                }
                                self.current_stmts.push(MirInst::CallExtern {
                                    dest: None,
                                    name: raise_fn.to_string(),
                                    args: raise_args,
                                    ty: self.tcx.none(),
                                });
                                raise_emitted = true;
                            }
                        }
                    }
                    if !raise_emitted {
                        // Generic raise — raise an existing exception instance variable.
                        let val = self.lower_expr(value_expr);
                        let boxed = self.box_operand(val, value_expr.ty());
                        let (raise_fn, mut raise_args) = if has_context {
                            ("mb_raise_instance_with_context", vec![boxed])
                        } else {
                            ("mb_raise_instance", vec![boxed])
                        };
                        if has_context {
                            raise_args.push(self.active_except_vreg.unwrap());
                        }
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: None,
                            name: raise_fn.to_string(),
                            args: raise_args,
                            ty: self.tcx.none(),
                        });
                        raise_emitted = true;
                    }
                } else {
                    // Bare re-raise — restore current exception to CURRENT_EXCEPTION
                    // so the enclosing handler chain can propagate it.
                    if let Some(exc_vreg) = self.active_except_vreg {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: None,
                            name: "mb_reraise".to_string(),
                            args: vec![exc_vreg],
                            ty: self.tcx.none(),
                        });
                    } else {
                        self.current_stmts.push(MirInst::Raise { value: None });
                    }
                    raise_emitted = true;
                }
                // Raise must halt execution of the current block when we're at
                // the bare function body level — otherwise subsequent statements
                // in the same function continue to execute (which broke custom
                // iterator StopIteration dispatch in __next__).
                //
                // When inside a `try` (innermost), fall through to the try body's
                // per-statement exception guard, which handles finally / except
                // matching. When inside a `with` (innermost, no inner try), jump
                // to that with's exit block so `__exit__` runs (suppression /
                // re-raise) and the remaining body statements are skipped.
                if raise_emitted {
                    let inner_try_depth = self.try_handler_stack.len();
                    let with_exit = self.with_exit_stack.last().copied();
                    let with_is_inner = matches!(
                        with_exit, Some((_, td)) if td == inner_try_depth
                    );
                    if with_is_inner {
                        // Skip remaining body statements; route to __exit__.
                        let (exit_block, _) = with_exit.unwrap();
                        let dead_block = self.fresh_block();
                        self.finish_block(Terminator::Goto(exit_block));
                        self.start_block(dead_block);
                    } else if self.try_handler_stack.is_empty() {
                        let dead_block = self.fresh_block();
                        // Bare function body — return with None so the exception
                        // propagates to the caller (which will see mb_has_exception
                        // at its try check).
                        let none_vreg = self.emit_none();
                        self.finish_block(Terminator::Return(Some(none_vreg)));
                        self.start_block(dead_block);
                    }
                    // else: inside a try (innermost) — fall through to the guard.
                }
            }
            HirStmt::Import { import, .. } => {
                // Lower import as CallExtern to mb_import
                let mod_name = import.module.join(".");
                let name_vreg = self.emit_str_const(&mod_name);
                let dest = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(dest),
                    name: "mb_import".to_string(),
                    args: vec![name_vreg],
                    ty: self.tcx.any(),
                });

                if let Some(names) = &import.names {
                    // @spec .aw/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md#R4
                    // `from X import *` — delegate to mb_import_star which loads
                    // the module, respects __all__, and binds names into globals.
                    let is_star = names.len() == 1 && names[0].0 == "*";
                    if is_star {
                        let star_name_vreg = self.emit_str_const(&mod_name);
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: None,
                            name: "mb_import_star".to_string(),
                            args: vec![star_name_vreg],
                            ty: self.tcx.any(),
                        });
                    } else {
                        // `from X import Y, Z as W` (#1132 R3)
                        // For each imported name, extract its value from the module
                        // and store it in the global namespace for LoadGlobal access.
                        for (name, alias) in names {
                            let attr_vreg = self.emit_str_const(name);
                            let mod_name_vreg2 = self.emit_str_const(&mod_name);
                            let attr_dest = self.fresh_vreg();
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(attr_dest),
                                name: "mb_module_getattr".to_string(),
                                args: vec![mod_name_vreg2, attr_vreg],
                                ty: self.tcx.any(),
                            });
                            // The bound name is the alias if present, otherwise the original name.
                            let bound = alias.as_deref().unwrap_or(name.as_str());
                            if let Some(sym_id) = self.symbol_table.and_then(|st| st.lookup(bound))
                            {
                                self.sym_to_vreg.insert(sym_id, attr_dest);
                                // Also emit StoreGlobal so functions can read via LoadGlobal.
                                self.current_stmts.push(MirInst::StoreGlobal {
                                    name: sym_id,
                                    value: attr_dest,
                                });
                            }
                        }
                    }
                } else {
                    // Bare `import X` / `import X.Y.Z` / `import X.Y.Z as alias`.
                    // CPython binding rule (PEP 328 / language ref §7.11.1):
                    //   - `import a.b.c`        binds local name `a` to the
                    //                           TOP module `a`, not `a.b.c`.
                    //   - `import a.b.c as q`   binds local name `q` to the
                    //                           LEAF module `a.b.c`.
                    // Without the alias-form carve-out, both
                    //   `import email.message; import email.policy`
                    // would clobber the local `email` with whichever submodule
                    // was loaded last (test_email failure: `email.__name__`
                    // came back as "email.policy").
                    let (bound_name, bound_vreg) = if let Some(alias) = &import.module_alias {
                        // Aliased dotted import: bind alias to the leaf module.
                        (alias.clone(), dest)
                    } else if import.module.len() > 1 {
                        // Dotted import without alias: re-import the top-level
                        // package so the local name resolves to it. mb_import
                        // is idempotent — the leaf was already loaded above,
                        // and its sys.modules registration as an attribute of
                        // the parent is what makes `email.policy` reachable.
                        let top = import.module.first().cloned().unwrap_or_default();
                        let top_name_vreg = self.emit_str_const(&top);
                        let top_dest = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(top_dest),
                            name: "mb_import".to_string(),
                            args: vec![top_name_vreg],
                            ty: self.tcx.any(),
                        });
                        (top, top_dest)
                    } else {
                        // Single-segment import: bind the only name to the module.
                        (import.module.first().cloned().unwrap_or_default(), dest)
                    };
                    if !bound_name.is_empty() {
                        if let Some(sym_id) =
                            self.symbol_table.and_then(|st| st.lookup(&bound_name))
                        {
                            self.sym_to_vreg.insert(sym_id, bound_vreg);
                            if self.in_module_scope {
                                self.current_stmts.push(MirInst::StoreGlobal {
                                    name: sym_id,
                                    value: bound_vreg,
                                });
                            }
                        }
                    }
                }
            }
            HirStmt::With {
                items,
                body,
                is_async,
                ..
            } => {
                // Desugar: with ctx as var → enter, execute body, exit.
                //
                // CPython semantics: if __enter__ raises, the body is NOT executed
                // and __exit__ is NOT called — the exception propagates. We check
                // mb_has_exception() after each enter and branch to the enclosing
                // try handler (if any) before running the body.
                let (enter_helper, exit_helper) = if *is_async {
                    ("mb_async_context_enter", "mb_async_context_exit")
                } else {
                    ("mb_context_enter", "mb_context_exit")
                };
                let mut ctx_vregs = Vec::new();
                for (ctx, alias) in items {
                    let ctx_vreg = self.lower_expr(ctx);
                    // Call __enter__ (or __aenter__ for async with) and bind to alias
                    let enter_dest = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(enter_dest),
                        name: enter_helper.to_string(),
                        args: vec![ctx_vreg],
                        ty: self.tcx.any(),
                    });
                    // Exception check after __enter__: if raised, skip body + exit
                    // and propagate to outer try handler.
                    if let Some(&(handler_block, _)) = self.try_handler_stack.last() {
                        let exc_check = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(exc_check),
                            name: "mb_has_exception".to_string(),
                            args: Vec::new(),
                            ty: self.tcx.bool(),
                        });
                        let continue_block = self.fresh_block();
                        let exc_block = self.fresh_block();
                        self.finish_block(Terminator::Branch {
                            cond: exc_check,
                            then_block: exc_block,
                            else_block: continue_block,
                        });
                        self.start_block(exc_block);
                        self.emit_extern_call(None, "mb_pop_handler");
                        self.finish_block(Terminator::Goto(handler_block));
                        self.start_block(continue_block);
                    }
                    if let Some(sym) = alias {
                        // Mirror the Assign handler's variable-class dispatch so that
                        // with...as bindings are visible at the correct scope.
                        if self.cell_override.contains(&sym.0) {
                            self.current_stmts.push(MirInst::StoreGlobal {
                                name: *sym,
                                value: enter_dest,
                            });
                        } else {
                            let var_class = self
                                .symbol_table
                                .map(|st| st.get_var_class(*sym))
                                .unwrap_or(VariableClass::Local);
                            if var_class == VariableClass::Global {
                                self.current_stmts.push(MirInst::StoreGlobal {
                                    name: *sym,
                                    value: enter_dest,
                                });
                            } else if var_class == VariableClass::Cell {
                                // Cell variables are captured by inner functions — use global
                                // storage so mutations are visible to inner function reads.
                                self.current_stmts.push(MirInst::StoreGlobal {
                                    name: *sym,
                                    value: enter_dest,
                                });
                            } else if var_class == VariableClass::Free {
                                // Free variables write back through the outer Cell symbol.
                                let outer_sym =
                                    self.symbol_table.and_then(|st| st.get_nonlocal_outer(*sym));
                                if let Some(outer) = outer_sym {
                                    self.current_stmts.push(MirInst::StoreGlobal {
                                        name: outer,
                                        value: enter_dest,
                                    });
                                } else if let Some(&orig) = self.sym_to_vreg.get(sym) {
                                    self.current_stmts.push(MirInst::Copy {
                                        dest: orig,
                                        source: enter_dest,
                                    });
                                } else {
                                    self.sym_to_vreg.insert(*sym, enter_dest);
                                }
                            } else if let Some(&orig) = self.sym_to_vreg.get(sym) {
                                self.current_stmts.push(MirInst::Copy {
                                    dest: orig,
                                    source: enter_dest,
                                });
                                if self.in_module_scope {
                                    self.current_stmts.push(MirInst::StoreGlobal {
                                        name: *sym,
                                        value: orig,
                                    });
                                }
                            } else {
                                self.sym_to_vreg.insert(*sym, enter_dest);
                                if self.in_module_scope {
                                    self.current_stmts.push(MirInst::StoreGlobal {
                                        name: *sym,
                                        value: enter_dest,
                                    });
                                }
                            }
                        }
                    }
                    ctx_vregs.push(ctx_vreg);
                }
                // Dedicated exit block: BOTH the normal end-of-body path and the
                // exception-propagation path (a call inside the body that raised)
                // converge here so `__exit__` always runs — that is what gives
                // `with` its suppression / re-raise semantics. Without this, a
                // method call that raised inside the body would return straight
                // out of the function (via emit_exception_propagate) and skip
                // `__exit__` entirely, so `assertRaises`/suppressing context
                // managers never got a chance to swallow the exception.
                let exit_block = self.fresh_block();
                let after_block = self.fresh_block();
                // Push context managers onto the with stack so that `raise`
                // inside the body can call their __exit__ before returning, and
                // register the exit block so call-site exception propagation
                // routes here. `with_exit_stack` records the try-depth at entry
                // so an inner `try` (pushed later) still takes precedence.
                let initial_with_depth = self.with_ctx_stack.len();
                for &ctx in &ctx_vregs {
                    self.with_ctx_stack.push(ctx);
                }
                self.with_exit_stack
                    .push((exit_block, self.try_handler_stack.len()));
                // Execute body
                for s in body {
                    self.lower_stmt(s);
                }
                // Pop this with's bookkeeping off the stacks (they only cover the
                // body lowering above).
                self.with_exit_stack.pop();
                self.with_ctx_stack.truncate(initial_with_depth);
                // Normal end-of-body falls through to the shared exit block.
                self.finish_block(Terminator::Goto(exit_block));

                // Exit block: run __exit__ for each ctx (reverse order), then —
                // since __exit__ may have suppressed (cleared) or re-raised the
                // exception — re-check and propagate to the enclosing try /
                // function, else continue past the `with`.
                self.start_block(exit_block);
                let none_vreg = self.emit_none();
                for &ctx_vreg in ctx_vregs.iter().rev() {
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: None,
                        name: exit_helper.to_string(),
                        args: vec![ctx_vreg, none_vreg],
                        ty: self.tcx.any(),
                    });
                }
                let exc_after = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(exc_after),
                    name: "mb_has_exception".to_string(),
                    args: Vec::new(),
                    ty: self.tcx.bool(),
                });
                let propagate_block = self.fresh_block();
                self.finish_block(Terminator::Branch {
                    cond: exc_after,
                    then_block: propagate_block,
                    else_block: after_block,
                });
                self.start_block(propagate_block);
                // Match emit_exception_propagate's precedence exactly: an
                // enclosing `with` whose recorded try-depth equals the current
                // try-handler depth (i.e. no `try` was pushed between that outer
                // `with` and here) takes precedence so its `__exit__` runs before
                // unwinding further. Only otherwise fall back to the innermost
                // `try` handler, then to returning None from the function.
                //
                // Note: this with's own bookkeeping was already popped off
                // with_exit_stack above, so `last()` here is the *enclosing*
                // with, which is exactly the one we want to route to.
                let outer_with = self.with_exit_stack.last().copied();
                let cur_try_depth = self.try_handler_stack.len();
                if let Some((outer_exit, try_depth)) = outer_with {
                    if try_depth == cur_try_depth {
                        // Nested `with` with no intervening `try`: propagate to
                        // the enclosing with's exit so its __exit__ runs.
                        self.finish_block(Terminator::Goto(outer_exit));
                        self.start_block(after_block);
                        return;
                    }
                }
                if let Some(&(handler_block, _)) = self.try_handler_stack.last() {
                    self.emit_extern_call(None, "mb_pop_handler");
                    self.finish_block(Terminator::Goto(handler_block));
                } else {
                    let ret_none = self.emit_none();
                    self.finish_block(Terminator::Return(Some(ret_none)));
                }
                self.start_block(after_block);
            }
            HirStmt::Assert { test, msg, .. } => {
                // Use the same truthiness conversion as `if` (mb_is_truthy for
                // heap-backed / `any` values). Without it, `assert <obj>` branched
                // on the raw NaN-boxed pointer (LSB always 0), so a truthy heap
                // object like an re.Match wrongly failed the assert even though
                // `if obj:` / `bool(obj)` reported True.
                let test_vreg = self.lower_cond_as_bool(test);
                let assert_block = self.fresh_block();
                let pass_block = self.fresh_block();
                self.finish_block(Terminator::Branch {
                    cond: test_vreg,
                    then_block: pass_block,
                    else_block: assert_block,
                });
                self.start_block(assert_block);
                if let Some(m) = msg {
                    let msg_vreg = self.lower_expr(m);
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: None,
                        name: "mb_assertion_error".to_string(),
                        args: vec![msg_vreg],
                        ty: self.tcx.none(),
                    });
                } else {
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: None,
                        name: "mb_assertion_error_no_msg".to_string(),
                        args: vec![],
                        ty: self.tcx.none(),
                    });
                }
                // After setting the exception, halt execution like `raise`
                // (mirror the raise routing precedence above):
                // - Innermost `with` (no inner try): jump to that with's exit
                //   block so `__exit__` runs (suppression / re-raise) before
                //   unwinding. Without this a failed assert inside a `with` body
                //   skipped __exit__ entirely.
                // - Bare function level (no try, no with): return None so the
                //   exception propagates to the caller's exception guard.
                // - Inside a `try` (innermost): fall through to the per-statement
                //   try guard, which handles finally / except matching.
                let inner_try_depth = self.try_handler_stack.len();
                let with_exit = self.with_exit_stack.last().copied();
                let with_is_inner = matches!(
                    with_exit, Some((_, td)) if td == inner_try_depth
                );
                if with_is_inner {
                    let (exit_block, _) = with_exit.unwrap();
                    self.finish_block(Terminator::Goto(exit_block));
                } else if self.try_handler_stack.is_empty() {
                    let none_vreg = self.emit_none();
                    self.finish_block(Terminator::Return(Some(none_vreg)));
                } else {
                    // Inside a try (innermost) — fall through to the guard.
                    self.finish_block(Terminator::Goto(pass_block));
                }
                self.start_block(pass_block);
            }
            HirStmt::Del { target, .. } => self.lower_delete_lvalue(target),
            HirStmt::Global { .. } | HirStmt::Nonlocal { .. } => {
                // Scope declarations — no MIR instructions needed
            }
            HirStmt::FuncDefPlaceholder { name: func_sym, redef, .. } => {
                // Register the function's __name__ so `f.__name__` works.
                {
                    let any_ty = self.tcx.any();
                    let fn_vreg = self.fresh_vreg();
                    self.current_stmts.push(MirInst::LoadConst {
                        dest: fn_vreg,
                        value: MirConst::FuncRef(*func_sym),
                        ty: any_ty,
                    });
                    // Look up the symbol's name via sym_name_lookup
                    if let Some(fname) = self.symbol_table.and_then(|st| {
                        if func_sym.0 < 1_000_000 {
                            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                st.get_symbol(*func_sym).name.clone()
                            }))
                            .ok()
                        } else {
                            None
                        }
                    }) {
                        let name_vreg = self.emit_str_const(&fname);
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: None,
                            name: "mb_func_set_name".to_string(),
                            args: vec![fn_vreg, name_vreg],
                            ty: self.tcx.none(),
                        });
                    }
                }
                // Apply decorator(s) to the function at this point in execution order.
                // Python semantics: `@dec def foo()` ≡ `def foo(); foo = dec(foo)`.
                // Pull the decorators from pending_decorators (if any).
                let decorators = {
                    let pos = self
                        .pending_decorators
                        .iter()
                        .position(|(s, _)| s == func_sym);
                    pos.map(|i| self.pending_decorators.remove(i))
                        .map(|(_, d)| d)
                };
                if let Some(decorators) = decorators {
                    let any_ty = self.tcx.any();
                    let mut func_vreg = self.fresh_vreg();
                    self.current_stmts.push(MirInst::LoadConst {
                        dest: func_vreg,
                        value: MirConst::FuncRef(*func_sym),
                        ty: any_ty,
                    });
                    for dec_expr in decorators.iter().rev() {
                        let result_vreg = self.fresh_vreg();
                        match dec_expr {
                            HirExpr::Var(dec_sym, _)
                                if !self.user_class_syms.contains(&dec_sym.0)
                                    && self.class_syms
                                        .get(&dec_sym.0)
                                        .map(|name| {
                                            matches!(
                                                name.as_str(),
                                                "property" | "staticmethod" | "classmethod"
                                            )
                                        })
                                        .unwrap_or(false) =>
                            {
                                let class_name = self.class_syms.get(&dec_sym.0).cloned()
                                    .unwrap_or_default();
                                let extern_name = match class_name.as_str() {
                                    "property" => "mb_property_new",
                                    "staticmethod" => "mb_staticmethod_new",
                                    "classmethod" => "mb_classmethod_new",
                                    _ => unreachable!(),
                                };
                                self.current_stmts.push(MirInst::CallExtern {
                                    dest: Some(result_vreg),
                                    name: extern_name.to_string(),
                                    args: vec![func_vreg],
                                    ty: any_ty,
                                });
                            }
                            HirExpr::Var(dec_sym, _) if self.user_funcs.contains(&dec_sym.0) => {
                                self.current_stmts.push(MirInst::Call {
                                    dest: Some(result_vreg),
                                    func: *dec_sym,
                                    args: vec![func_vreg],
                                    ty: any_ty,
                                });
                            }
                            HirExpr::Var(dec_sym, _)
                                if self.user_class_syms.contains(&dec_sym.0) =>
                            {
                                // Class-as-decorator: @CountCalls def f → f = CountCalls(f)
                                // Lower to mb_instance_new_with_init so the __init__ fires.
                                let class_name =
                                    self.class_syms.get(&dec_sym.0).cloned().unwrap_or_default();
                                let name_vreg = self.emit_str_const(&class_name);
                                let args_list = self.fresh_vreg();
                                self.current_stmts.push(MirInst::MakeList {
                                    dest: args_list,
                                    elements: vec![func_vreg],
                                    ty: any_ty,
                                });
                                self.current_stmts.push(MirInst::CallExtern {
                                    dest: Some(result_vreg),
                                    name: "mb_instance_new_with_init".to_string(),
                                    args: vec![name_vreg, args_list],
                                    ty: any_ty,
                                });
                            }
                            _ => {
                                let dec_vreg = self.lower_expr(dec_expr);
                                self.current_stmts.push(MirInst::CallExtern {
                                    dest: Some(result_vreg),
                                    name: "mb_call1_val".to_string(),
                                    args: vec![dec_vreg, func_vreg],
                                    ty: any_ty,
                                });
                            }
                        }
                        func_vreg = result_vreg;
                    }
                    self.current_stmts.push(MirInst::StoreGlobal {
                        name: *func_sym,
                        value: func_vreg,
                    });
                } else if *redef {
                    // A non-decorated `def` that redefines a name previously
                    // bound by a DECORATED `def`. The earlier decorated def
                    // already ran its StoreGlobal (binding the name to the
                    // decorator's wrapper/dummy); re-store the global to this
                    // impl's FuncRef so the plain impl wins the name.
                    let any_ty = self.tcx.any();
                    let func_vreg = self.fresh_vreg();
                    self.current_stmts.push(MirInst::LoadConst {
                        dest: func_vreg,
                        value: MirConst::FuncRef(*func_sym),
                        ty: any_ty,
                    });
                    self.current_stmts.push(MirInst::StoreGlobal {
                        name: *func_sym,
                        value: func_vreg,
                    });
                }
            }
            HirStmt::Match { subject, cases, .. } => {
                self.lower_match(subject, cases);
            }
            HirStmt::ClassDefPlaceholder { name: cls_sym, .. } => {
                self.emit_pending_class_registrations(Some(*cls_sym));
                self.emit_runtime_class_bases_for(Some(*cls_sym));
                // P2-R3: emit class-level attribute assignments at the class's
                // textual position so initializer expressions resolve imports
                // and bindings established by preceding statements (#1686
                // motivation), and BEFORE decorators so the decorator sees the
                // fully-initialized class body (CPython execution order).
                self.emit_class_attrs_for(Some(*cls_sym));
                // PEP 557: record ordered dataclass field facts BEFORE the
                // decorator call so the runtime `@dataclass` synthesizer sees
                // them. Emitted here (not at registration time) so default
                // expressions like `field(default_factory=list)` resolve
                // imports bound above the class (#1686 motivation).
                let dc_fields = {
                    let pos = self
                        .pending_dataclass_fields
                        .iter()
                        .position(|(_, s, _)| s == cls_sym);
                    pos.map(|i| self.pending_dataclass_fields.remove(i))
                };
                if let Some((class_name, _, facts)) = dc_fields {
                    for (field_name, ann, default) in &facts {
                        let cls_vreg = self.emit_str_const(&class_name);
                        let fname_vreg = self.emit_str_const(field_name);
                        let ann_vreg = self.emit_str_const(ann);
                        match default {
                            Some(def_expr) => {
                                let raw = self.lower_expr(def_expr);
                                let boxed = self.box_operand(raw, def_expr.ty());
                                self.current_stmts.push(MirInst::CallExtern {
                                    dest: None,
                                    name: "mb_dataclass_record_field".to_string(),
                                    args: vec![cls_vreg, fname_vreg, ann_vreg, boxed],
                                    ty: self.tcx.none(),
                                });
                            }
                            None => {
                                self.current_stmts.push(MirInst::CallExtern {
                                    dest: None,
                                    name: "mb_dataclass_record_field_nodefault".to_string(),
                                    args: vec![cls_vreg, fname_vreg, ann_vreg],
                                    ty: self.tcx.none(),
                                });
                            }
                        }
                    }
                }
                // Apply class decorator(s) at the textual position so:
                //  - decorator expressions can see imports declared above (#1686), and
                //  - subsequent statements observe the post-decorator class (#1690).
                // Class registration (mb_class_define_multi etc.) already
                // happened at the top of __main__ so the class symbol is live.
                let decorators = {
                    let pos = self
                        .pending_class_decorators
                        .iter()
                        .position(|(_, s, _)| s == cls_sym);
                    pos.map(|i| self.pending_class_decorators.remove(i))
                };
                if let Some((class_name, _cls_sym, decorators)) = decorators {
                    let any_ty = self.tcx.any();
                    let mut cls_vreg = self.emit_str_const(&class_name);
                    for dec_expr in decorators.iter().rev() {
                        let result_vreg = self.fresh_vreg();
                        match dec_expr {
                            HirExpr::Var(dec_sym, _) if self.user_funcs.contains(&dec_sym.0) => {
                                self.current_stmts.push(MirInst::Call {
                                    dest: Some(result_vreg),
                                    func: *dec_sym,
                                    args: vec![cls_vreg],
                                    ty: any_ty,
                                });
                            }
                            _ => {
                                let dec_vreg = self.lower_expr(dec_expr);
                                self.current_stmts.push(MirInst::CallExtern {
                                    dest: Some(result_vreg),
                                    name: "mb_call1_val".to_string(),
                                    args: vec![dec_vreg, cls_vreg],
                                    ty: any_ty,
                                });
                            }
                        }
                        // A raising decorator (@enum.unique on an aliased
                        // class) must reach an enclosing try's handler.
                        self.emit_try_exception_guard();
                        cls_vreg = result_vreg;
                    }
                }
            }
        }
    }

    fn emit_pending_class_registrations(&mut self, cls_sym: Option<SymbolId>) {
        let mut i = 0;
        while i < self.pending_classes.len() {
            let should_emit = match cls_sym {
                Some(sym) => self.pending_classes[i].1 == sym,
                // #82: skip classes that must register at their textual
                // placeholder (cross-class chained property decorator).
                None => self.pending_classes[i].8.is_empty()
                    && !self
                        .classes_needing_textual_registration
                        .contains(&self.pending_classes[i].1.0),
            };
            if should_emit {
                let registration = self.pending_classes.remove(i);
                self.emit_class_registration(&registration);
            } else {
                i += 1;
            }
        }
    }

    fn emit_class_registration(&mut self, registration: &PendingClassRegistration) {
        let (
            class_name,
            _class_sym,
            all_base_names,
            namedtuple_base,
            methods,
            match_args,
            metaclass,
            slots,
            class_kwargs,
        ) = registration;
        let name_vreg = self.emit_str_const(class_name);
        // Build bases list for multiple inheritance (P1 OOP conformance).
        // For single base, pass the base name directly for backward compat.
        // For multiple bases, build a list of base name strings.
        let bases_list_vreg = if all_base_names.is_empty() {
            self.emit_none()
        } else {
            let mut base_vregs = Vec::new();
            for base in all_base_names {
                base_vregs.push(self.emit_str_const(base));
            }
            let list_vreg = self.fresh_vreg();
            self.current_stmts.push(MirInst::MakeList {
                dest: list_vreg, elements: base_vregs, ty: self.tcx.any(),
            });
            list_vreg
        };
        // Build method_names list and method_values list. For methods
        // decorated with @property/@classmethod/@staticmethod, wrap the
        // raw function pointer in the corresponding descriptor object so
        // mb_getattr dispatches through the descriptor protocol.
        let any_ty = self.tcx.any();
        let mut name_vregs = Vec::new();
        let mut value_vregs = Vec::new();
        for (method_name, method_sym, decor_kind, setter_sym, deleter_sym, marker_attrs, generic_decorator) in methods {
            let name_vreg = self.emit_str_const(method_name);
            name_vregs.push(name_vreg);
            let addr_vreg = self.fresh_vreg();
            self.current_stmts.push(MirInst::LoadConst {
                dest: addr_vreg,
                value: MirConst::FuncRef(*method_sym),
                ty: self.tcx.int(),
            });
            for attr_name in marker_attrs {
                let attr_vreg = self.emit_str_const(attr_name);
                let true_vreg = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest: true_vreg,
                    value: MirConst::Bool(true),
                    ty: self.tcx.bool(),
                });
                let true_boxed = self.box_operand(true_vreg, self.tcx.bool());
                self.current_stmts.push(MirInst::CallExtern {
                    dest: None,
                    name: "mb_setattr".to_string(),
                    args: vec![addr_vreg, attr_vreg, true_boxed],
                    ty: self.tcx.none(),
                });
            }
            let wrapped = match decor_kind {
                MethodDecorKind::None => addr_vreg,
                MethodDecorKind::Property => {
                    let mut w = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(w), name: "mb_property_new".to_string(),
                        args: vec![addr_vreg], ty: any_ty,
                    });
                    // Attach setter if present. mb_property_setter returns a NEW
                    // property (sharing fget); capture it so the accessor is not
                    // dropped (it no longer mutates in place). (#82)
                    if let Some(ssym) = setter_sym {
                        let setter_addr = self.fresh_vreg();
                        self.current_stmts.push(MirInst::LoadConst {
                            dest: setter_addr,
                            value: MirConst::FuncRef(*ssym),
                            ty: self.tcx.int(),
                        });
                        let next = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(next), name: "mb_property_setter".to_string(),
                            args: vec![w, setter_addr], ty: any_ty,
                        });
                        w = next;
                    }
                    // Attach deleter if present (same NEW-property semantics).
                    if let Some(dsym) = deleter_sym {
                        let del_addr = self.fresh_vreg();
                        self.current_stmts.push(MirInst::LoadConst {
                            dest: del_addr,
                            value: MirConst::FuncRef(*dsym),
                            ty: self.tcx.int(),
                        });
                        let next = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(next), name: "mb_property_deleter".to_string(),
                            args: vec![w, del_addr], ty: any_ty,
                        });
                        w = next;
                    }
                    w
                }
                MethodDecorKind::ClassMethod => {
                    let w = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(w), name: "mb_classmethod_new".to_string(),
                        args: vec![addr_vreg], ty: any_ty,
                    });
                    w
                }
                MethodDecorKind::StaticMethod => {
                    let w = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(w), name: "mb_staticmethod_new".to_string(),
                        args: vec![addr_vreg], ty: any_ty,
                    });
                    w
                }
                MethodDecorKind::CachedProperty => {
                    let name_str = self.emit_str_const(method_name);
                    let w = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(w), name: "mb_cached_property_new".to_string(),
                        args: vec![addr_vreg, name_str], ty: any_ty,
                    });
                    w
                }
            };
            // Apply a generic cross-class chained decorator (`@Base.x.setter`):
            // evaluate the descriptor-bound method (`Base.x.setter`) and call it
            // on the wrapped method value; the result (a NEW property sharing the
            // base's accessors) becomes this class's attribute. Applied inline so
            // execution order matches CPython (the decorator runs at class-def
            // time, before later statements use the property). (#82)
            let wrapped = if let Some(dec_expr) = generic_decorator {
                let dec_vreg = self.lower_expr(dec_expr);
                let result_vreg = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(result_vreg),
                    name: "mb_call1_val".to_string(),
                    args: vec![dec_vreg, wrapped],
                    ty: any_ty,
                });
                result_vreg
            } else {
                wrapped
            };
            value_vregs.push(wrapped);
        }
        let names_list = self.fresh_vreg();
        self.current_stmts.push(MirInst::MakeList {
            dest: names_list, elements: name_vregs, ty: self.tcx.any(),
        });
        let values_list = self.fresh_vreg();
        self.current_stmts.push(MirInst::MakeList {
            dest: values_list, elements: value_vregs, ty: self.tcx.any(),
        });
        // R10: Emit class keyword arguments BEFORE class registration
        // so they are available in KWARGS_REGISTRY when __init_subclass__ is called.
        if !class_kwargs.is_empty() {
            let mut key_vregs = Vec::new();
            let mut val_vregs_kw = Vec::new();
            for (kwarg_name, kwarg_expr) in class_kwargs {
                key_vregs.push(self.emit_str_const(kwarg_name));
                let val_vreg = self.lower_expr(kwarg_expr);
                let boxed = self.box_operand(val_vreg, kwarg_expr.ty());
                val_vregs_kw.push(boxed);
            }
            let keys_list = self.fresh_vreg();
            self.current_stmts.push(MirInst::MakeList {
                dest: keys_list, elements: key_vregs, ty: self.tcx.any(),
            });
            let vals_list_kw = self.fresh_vreg();
            self.current_stmts.push(MirInst::MakeList {
                dest: vals_list_kw, elements: val_vregs_kw, ty: self.tcx.any(),
            });
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_class_set_kwargs".to_string(),
                args: vec![name_vreg, keys_list, vals_list_kw],
                ty: self.tcx.none(),
            });
        }
        self.current_stmts.push(MirInst::CallExtern {
            dest: None,
            name: "mb_class_define_multi".to_string(),
            args: vec![name_vreg, bases_list_vreg, names_list, values_list],
            ty: self.tcx.none(),
        });
        if let Some(spec) = namedtuple_base {
            let tuple_name_vreg = self.emit_str_const(&spec.tuple_name);
            let mut field_vregs = Vec::new();
            for field in &spec.fields {
                field_vregs.push(self.emit_str_const(field));
            }
            let fields_list = self.fresh_vreg();
            self.current_stmts.push(MirInst::MakeList {
                dest: fields_list, elements: field_vregs, ty: self.tcx.any(),
            });
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_class_set_namedtuple_base".to_string(),
                args: vec![name_vreg, tuple_name_vreg, fields_list],
                ty: self.tcx.none(),
            });
        }
        // abc: register the names declared `@abc.abstractmethod` so the
        // runtime can compute `__abstractmethods__` and reject instantiation
        // of classes that still have un-overridden abstract methods.
        let abs_names: Vec<String> = self
            .pending_abstract_methods
            .iter()
            .find(|(cn, _)| cn == class_name)
            .map(|(_, names)| names.clone())
            .unwrap_or_default();
        if !abs_names.is_empty() {
            let mut abs_vregs = Vec::new();
            for an in &abs_names {
                abs_vregs.push(self.emit_str_const(an));
            }
            let abs_list = self.fresh_vreg();
            self.current_stmts.push(MirInst::MakeList {
                dest: abs_list, elements: abs_vregs, ty: self.tcx.any(),
            });
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_class_set_abstractmethods".to_string(),
                args: vec![name_vreg, abs_list],
                ty: self.tcx.none(),
            });
        }
        // P2-R2: Set metaclass if specified (e.g., class Foo(metaclass=Meta)).
        if let Some(ref meta_name) = metaclass {
            let meta_vreg = self.emit_str_const(meta_name);
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_class_set_metaclass".to_string(),
                args: vec![name_vreg, meta_vreg],
                ty: self.tcx.none(),
            });
        }
        // Register __match_args__ for PEP 634 positional class patterns (#827)
        if !match_args.is_empty() {
            let mut arg_vregs = Vec::new();
            for arg_name in match_args {
                arg_vregs.push(self.emit_str_const(arg_name));
            }
            let args_tuple = self.fresh_vreg();
            self.current_stmts.push(MirInst::MakeTuple {
                dest: args_tuple, elements: arg_vregs, ty: self.tcx.any(),
            });
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_class_set_match_args".to_string(),
                args: vec![name_vreg, args_tuple],
                ty: self.tcx.none(),
            });
        }
        // R14: Emit mb_register_slots if __slots__ declared in class body.
        if let Some(ref slot_names) = slots {
            let mut slot_vregs = Vec::new();
            for slot_name in slot_names {
                slot_vregs.push(self.emit_str_const(slot_name));
            }
            let slots_list = self.fresh_vreg();
            self.current_stmts.push(MirInst::MakeList {
                dest: slots_list, elements: slot_vregs, ty: self.tcx.any(),
            });
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_register_slots".to_string(),
                args: vec![name_vreg, slots_list],
                ty: self.tcx.none(),
            });
        }
    }

    /// P2-R3: Emit `mb_class_set_class_attr` calls for pending class-level
    /// attribute assignments. `cls_sym = Some(sym)` drains only that class's
    /// entries (ClassDefPlaceholder path, textual order); `None` drains every
    /// remaining entry (post-loop fallback for classes without a placeholder).
    fn emit_class_attrs_for(&mut self, cls_sym: Option<SymbolId>) {
        let mut i = 0;
        while i < self.pending_class_attrs.len() {
            if cls_sym.map_or(true, |s| self.pending_class_attrs[i].1 == s) {
                let (class_name, _, attr_name, val_expr) = self.pending_class_attrs.remove(i);
                let cls_vreg = self.emit_str_const(&class_name);
                let attr_vreg = self.emit_str_const(&attr_name);
                let val_vreg = self.lower_expr(&val_expr);
                let boxed = self.box_operand(val_vreg, val_expr.ty());
                self.current_stmts.push(MirInst::CallExtern {
                    dest: None,
                    name: "mb_class_set_class_attr".to_string(),
                    args: vec![cls_vreg, attr_vreg, boxed],
                    ty: self.tcx.none(),
                });
            } else {
                i += 1;
            }
        }
    }

    fn emit_runtime_class_bases_for(&mut self, cls_sym: Option<SymbolId>) {
        let mut i = 0;
        while i < self.pending_runtime_class_bases.len() {
            if !cls_sym.map_or(true, |s| self.pending_runtime_class_bases[i].1 == s) {
                i += 1;
                continue;
            }
            let (class_name, _, base_exprs) = self.pending_runtime_class_bases.remove(i);
            let cls_vreg = self.emit_str_const(&class_name);
            let mut base_vregs = Vec::new();
            for expr in &base_exprs {
                let raw = self.lower_expr(expr);
                base_vregs.push(self.box_operand(raw, expr.ty()));
            }
            let bases_list = self.fresh_vreg();
            self.current_stmts.push(MirInst::MakeList {
                dest: bases_list,
                elements: base_vregs,
                ty: self.tcx.any(),
            });
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_class_update_bases".to_string(),
                args: vec![cls_vreg, bases_list],
                ty: self.tcx.none(),
            });
        }
    }

    /// Convert an expression condition to a 0/1 bool value for branch instructions.
    /// For typed integers, emit explicit != 0 for Python truthiness.
    /// For NaN-boxed bools, use directly (band_imm in Cranelift extracts LSB = bool bit).
    /// For heap-backed types (str, list, etc.) and `any`, call mb_is_truthy to get 0/1.
    fn lower_cond_as_bool(&mut self, cond: &HirExpr) -> VReg {
        let cond_vreg = self.lower_expr(cond);
        let ty = cond.ty();
        if ty == self.tcx.int() {
            let zero = self.fresh_vreg();
            self.current_stmts.push(MirInst::LoadConst {
                dest: zero,
                value: MirConst::Int(0),
                ty: self.tcx.int(),
            });
            let result = self.fresh_vreg();
            self.current_stmts.push(MirInst::BinOp {
                dest: result,
                op: MirBinOp::NotEq,
                lhs: cond_vreg,
                rhs: zero,
                ty: self.tcx.bool(),
            });
            result
        } else if ty == self.tcx.bool() {
            // NaN-boxed bool: LSB encodes the bool value — band_imm in Cranelift handles it.
            cond_vreg
        } else {
            // Heap-backed or `any` type: call mb_is_truthy to get a clean 0/1 result.
            // Heap pointers have LSB=0 regardless of emptiness, so band_imm would always
            // evaluate to false without this call (#827).
            let result = self.fresh_vreg();
            self.current_stmts.push(MirInst::CallExtern {
                dest: Some(result),
                name: "mb_is_truthy".to_string(),
                args: vec![cond_vreg],
                ty: self.tcx.int(),
            });
            result
        }
    }

    fn lower_if(&mut self, cond: &HirExpr, then_body: &[HirStmt], else_body: &[HirStmt]) {
        let cond_vreg = self.lower_cond_as_bool(cond);
        let then_block = self.fresh_block();
        let else_block = self.fresh_block();
        let merge_block = self.fresh_block();

        self.finish_block(Terminator::Branch {
            cond: cond_vreg,
            then_block,
            else_block,
        });

        // Then branch
        self.start_block(then_block);
        for s in then_body {
            self.lower_stmt(s);
        }
        self.finish_block(Terminator::Goto(merge_block));

        // Else branch
        self.start_block(else_block);
        for s in else_body {
            self.lower_stmt(s);
        }
        self.finish_block(Terminator::Goto(merge_block));

        // Continue at merge
        self.start_block(merge_block);
    }

    fn lower_while(&mut self, cond: &HirExpr, body: &[HirStmt], else_body: &[HirStmt]) {
        let header = self.fresh_block();
        let body_block = self.fresh_block();
        let final_exit = self.fresh_block();

        // If there's an else clause, condition-false goes to else_block first.
        // Break jumps directly to final_exit (skipping else).
        let natural_exit = if !else_body.is_empty() {
            self.fresh_block()
        } else {
            final_exit
        };

        self.finish_block(Terminator::Goto(header));

        // Header: evaluate condition
        self.start_block(header);
        let cond_vreg = self.lower_cond_as_bool(cond);
        self.finish_block(Terminator::Branch {
            cond: cond_vreg,
            then_block: body_block,
            else_block: natural_exit,
        });

        // Body — break jumps to final_exit (past else)
        // Suppress StoreGlobal in loop body — module-scope stores per iteration
        // are the #1 perf killer. Sync globals after loop exits.
        let old_exit = self.loop_exit.replace(final_exit);
        let old_header = self.loop_header.replace(header);
        let was_module_scope = self.in_module_scope;
        self.in_module_scope = false;
        self.start_block(body_block);
        for s in body {
            self.lower_stmt(s);
        }
        self.in_module_scope = was_module_scope;
        // If we're inside a try block, emit an exception check at the end of
        // the loop body so that exceptions raised during the iteration
        // (e.g. StopIteration from next()) propagate to the try handler
        // instead of looping forever.
        if let Some(&(handler_block, _)) = self.try_handler_stack.last() {
            let exc_check = self.fresh_vreg();
            self.current_stmts.push(MirInst::CallExtern {
                dest: Some(exc_check),
                name: "mb_has_exception".to_string(),
                args: Vec::new(),
                ty: self.tcx.bool(),
            });
            let exc_break_block = self.fresh_block();
            let normal_continue = self.fresh_block();
            self.finish_block(Terminator::Branch {
                cond: exc_check,
                then_block: exc_break_block,
                else_block: normal_continue,
            });
            // Exception detected — pop the handler and jump to handler block
            self.start_block(exc_break_block);
            self.emit_extern_call(None, "mb_pop_handler");
            self.finish_block(Terminator::Goto(handler_block));
            // Normal path — continue to loop header
            self.start_block(normal_continue);
        }
        self.finish_block(Terminator::Goto(header));
        self.loop_exit = old_exit;
        self.loop_header = old_header;

        // Else block (only when loop exits naturally, not via break)
        if !else_body.is_empty() {
            self.start_block(natural_exit);
            for s in else_body {
                self.lower_stmt(s);
            }
            self.finish_block(Terminator::Goto(final_exit));
        }

        // Continue after loop
        self.start_block(final_exit);
    }

    /// Lower a for-loop using the iterator protocol:
    /// iter_obj = mb_iter(iterable)
    /// loop:
    ///   if !mb_has_next(iter_obj): goto exit
    ///   var = mb_next(iter_obj)
    ///   body...
    ///   goto loop
    /// exit:
    ///   mb_iter_release(iter_obj)
    /// Check if an expression is `range(stop)`, `range(start, stop)`, or
    /// `range(start, stop, step)` with all arguments typed as Int.
    /// Only typed-int args are safe for native counter (raw i64 arithmetic).
    fn is_range_call(&self, expr: &HirExpr) -> bool {
        if let HirExpr::Call { func, args, .. } = expr {
            if let HirExpr::Var(sym, _) = func.as_ref() {
                if let Some(ext) = self.builtin_syms.get(&sym.0) {
                    if ext == "mb_range" && !args.is_empty() && args.len() <= 3 {
                        // All args must have Int type — untyped/Any args are
                        // NaN-boxed and can't be used with raw iadd/icmp.
                        if !args
                            .iter()
                            .all(|a| matches!(self.tcx.get(a.ty()), crate::types::Ty::Int))
                        {
                            return false;
                        }
                        // Native counter compares with a fixed `Lt`/`Gt`, so the
                        // step sign must be known at compile time. For 3-arg
                        // `range`, only literal non-zero steps qualify; dynamic
                        // or zero steps fall back to the boxed iterator path
                        // (which dispatches on step sign at runtime).
                        if args.len() == 3 {
                            return matches!(Self::hir_literal_int(&args[2]), Some(s) if s != 0);
                        }
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Extract a compile-time integer value from a HIR expression.
    /// Handles plain literals and unary `-IntLit` for negative literals.
    fn hir_literal_int(e: &HirExpr) -> Option<i64> {
        match e {
            HirExpr::IntLit(n, _) => Some(*n),
            HirExpr::UnaryOp {
                op: crate::hir::HirUnaryOp::Neg,
                operand,
                ..
            } => {
                if let HirExpr::IntLit(n, _) = operand.as_ref() {
                    Some(-n)
                } else {
                    None
                }
            }
            HirExpr::UnaryOp {
                op: crate::hir::HirUnaryOp::Pos,
                operand,
                ..
            } => {
                if let HirExpr::IntLit(n, _) = operand.as_ref() {
                    Some(*n)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Emit a native counter loop for `for var in range(...)`.
    /// Produces: var = start; while var < stop { body; var += step }
    /// No iterator allocation, no extern calls in the loop hot path.
    fn lower_for_range(
        &mut self,
        var: SymbolId,
        iter_expr: &HirExpr,
        body: &[HirStmt],
        else_body: &[HirStmt],
    ) {
        let range_args: Vec<&HirExpr> = if let HirExpr::Call { args, .. } = iter_expr {
            args.iter().collect()
        } else {
            unreachable!()
        };
        let int_ty = self.tcx.int();

        // Check if the loop var has a known Int type in the type system.
        // If so, we can use raw int arithmetic (fast). Otherwise, use boxed
        // arithmetic (safe for generators/yield/polymorphic use).

        // #2105: lower each range() argument and unbox-if-boxed before the
        // native counter loop, which compares values as raw i64. Static-int
        // expressions (e.g. literals, raw arithmetic) arrive raw; but the
        // result of a user-function call that internally uses NaN-boxing
        // builtins like `max()` / `min()` arrives as a NaN-boxed MbValue.
        // Comparing the NaN-boxed bit pattern against 0 as a signed i64
        // yields a huge negative number, so `var < stop` is false at entry
        // and the loop body silently elides. `mb_unbox_int_if_boxed` is a
        // pass-through for already-raw values, so it is correctness-only.
        let unbox_range_arg = |this: &mut Self, e: &HirExpr| -> VReg {
            let v = this.lower_expr(e);
            let unboxed = this.fresh_vreg();
            this.current_stmts.push(MirInst::CallExtern {
                dest: Some(unboxed),
                name: "mb_unbox_int_if_boxed".to_string(),
                args: vec![v],
                ty: int_ty,
            });
            unboxed
        };

        // Parse range args: range(stop), range(start, stop), range(start, stop, step)
        let (start_raw, stop_raw, step_raw) = match range_args.len() {
            1 => {
                let stop = unbox_range_arg(self, range_args[0]);
                let start = self.emit_int_const(0);
                let step = self.emit_int_const(1);
                (start, stop, step)
            }
            2 => {
                let start = unbox_range_arg(self, range_args[0]);
                let stop = unbox_range_arg(self, range_args[1]);
                let step = self.emit_int_const(1);
                (start, stop, step)
            }
            3 => {
                let start = unbox_range_arg(self, range_args[0]);
                let stop = unbox_range_arg(self, range_args[1]);
                let step = unbox_range_arg(self, range_args[2]);
                (start, stop, step)
            }
            _ => unreachable!(),
        };

        // Initialize loop variable: var = start
        // Separate the private loop counter from the user-visible loop variable.
        //
        // Bug: if two nested `for`-loops share the same variable name (e.g. both
        // use `_`), the original code used `var_vreg` both as the JIT loop counter
        // AND as the user-visible binding.  When the inner loop ran, it wrote its
        // own counter values into the shared VReg, corrupting the outer loop's
        // counter and causing it to exit after just one iteration.
        //
        // Fix: `counter_vreg` is a fresh, private VReg used exclusively for loop
        // control (header condition + latch increment).  `var_vreg` (in sym_to_vreg)
        // is the user-visible binding; it is synced from `counter_vreg` at the
        // start of every body iteration.  Nested loops over the same name write to
        // `var_vreg` but never touch `counter_vreg`, so the outer counter is safe.
        let counter_vreg = self.fresh_vreg();
        self.current_stmts.push(MirInst::Copy {
            dest: counter_vreg,
            source: start_raw,
        });

        // User-visible loop variable — may already exist if an enclosing scope
        // declared it (including an outer `for _ in ...`).
        let var_vreg = if let Some(&existing) = self.sym_to_vreg.get(&var) {
            existing
        } else {
            // Use start_raw's VReg slot as the user-visible binding when the
            // symbol hasn't been mapped yet (avoids an extra allocation for the
            // common non-nested case).
            self.sym_to_vreg.insert(var, start_raw);
            start_raw
        };

        let header = self.fresh_block();
        let body_block = self.fresh_block();
        let latch_block = self.fresh_block(); // increment lives here — continue jumps here
        let cleanup_block = self.fresh_block();

        let natural_exit = if !else_body.is_empty() {
            self.fresh_block()
        } else {
            cleanup_block
        };

        self.finish_block(Terminator::Goto(header));

        // Header: loop condition checks the private counter, not var_vreg.
        // For positive step (the only sign the 1/2-arg form can produce) we
        // exit when `counter >= stop`; for a literal negative step we mirror
        // with `counter <= stop`.
        let cmp_op = if range_args.len() == 3 {
            match Self::hir_literal_int(range_args[2]) {
                Some(s) if s > 0 => MirBinOp::Lt,
                Some(s) if s < 0 => MirBinOp::Gt,
                _ => unreachable!("is_range_call rejects non-literal/zero step"),
            }
        } else {
            MirBinOp::Lt
        };
        self.start_block(header);
        let cond = self.fresh_vreg();
        self.current_stmts.push(MirInst::BinOp {
            dest: cond,
            op: cmp_op,
            lhs: counter_vreg,
            rhs: stop_raw,
            ty: int_ty,
        });
        self.finish_block(Terminator::Branch {
            cond,
            then_block: body_block,
            else_block: natural_exit,
        });

        // Body: sync user-visible var from private counter, then execute body.
        // `continue` jumps to latch_block so the counter increment is never skipped.
        // Suppress StoreGlobal during loop body — module-scope variable stores
        // inside a tight loop are the #1 performance killer (~200ns/iter).
        let old_exit = self.loop_exit.replace(cleanup_block);
        let old_header = self.loop_header.replace(latch_block);
        let was_module_scope = self.in_module_scope;
        self.in_module_scope = false;
        self.start_block(body_block);
        // Expose current counter value as the user-visible loop variable.
        // This is a no-op for non-nested loops (var_vreg == start_raw, same
        // value) but is essential when nested loops share the same name.
        self.current_stmts.push(MirInst::Copy {
            dest: var_vreg,
            source: counter_vreg,
        });
        for s in body {
            self.lower_stmt(s);
        }
        self.in_module_scope = was_module_scope;
        self.finish_block(Terminator::Goto(latch_block));

        // Latch: increment private counter, then jump back to header.
        // var_vreg is NOT touched here — it keeps the value from body entry
        // (the last yielded value), which is what Python exposes after the loop.
        self.start_block(latch_block);
        let next_val = self.fresh_vreg();
        self.current_stmts.push(MirInst::BinOp {
            dest: next_val,
            op: MirBinOp::Add,
            lhs: counter_vreg,
            rhs: step_raw,
            ty: int_ty,
        });
        self.current_stmts.push(MirInst::Copy {
            dest: counter_vreg,
            source: next_val,
        });
        self.finish_block(Terminator::Goto(header));
        self.loop_exit = old_exit;
        self.loop_header = old_header;

        // Else block
        if !else_body.is_empty() {
            self.start_block(natural_exit);
            for s in else_body {
                self.lower_stmt(s);
            }
            self.finish_block(Terminator::Goto(cleanup_block));
        }

        // Cleanup: sync modified variables back to global storage.
        // var_vreg holds the last value assigned to the loop variable (the
        // last counter value before the loop condition failed).
        self.start_block(cleanup_block);
        if was_module_scope {
            self.current_stmts.push(MirInst::StoreGlobal {
                name: var,
                value: var_vreg,
            });
        }
    }

    fn lower_for(
        &mut self,
        var: SymbolId,
        iter: &HirExpr,
        body: &[HirStmt],
        else_body: &[HirStmt],
    ) {
        // Fast path: for i in range(...) → native counter loop.
        // Skip for generator bodies — yield needs NaN-boxed values, but the
        // native counter uses raw i64. TODO: box at yield points only.
        if self.is_range_call(iter) && !self.is_gen_body {
            self.lower_for_range(var, iter, body, else_body);
            return;
        }

        let iterable = self.lower_expr(iter);

        // Create iterator: iter_obj = mb_iter(iterable)
        let iter_obj = self.fresh_vreg();
        self.current_stmts.push(MirInst::CallExtern {
            dest: Some(iter_obj),
            name: "mb_iter".to_string(),
            args: vec![iterable],
            ty: self.tcx.any(),
        });

        let header = self.fresh_block();
        let body_block = self.fresh_block();
        let cleanup_block = self.fresh_block();

        // For else: natural exit → else_block → cleanup. Break → cleanup (skip else).
        let natural_exit = if !else_body.is_empty() {
            self.fresh_block()
        } else {
            cleanup_block
        };

        self.finish_block(Terminator::Goto(header));

        // Header: single-call advance via mb_next_or_stop — returns sentinel
        // when exhausted, otherwise the next value (which may legitimately
        // be None). mb_is_stop_iter does the bool check; together they
        // replace the legacy mb_has_next + mb_next pair (Lever A).
        self.start_block(header);
        let next_val = self.fresh_vreg();
        self.current_stmts.push(MirInst::CallExtern {
            dest: Some(next_val),
            name: "mb_next_or_stop".to_string(),
            args: vec![iter_obj],
            ty: self.tcx.any(),
        });
        let is_stop = self.fresh_vreg();
        self.current_stmts.push(MirInst::CallExtern {
            dest: Some(is_stop),
            name: "mb_is_stop_iter".to_string(),
            args: vec![next_val],
            ty: self.tcx.bool(),
        });
        self.finish_block(Terminator::Branch {
            cond: is_stop,
            then_block: natural_exit,
            else_block: body_block,
        });

        // Body: assign value to loop variable, execute body
        // break jumps to cleanup_block (past else)
        // Suppress StoreGlobal in loop body for performance.
        let old_exit = self.loop_exit.replace(cleanup_block);
        let old_header = self.loop_header.replace(header);
        let was_module_scope = self.in_module_scope;
        self.in_module_scope = false;
        self.start_block(body_block);
        if let Some(&orig) = self.sym_to_vreg.get(&var) {
            self.current_stmts.push(MirInst::Copy {
                dest: orig,
                source: next_val,
            });
        } else {
            self.sym_to_vreg.insert(var, next_val);
        }
        for s in body {
            self.lower_stmt(s);
        }
        self.in_module_scope = was_module_scope;
        self.finish_block(Terminator::Goto(header));
        self.loop_exit = old_exit;
        self.loop_header = old_header;

        // Else block (only when loop exits naturally)
        if !else_body.is_empty() {
            self.start_block(natural_exit);
            for s in else_body {
                self.lower_stmt(s);
            }
            self.finish_block(Terminator::Goto(cleanup_block));
        }

        // Cleanup: release iterator (always runs)
        self.start_block(cleanup_block);
        self.current_stmts.push(MirInst::CallExtern {
            dest: None,
            name: "mb_iter_release".to_string(),
            args: vec![iter_obj],
            ty: self.tcx.none(),
        });
    }

    /// Desugar tuple/starred unpacking assignment into indexed access (#409).
    ///
    /// `a, b, c = rhs`  → `a = rhs[0]; b = rhs[1]; c = rhs[2]`
    /// `a, *rest, b = rhs` → `a = rhs[0]; b = rhs[-1]; rest = rhs[1:-1]`
    fn lower_unpack_assign(
        &mut self,
        rhs_in: VReg,
        targets: &[HirLValue],
        star_index: Option<usize>,
    ) {
        let n = targets.len();
        let int_ty = self.tcx.int();
        // Materialize the RHS only when it isn't already a list/tuple. The
        // legacy `mb_list_from_iterable` path always allocated a fresh list +
        // cloned the items vec + retained every element — a major chunk of the
        // residual #2178 per-iter alloc cost for hot patterns like
        // `h, l, s = rgb_to_hls(...)`. `mb_seq_for_unpack` is a zero-copy
        // passthrough for list/tuple inputs (with one retain so the trailing
        // `mb_release` on the temporary still balances) and falls back to
        // `mb_list_from_iterable` for iterators / strings / dicts / sets /
        // user iterables.
        //
        // Star-target unpacking (`a, *rest, b = ...`) lowers to `mb_list_slice`
        // which today only handles lists, so for that shape we stay on the
        // legacy materializing path. Non-star unpack is the dominant case
        // and the hot loop the #2178 cohort actually exercises.
        let rhs = self.fresh_vreg();
        let materialize_fn = if star_index.is_some() {
            "mb_list_from_iterable"
        } else {
            "mb_seq_for_unpack"
        };
        self.current_stmts.push(MirInst::CallExtern {
            dest: Some(rhs),
            name: materialize_fn.to_string(),
            args: vec![rhs_in],
            ty: self.tcx.any(),
        });
        // Raise ValueError on length mismatch (CPython semantics).
        // - Without star: exactly n values required.
        // - With star: at least (n-1) values required.
        // `mb_seq_len_boxed` mirrors `mb_list_len` for lists and adds tuple
        // support so the zero-copy passthrough above works correctly when the
        // RHS arrives as a tuple. Returns a NaN-boxed int; mb_ne compares by
        // bits, so the expected literal must be boxed to match — otherwise the
        // unpack always "fails" (tagged vs raw bits never equal), silently
        // setting a ValueError that surfaces at module exit.
        let actual_len_raw = self.fresh_vreg();
        self.current_stmts.push(MirInst::CallExtern {
            dest: Some(actual_len_raw),
            name: "mb_seq_len_boxed".to_string(),
            args: vec![rhs],
            ty: int_ty,
        });
        let actual_len = actual_len_raw;
        let expected = if star_index.is_some() {
            (n - 1) as i64
        } else {
            n as i64
        };
        let expected_raw = self.emit_int_const(expected);
        let expected_vreg = self.box_operand(expected_raw, int_ty);
        let cmp = self.fresh_vreg();
        if star_index.is_some() {
            // Need at least (n-1) — raise if actual < expected
            self.current_stmts.push(MirInst::CallExtern {
                dest: Some(cmp),
                name: "mb_lt".to_string(),
                args: vec![actual_len, expected_vreg],
                ty: self.tcx.bool(),
            });
        } else {
            // Need exactly n — raise if actual != expected
            self.current_stmts.push(MirInst::CallExtern {
                dest: Some(cmp),
                name: "mb_ne".to_string(),
                args: vec![actual_len, expected_vreg],
                ty: self.tcx.bool(),
            });
        }
        let raise_block = self.fresh_block();
        let ok_block = self.fresh_block();
        self.finish_block(Terminator::Branch {
            cond: cmp,
            then_block: raise_block,
            else_block: ok_block,
        });
        self.start_block(raise_block);
        let err_type = self.emit_str_const("ValueError");
        let err_msg = if star_index.is_some() {
            self.emit_str_const("not enough values to unpack")
        } else {
            // Either too few or too many — CPython distinguishes, but a generic
            // message still satisfies `except ValueError` handlers.
            self.emit_str_const("unpack count mismatch")
        };
        self.current_stmts.push(MirInst::CallExtern {
            dest: None,
            name: "mb_raise".to_string(),
            args: vec![err_type, err_msg],
            ty: self.tcx.none(),
        });
        // If in a try, propagate to handler; otherwise continue (exception check elsewhere)
        if let Some(&(handler_block, _)) = self.try_handler_stack.last() {
            self.emit_extern_call(None, "mb_pop_handler");
            self.finish_block(Terminator::Goto(handler_block));
        } else {
            self.finish_block(Terminator::Goto(ok_block));
        }
        self.start_block(ok_block);
        match star_index {
            None => {
                // Simple unpacking: each target gets rhs[i]
                // Indices must be NaN-boxed for mb_list_getitem (which handles both list & tuple).
                for (i, target) in targets.iter().enumerate() {
                    let idx_raw = self.emit_int_const(i as i64);
                    let idx = self.box_operand(idx_raw, int_ty);
                    let elem = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(elem),
                        name: "mb_list_getitem".to_string(),
                        args: vec![rhs, idx],
                        ty: self.tcx.any(),
                    });
                    self.assign_to_lvalue(target, elem);
                }
            }
            Some(star_pos) => {
                let after_star = n - star_pos - 1; // targets after the starred one
                                                   // Targets before star: rhs[0], rhs[1], ...
                for i in 0..star_pos {
                    let idx_raw = self.emit_int_const(i as i64);
                    let idx = self.box_operand(idx_raw, int_ty);
                    let elem = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(elem),
                        name: "mb_list_getitem".to_string(),
                        args: vec![rhs, idx],
                        ty: self.tcx.any(),
                    });
                    self.assign_to_lvalue(&targets[i], elem);
                }
                // Targets after star: rhs[-after_star], rhs[-after_star+1], ...
                for j in 0..after_star {
                    let neg_idx = -((after_star - j) as i64);
                    let idx_raw = self.emit_int_const(neg_idx);
                    let idx = self.box_operand(idx_raw, int_ty);
                    let elem = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(elem),
                        name: "mb_list_getitem".to_string(),
                        args: vec![rhs, idx],
                        ty: self.tcx.any(),
                    });
                    self.assign_to_lvalue(&targets[star_pos + 1 + j], elem);
                }
                // Star target: rhs[star_pos : n-after_star]
                let start_raw = self.emit_int_const(star_pos as i64);
                let start = self.box_operand(start_raw, int_ty);
                let end_neg = if after_star > 0 {
                    let raw = self.emit_int_const(-(after_star as i64));
                    self.box_operand(raw, int_ty)
                } else {
                    // No targets after star — slice to end. We're on the
                    // legacy `mb_list_from_iterable` materializing path for
                    // star unpack (see comment above) so the rhs is always
                    // a list — `mb_list_len` is fine.
                    let len = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(len),
                        name: "mb_list_len".to_string(),
                        args: vec![rhs],
                        ty: self.tcx.int(),
                    });
                    self.box_operand(len, int_ty)
                };
                let slice = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(slice),
                    name: "mb_list_slice".to_string(),
                    args: vec![rhs, start, end_neg],
                    ty: self.tcx.any(),
                });
                self.assign_to_lvalue(&targets[star_pos], slice);
            }
        }
    }

    /// Assign a value to a single l-value target.
    fn assign_to_lvalue(&mut self, target: &HirLValue, val: VReg) {
        match target {
            HirLValue::Var(sym) => {
                // Mirror the HirStmt::Assign Var-target path so that
                // unpack-assignment (`a, b = ...`) honours the same
                // variable-class dispatch (Global / Cell / Free / Local)
                // as plain assignment. The earlier shorthand "Copy if
                // mapped, else insert val directly" silently dropped
                // global / cell write-throughs and skipped the fresh
                // VReg + Copy that drives Cranelift's SSA def-var, so
                // generator bodies in particular saw stale `None`s in
                // place of the unpacked values.
                let val_ty = self.tcx.any();
                if self.cell_override.contains(&sym.0) {
                    let boxed = self.box_operand(val, val_ty);
                    self.current_stmts.push(MirInst::StoreGlobal {
                        name: *sym,
                        value: boxed,
                    });
                    return;
                }
                let var_class = self
                    .symbol_table
                    .map(|st| st.get_var_class(*sym))
                    .unwrap_or(VariableClass::Local);
                match var_class {
                    VariableClass::Global => {
                        self.current_stmts.push(MirInst::StoreGlobal {
                            name: *sym,
                            value: val,
                        });
                    }
                    VariableClass::Cell => {
                        self.current_stmts.push(MirInst::StoreGlobal {
                            name: *sym,
                            value: val,
                        });
                    }
                    VariableClass::Free => {
                        let outer_sym =
                            self.symbol_table.and_then(|st| st.get_nonlocal_outer(*sym));
                        if let Some(outer) = outer_sym {
                            self.current_stmts.push(MirInst::StoreGlobal {
                                name: outer,
                                value: val,
                            });
                        } else if let Some(&orig) = self.sym_to_vreg.get(sym) {
                            self.current_stmts.push(MirInst::Copy {
                                dest: orig,
                                source: val,
                            });
                        } else {
                            let dest = self.fresh_vreg();
                            self.current_stmts.push(MirInst::Copy { dest, source: val });
                            self.sym_to_vreg.insert(*sym, dest);
                        }
                    }
                    VariableClass::Local => {
                        if let Some(&orig) = self.sym_to_vreg.get(sym) {
                            self.current_stmts.push(MirInst::Copy {
                                dest: orig,
                                source: val,
                            });
                            if self.in_module_scope {
                                let boxed = self.box_operand(orig, val_ty);
                                self.current_stmts.push(MirInst::StoreGlobal {
                                    name: *sym,
                                    value: boxed,
                                });
                            }
                        } else {
                            let dest = self.fresh_vreg();
                            self.current_stmts.push(MirInst::Copy { dest, source: val });
                            self.sym_to_vreg.insert(*sym, dest);
                            if self.in_module_scope {
                                let boxed = self.box_operand(dest, val_ty);
                                self.current_stmts.push(MirInst::StoreGlobal {
                                    name: *sym,
                                    value: boxed,
                                });
                            }
                        }
                    }
                }
            }
            HirLValue::Attr { object, attr } => {
                let obj = self.lower_expr(object);
                self.current_stmts.push(MirInst::SetAttr {
                    object: obj,
                    attr: attr.clone(),
                    value: val,
                });
            }
            HirLValue::Index { object, index } => {
                let obj = self.lower_expr(object);
                let idx = self.lower_expr(index);
                self.current_stmts.push(MirInst::SetItem {
                    object: obj,
                    index: idx,
                    value: val,
                });
            }
            HirLValue::Unpack {
                targets,
                star_index,
            } => {
                self.lower_unpack_assign(val, targets, *star_index);
            }
        }
    }

    /// Recursively emit decision-tree tests for a pattern against `subj_vreg` (#827, R7).
    ///
    /// Falls through when the pattern matches; jumps to `fail_block` on mismatch.
    /// Capture bindings are registered as a side effect.
    ///
    /// `raw_subject` is the vreg to bind for captures/AS/star.
    /// `raw_is_boxed` is true when `raw_subject` is a NaN-boxed MbValue (e.g., from
    /// `mb_seq_getitem`, `mb_dict_getitem`, or `GetAttr`) rather than a raw primitive.
    /// At the top level `raw_is_boxed=false` (subject was lowered as a raw primitive).
    fn emit_pattern_test(
        &mut self,
        subj_vreg: VReg,
        pattern: &HirPattern,
        fail_block: crate::mir::BlockId,
        raw_subject: VReg,
        raw_is_boxed: bool,
    ) {
        use HirPattern::*;
        match pattern {
            Wildcard => {
                // Always matches — nothing to emit
            }
            Capture(sym) => {
                // Determine the vreg to bind: if raw_subject is a boxed MbValue and the
                // capture has a primitive type, we must unbox before binding so that
                // arithmetic BinOps on the capture use the correct representation (#827).
                let capture_vreg = if raw_is_boxed {
                    let cap_ty = self
                        .sym_types
                        .get(sym)
                        .copied()
                        .unwrap_or_else(|| self.tcx.any());
                    let unbox_fn = if cap_ty == self.tcx.int() {
                        Some(("mb_unbox_int", cap_ty))
                    } else if cap_ty == self.tcx.bool() {
                        Some(("mb_unbox_bool", cap_ty))
                    } else if cap_ty == self.tcx.float() {
                        Some(("mb_unbox_float", cap_ty))
                    } else {
                        None
                    };
                    if let Some((fn_name, prim_ty)) = unbox_fn {
                        let unboxed = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(unboxed),
                            name: fn_name.to_string(),
                            args: vec![raw_subject],
                            ty: prim_ty,
                        });
                        unboxed
                    } else {
                        raw_subject
                    }
                } else {
                    raw_subject
                };
                // Bind the vreg so the arm body sees the correct primitive (#827 Issue 1).
                if let Some(&orig) = self.sym_to_vreg.get(sym) {
                    self.current_stmts.push(MirInst::Copy {
                        dest: orig,
                        source: capture_vreg,
                    });
                } else {
                    self.sym_to_vreg.insert(*sym, capture_vreg);
                }
            }
            Literal(expr) => {
                let lit_raw = self.lower_expr(expr);
                let eq = self.fresh_vreg();
                if matches!(expr, HirExpr::BoolLit(..)) {
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(eq),
                        name: "mb_match_bool_literal".to_string(),
                        args: vec![subj_vreg, lit_raw],
                        ty: self.tcx.bool(),
                    });
                } else {
                    // Box the literal so both operands are MbValue for mb_eq (#827 R1)
                    let lit = self.box_operand(lit_raw, expr.ty());
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(eq),
                        name: "mb_eq".to_string(),
                        args: vec![subj_vreg, lit],
                        ty: self.tcx.bool(),
                    });
                }
                let ok_block = self.fresh_block();
                self.finish_block(Terminator::Branch {
                    cond: eq,
                    then_block: ok_block,
                    else_block: fail_block,
                });
                self.start_block(ok_block);
            }
            Or(alternatives) => {
                // Collect all names bound by any alternative for merge vreg allocation
                // (#827 Issue 2: OR-pattern bindings must survive into shared success block).
                let bound_names: Vec<SymbolId> = {
                    let mut names = Vec::new();
                    for alt in alternatives.iter() {
                        collect_pattern_bindings(alt, &mut names);
                    }
                    // Deduplicate while preserving order
                    let mut seen = HashSet::new();
                    names.retain(|sym| seen.insert(*sym));
                    names
                };

                // Pre-allocate merge vregs for each bound name and register them
                // in sym_to_vreg so the arm body uses the merge vregs.
                let merge_vregs: Vec<(SymbolId, VReg)> = bound_names
                    .iter()
                    .map(|sym| {
                        let mv = self.fresh_vreg();
                        (*sym, mv)
                    })
                    .collect();

                // Save the pre-OR snapshot to restore between alternatives on failure.
                let pre_or_snapshot = self.sym_to_vreg.clone();

                // Register merge vregs in sym_to_vreg so the success block body sees them.
                for (sym, mv) in &merge_vregs {
                    self.sym_to_vreg.insert(*sym, *mv);
                }
                let post_or_sym_to_vreg = self.sym_to_vreg.clone();

                // Try each alternative in turn; succeed on first match (#827)
                let success = self.fresh_block();
                let mut next_blocks: Vec<crate::mir::BlockId> = alternatives
                    .iter()
                    .skip(1)
                    .map(|_| self.fresh_block())
                    .collect();
                next_blocks.push(fail_block);

                // Helper: after an alternative matches, emit copies to merge vregs.
                // Defined as a macro-like closure over merge_vregs and sym_to_vreg.

                for (i, alt) in alternatives.iter().enumerate() {
                    let next = next_blocks[i];
                    // Start each alternative from the pre-OR snapshot so merge vregs
                    // are visible but alternative-local bindings don't leak.
                    self.sym_to_vreg = pre_or_snapshot.clone();
                    // Also add the merge vregs so recursive calls can see them.
                    for (sym, mv) in &merge_vregs {
                        self.sym_to_vreg.insert(*sym, *mv);
                    }

                    match alt {
                        Literal(expr) => {
                            let lit_raw = self.lower_expr(expr);
                            let eq = self.fresh_vreg();
                            if matches!(expr, HirExpr::BoolLit(..)) {
                                self.current_stmts.push(MirInst::CallExtern {
                                    dest: Some(eq),
                                    name: "mb_match_bool_literal".to_string(),
                                    args: vec![subj_vreg, lit_raw],
                                    ty: self.tcx.bool(),
                                });
                            } else {
                                // Box the literal so both operands are MbValue for mb_eq (#827 R1)
                                let lit = self.box_operand(lit_raw, expr.ty());
                                self.current_stmts.push(MirInst::CallExtern {
                                    dest: Some(eq),
                                    name: "mb_eq".to_string(),
                                    args: vec![subj_vreg, lit],
                                    ty: self.tcx.bool(),
                                });
                            }
                            self.finish_block(Terminator::Branch {
                                cond: eq,
                                then_block: success,
                                else_block: next,
                            });
                        }
                        Wildcard | Capture(_) => {
                            // Always matches — register bindings then copy to merge vregs
                            self.emit_pattern_test(
                                subj_vreg,
                                alt,
                                fail_block,
                                raw_subject,
                                raw_is_boxed,
                            );
                            // Emit copies from alternative-local bindings to merge vregs
                            for (sym, mv) in &merge_vregs {
                                if let Some(&alt_vreg) = self.sym_to_vreg.get(sym) {
                                    if alt_vreg != *mv {
                                        self.current_stmts.push(MirInst::Copy {
                                            dest: *mv,
                                            source: alt_vreg,
                                        });
                                    }
                                }
                            }
                            self.finish_block(Terminator::Goto(success));
                        }
                        _ => {
                            // Complex nested OR alternative: alternative emits into a fresh
                            // binding environment; on success, copy to merge vregs.
                            let alt_ok = self.fresh_block();
                            let alt_fail = next;
                            self.emit_pattern_test(
                                subj_vreg,
                                alt,
                                alt_fail,
                                raw_subject,
                                raw_is_boxed,
                            );
                            self.finish_block(Terminator::Goto(alt_ok));
                            self.start_block(alt_ok);
                            // Emit copies from alternative-local bindings to merge vregs
                            for (sym, mv) in &merge_vregs {
                                if let Some(&alt_vreg) = self.sym_to_vreg.get(sym) {
                                    if alt_vreg != *mv {
                                        self.current_stmts.push(MirInst::Copy {
                                            dest: *mv,
                                            source: alt_vreg,
                                        });
                                    }
                                }
                            }
                            self.finish_block(Terminator::Goto(success));
                            if i + 1 < alternatives.len() {
                                self.start_block(next_blocks[i]);
                            }
                        }
                    }
                    if i + 1 < alternatives.len() {
                        // For Literal/Wildcard/Capture arms, start the next block here.
                        // For complex arms the block was already started above.
                        match alt {
                            Literal(_) | Wildcard | Capture(_) => {
                                self.start_block(next_blocks[i]);
                            }
                            _ => { /* already started inside the `_` arm above */ }
                        }
                    }
                }
                // Restore sym_to_vreg to the post-OR state (merge vregs registered)
                // so the success block body sees the correct bindings.
                self.sym_to_vreg = post_or_sym_to_vreg;
                self.start_block(success);
            }
            Sequence(pats) => {
                // Verify subject is a sequence before matching (#827)
                let is_seq = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(is_seq),
                    name: "mb_is_sequence".to_string(),
                    args: vec![subj_vreg],
                    ty: self.tcx.bool(),
                });
                let seq_ok = self.fresh_block();
                self.finish_block(Terminator::Branch {
                    cond: is_seq,
                    then_block: seq_ok,
                    else_block: fail_block,
                });
                self.start_block(seq_ok);

                // Find star pattern index (if any)
                let star_idx = pats.iter().position(|p| matches!(p, Star(_)));
                let len_vreg = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(len_vreg),
                    name: "mb_seq_len".to_string(),
                    args: vec![subj_vreg],
                    ty: self.tcx.int(),
                });

                if let Some(si) = star_idx {
                    // Has star: len >= prefix_count + suffix_count
                    let prefix_count = si;
                    let suffix_count = pats.len() - si - 1;
                    let min_len = prefix_count + suffix_count;
                    let min_len_vreg = self.fresh_vreg();
                    self.current_stmts.push(MirInst::LoadConst {
                        dest: min_len_vreg,
                        value: MirConst::Int(min_len as i64),
                        ty: self.tcx.int(),
                    });
                    let len_ok = self.fresh_vreg();
                    self.current_stmts.push(MirInst::BinOp {
                        dest: len_ok,
                        op: MirBinOp::GtEq,
                        lhs: len_vreg,
                        rhs: min_len_vreg,
                        ty: self.tcx.bool(),
                    });
                    let elem_block = self.fresh_block();
                    self.finish_block(Terminator::Branch {
                        cond: len_ok,
                        then_block: elem_block,
                        else_block: fail_block,
                    });
                    self.start_block(elem_block);
                    // Match prefix elements
                    for (i, pat) in pats[..si].iter().enumerate() {
                        let idx = self.fresh_vreg();
                        self.current_stmts.push(MirInst::LoadConst {
                            dest: idx,
                            value: MirConst::Int(i as i64),
                            ty: self.tcx.int(),
                        });
                        let elem = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(elem),
                            name: "mb_seq_getitem".to_string(),
                            args: vec![subj_vreg, idx],
                            ty: self.tcx.any(),
                        });
                        // elem is MbValue from mb_seq_getitem; raw_is_boxed=true so Capture
                        // arms will unbox primitives before binding (#827 nested capture fix).
                        self.emit_pattern_test(elem, pat, fail_block, elem, true);
                    }
                    // Bind star element (slice prefix..len-suffix)
                    let star_pat = &pats[si];
                    if let Star(Some(sym)) = star_pat {
                        let start_idx = self.fresh_vreg();
                        self.current_stmts.push(MirInst::LoadConst {
                            dest: start_idx,
                            value: MirConst::Int(prefix_count as i64),
                            ty: self.tcx.int(),
                        });
                        // stop = len - suffix_count
                        let suffix_vreg = self.fresh_vreg();
                        self.current_stmts.push(MirInst::LoadConst {
                            dest: suffix_vreg,
                            value: MirConst::Int(suffix_count as i64),
                            ty: self.tcx.int(),
                        });
                        let stop_idx = self.fresh_vreg();
                        self.current_stmts.push(MirInst::BinOp {
                            dest: stop_idx,
                            op: MirBinOp::Sub,
                            lhs: len_vreg,
                            rhs: suffix_vreg,
                            ty: self.tcx.int(),
                        });
                        // mb_seq_slice takes MbValue args (NaN-boxed ints)
                        let start_mb = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(start_mb),
                            name: "mb_box_int".to_string(),
                            args: vec![start_idx],
                            ty: self.tcx.any(),
                        });
                        let stop_mb = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(stop_mb),
                            name: "mb_box_int".to_string(),
                            args: vec![stop_idx],
                            ty: self.tcx.any(),
                        });
                        let slice = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(slice),
                            name: "mb_seq_slice".to_string(),
                            args: vec![subj_vreg, start_mb, stop_mb],
                            ty: self.tcx.any(),
                        });
                        if let Some(&orig) = self.sym_to_vreg.get(sym) {
                            self.current_stmts.push(MirInst::Copy {
                                dest: orig,
                                source: slice,
                            });
                        } else {
                            self.sym_to_vreg.insert(*sym, slice);
                        }
                    }
                    // Match suffix elements (from end)
                    for (i, pat) in pats[si + 1..].iter().enumerate() {
                        // index = len - suffix_count + i = len - (suffix_count - i)
                        let neg_off = suffix_count - i - 1; // offset from end, 0-based
                        let neg_vreg = self.fresh_vreg();
                        self.current_stmts.push(MirInst::LoadConst {
                            dest: neg_vreg,
                            value: MirConst::Int(neg_off as i64 + 1),
                            ty: self.tcx.int(),
                        });
                        let idx = self.fresh_vreg();
                        self.current_stmts.push(MirInst::BinOp {
                            dest: idx,
                            op: MirBinOp::Sub,
                            lhs: len_vreg,
                            rhs: neg_vreg,
                            ty: self.tcx.int(),
                        });
                        let elem = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(elem),
                            name: "mb_seq_getitem".to_string(),
                            args: vec![subj_vreg, idx],
                            ty: self.tcx.any(),
                        });
                        self.emit_pattern_test(elem, pat, fail_block, elem, true);
                    }
                } else {
                    // No star: exact length check
                    let expected = self.fresh_vreg();
                    self.current_stmts.push(MirInst::LoadConst {
                        dest: expected,
                        value: MirConst::Int(pats.len() as i64),
                        ty: self.tcx.int(),
                    });
                    let len_eq = self.fresh_vreg();
                    self.current_stmts.push(MirInst::BinOp {
                        dest: len_eq,
                        op: MirBinOp::Eq,
                        lhs: len_vreg,
                        rhs: expected,
                        ty: self.tcx.bool(),
                    });
                    let elem_block = self.fresh_block();
                    self.finish_block(Terminator::Branch {
                        cond: len_eq,
                        then_block: elem_block,
                        else_block: fail_block,
                    });
                    self.start_block(elem_block);
                    for (i, pat) in pats.iter().enumerate() {
                        let idx = self.fresh_vreg();
                        self.current_stmts.push(MirInst::LoadConst {
                            dest: idx,
                            value: MirConst::Int(i as i64),
                            ty: self.tcx.int(),
                        });
                        let elem = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(elem),
                            name: "mb_seq_getitem".to_string(),
                            args: vec![subj_vreg, idx],
                            ty: self.tcx.any(),
                        });
                        // Recursively test nested element pattern (R7)
                        // elem is MbValue from mb_seq_getitem; raw_is_boxed=true (#827).
                        self.emit_pattern_test(elem, pat, fail_block, elem, true);
                    }
                }
            }
            Class {
                class: _,
                class_name,
                args,
            } => {
                // Built-in self-subject types: positional captures bind the outer subject.
                let is_builtin_self_subject = matches!(
                    class_name.as_str(),
                    "int" | "bool" | "str" | "float" | "list" | "tuple" | "dict"
                );
                // Emit isinstance check (#827, R8)
                let class_str = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest: class_str,
                    value: MirConst::Str(class_name.clone()),
                    ty: self.tcx.str(),
                });
                let is_inst = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(is_inst),
                    name: "mb_isinstance".to_string(),
                    args: vec![subj_vreg, class_str],
                    ty: self.tcx.bool(),
                });
                let attr_block = self.fresh_block();
                self.finish_block(Terminator::Branch {
                    cond: is_inst,
                    then_block: attr_block,
                    else_block: fail_block,
                });
                self.start_block(attr_block);
                for (attr_name, pat) in args {
                    let attr_val = self.fresh_vreg();
                    // Detect positional args: synthetic names like "_0", "_1" (#827)
                    let is_positional = attr_name.starts_with('_')
                        && attr_name.len() > 1
                        && attr_name[1..].chars().all(|c| c.is_ascii_digit());
                    if is_positional {
                        let pos_idx = attr_name[1..].parse::<i64>().unwrap_or(0);
                        let pos_vreg = self.fresh_vreg();
                        self.current_stmts.push(MirInst::LoadConst {
                            dest: pos_vreg,
                            value: MirConst::Int(pos_idx),
                            ty: self.tcx.int(),
                        });
                        // Fail if class doesn't have __match_args__[pos] (#827)
                        let has_pos = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(has_pos),
                            name: "mb_class_has_pos_match".to_string(),
                            args: vec![subj_vreg, class_str, pos_vreg],
                            ty: self.tcx.bool(),
                        });
                        let pos_ok = self.fresh_block();
                        self.finish_block(Terminator::Branch {
                            cond: has_pos,
                            then_block: pos_ok,
                            else_block: fail_block,
                        });
                        self.start_block(pos_ok);
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(attr_val),
                            name: "mb_match_pos_arg".to_string(),
                            args: vec![subj_vreg, class_str, pos_vreg],
                            ty: self.tcx.any(),
                        });
                    } else {
                        // Fail if keyword attribute doesn't exist on subject (#827)
                        let attr_str = self.fresh_vreg();
                        self.current_stmts.push(MirInst::LoadConst {
                            dest: attr_str,
                            value: MirConst::Str(attr_name.clone()),
                            ty: self.tcx.str(),
                        });
                        let has_attr = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(has_attr),
                            name: "mb_instance_hasattr".to_string(),
                            args: vec![subj_vreg, attr_str],
                            ty: self.tcx.bool(),
                        });
                        let attr_ok = self.fresh_block();
                        self.finish_block(Terminator::Branch {
                            cond: has_attr,
                            then_block: attr_ok,
                            else_block: fail_block,
                        });
                        self.start_block(attr_ok);
                        self.current_stmts.push(MirInst::GetAttr {
                            dest: attr_val,
                            object: subj_vreg,
                            attr: attr_name.clone(),
                            ty: self.tcx.any(),
                        });
                    }
                    // Recursively test nested attribute pattern (R7)
                    // For built-in self-subject types, positional captures refer to the
                    // original subject (e.g. `case int(x):` binds the int itself), so
                    // pass raw_subject as the raw vreg (not boxed) instead of attr_val.
                    let (pat_raw, pat_raw_is_boxed) = if is_builtin_self_subject && is_positional {
                        // raw_subject is already an unboxed primitive
                        (raw_subject, raw_is_boxed)
                    } else {
                        // attr_val is MbValue from GetAttr/mb_match_pos_arg; boxed
                        (attr_val, true)
                    };
                    self.emit_pattern_test(attr_val, pat, fail_block, pat_raw, pat_raw_is_boxed);
                }
            }
            Mapping { pairs, rest } => {
                // Verify subject is a mapping (dict) before pattern matching (#827)
                let is_map = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(is_map),
                    name: "mb_is_mapping".to_string(),
                    args: vec![subj_vreg],
                    ty: self.tcx.bool(),
                });
                let map_ok = self.fresh_block();
                self.finish_block(Terminator::Branch {
                    cond: is_map,
                    then_block: map_ok,
                    else_block: fail_block,
                });
                self.start_block(map_ok);
                let mut matched_keys = Vec::new();
                for (key_expr, val_pat) in pairs {
                    let key_raw = self.lower_expr(key_expr);
                    // Box primitive keys (int/bool) so dict functions receive MbValue (#827)
                    let key_vreg = self.box_operand(key_raw, key_expr.ty());
                    matched_keys.push((key_vreg, val_pat));
                }
                // PEP 634: duplicate mapping pattern keys that compare equal at
                // runtime raise ValueError before matching values.
                for i in 0..matched_keys.len() {
                    for j in (i + 1)..matched_keys.len() {
                        let duplicate = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(duplicate),
                            name: "mb_eq".to_string(),
                            args: vec![matched_keys[i].0, matched_keys[j].0],
                            ty: self.tcx.bool(),
                        });
                        let raise_dup = self.fresh_block();
                        let next_cmp = self.fresh_block();
                        self.finish_block(Terminator::Branch {
                            cond: duplicate,
                            then_block: raise_dup,
                            else_block: next_cmp,
                        });
                        self.start_block(raise_dup);
                        let err_type = self.emit_str_const("ValueError");
                        let err_msg = self.emit_str_const("mapping pattern checks duplicate key");
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: None,
                            name: "mb_raise".to_string(),
                            args: vec![err_type, err_msg],
                            ty: self.tcx.none(),
                        });
                        self.emit_exception_propagate();
                        self.finish_block(Terminator::Goto(next_cmp));
                        self.start_block(next_cmp);
                    }
                }
                // For each key: check presence + value pattern; then collect rest (#827, R3)
                for (key_vreg, val_pat) in &matched_keys {
                    // Check: key in subject dict
                    let has_key = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(has_key),
                        name: "mb_dict_contains".to_string(),
                        args: vec![subj_vreg, *key_vreg],
                        ty: self.tcx.bool(),
                    });
                    let key_ok = self.fresh_block();
                    self.finish_block(Terminator::Branch {
                        cond: has_key,
                        then_block: key_ok,
                        else_block: fail_block,
                    });
                    self.start_block(key_ok);
                    // Extract value and recursively test value pattern
                    let val_vreg = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(val_vreg),
                        name: "mb_dict_getitem".to_string(),
                        args: vec![subj_vreg, *key_vreg],
                        ty: self.tcx.any(),
                    });
                    // val_vreg is MbValue from mb_dict_getitem; raw_is_boxed=true (#827).
                    self.emit_pattern_test(val_vreg, val_pat, fail_block, val_vreg, true);
                }
                // Bind rest-capture dict if present (#827, R3)
                if let Some(rest_sym) = rest {
                    // Start with a copy of the subject dict
                    let rest_dict = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(rest_dict),
                        name: "mb_dict_copy".to_string(),
                        args: vec![subj_vreg],
                        ty: self.tcx.any(),
                    });
                    // Delete each matched key from the copy
                    for (key_v, _) in &matched_keys {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: None,
                            name: "mb_dict_delitem".to_string(),
                            args: vec![rest_dict, *key_v],
                            ty: self.tcx.none(),
                        });
                    }
                    if let Some(&orig) = self.sym_to_vreg.get(rest_sym) {
                        self.current_stmts.push(MirInst::Copy {
                            dest: orig,
                            source: rest_dict,
                        });
                    } else {
                        self.sym_to_vreg.insert(*rest_sym, rest_dict);
                    }
                }
            }
            Star(sym_opt) => {
                // Star in sequence context: bind to sym if named.
                // Use raw_subject so arithmetic on the star alias uses the correct
                // primitive representation (#827 Issue 1).
                if let Some(sym) = sym_opt {
                    if let Some(&orig) = self.sym_to_vreg.get(sym) {
                        self.current_stmts.push(MirInst::Copy {
                            dest: orig,
                            source: raw_subject,
                        });
                    } else {
                        self.sym_to_vreg.insert(*sym, raw_subject);
                    }
                }
                // Always matches
            }
            As {
                pattern: inner,
                name,
            } => {
                // Match inner pattern then bind subject to `name` (#827, R2).
                // Pass raw_subject/raw_is_boxed through so the inner Capture/Star
                // also unbox correctly.
                self.emit_pattern_test(subj_vreg, inner, fail_block, raw_subject, raw_is_boxed);
                // Bind `name`: same unboxing logic as Capture (#827 nested capture fix).
                let as_vreg = if raw_is_boxed {
                    let cap_ty = self
                        .sym_types
                        .get(name)
                        .copied()
                        .unwrap_or_else(|| self.tcx.any());
                    let unbox_fn = if cap_ty == self.tcx.int() {
                        Some(("mb_unbox_int", cap_ty))
                    } else if cap_ty == self.tcx.bool() {
                        Some(("mb_unbox_bool", cap_ty))
                    } else if cap_ty == self.tcx.float() {
                        Some(("mb_unbox_float", cap_ty))
                    } else {
                        None
                    };
                    if let Some((fn_name, prim_ty)) = unbox_fn {
                        let unboxed = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(unboxed),
                            name: fn_name.to_string(),
                            args: vec![raw_subject],
                            ty: prim_ty,
                        });
                        unboxed
                    } else {
                        raw_subject
                    }
                } else {
                    raw_subject
                };
                if let Some(&orig) = self.sym_to_vreg.get(name) {
                    self.current_stmts.push(MirInst::Copy {
                        dest: orig,
                        source: as_vreg,
                    });
                } else {
                    self.sym_to_vreg.insert(*name, as_vreg);
                }
            }
        }
    }

    /// Lower match/case statement (#309, #827).
    ///
    /// Desugars to a chain of if/else blocks, one per case arm.
    /// Uses `emit_pattern_test` for recursive decision-tree generation (R7).
    fn lower_match(&mut self, subject: &HirExpr, cases: &[HirMatchCase]) {
        let subj_raw = self.lower_expr(subject);
        // Box primitive subjects so pattern tests receive uniform MbValue (#827 R1)
        let subj = self.box_operand(subj_raw, subject.ty());
        let merge_block = self.fresh_block();

        for case in cases {
            let case_body_block = self.fresh_block();
            let next_case_block = self.fresh_block();

            // Emit the pattern test: falls through on match, jumps to next_case on fail.
            // Wildcard always falls through unconditionally.
            // Pass subj_raw as raw_subject (raw_is_boxed=false): at top level, the
            // subject was lowered as a raw primitive vreg (#827 Issue 1).
            let is_wildcard = matches!(&case.pattern, HirPattern::Wildcard);
            self.emit_pattern_test(subj, &case.pattern, next_case_block, subj_raw, false);

            // Apply guard if present
            if let Some(guard) = &case.guard {
                let g = self.lower_cond_as_bool(guard);
                self.finish_block(Terminator::Branch {
                    cond: g,
                    then_block: case_body_block,
                    else_block: next_case_block,
                });
            } else {
                self.finish_block(Terminator::Goto(case_body_block));
            }

            self.start_block(case_body_block);
            for s in &case.body {
                self.lower_stmt(s);
            }
            self.finish_block(Terminator::Goto(merge_block));

            self.start_block(next_case_block);

            // Only suppress later arms for unconditional wildcards (#827)
            // A guarded wildcard (`case _ if cond:`) can still fall through when cond is false.
            if is_wildcard && case.guard.is_none() {
                break;
            }
        }

        // If no case matched, fall through to merge
        self.finish_block(Terminator::Goto(merge_block));
        self.start_block(merge_block);
    }

    /// Lower a comprehension's generators into a nested loop structure,
    /// calling `emit_body` for each iteration.
    fn lower_comprehension_loops(
        &mut self,
        generators: &[HirComprehension],
        mut emit_body: impl FnMut(&mut Self),
    ) {
        if generators.is_empty() {
            emit_body(self);
            return;
        }
        let gen = &generators[0];
        let rest = &generators[1..];
        let gen_var = gen.var;
        let extra_vars = gen.extra_vars.clone();

        // P0-R5: Save outer binding for comprehension scope isolation.
        // Comprehension loop variables must not leak into the enclosing scope.
        let saved_binding = self.sym_to_vreg.get(&gen_var).copied();
        let saved_extras: Vec<(SymbolId, Option<VReg>)> = extra_vars
            .iter()
            .map(|&sym| (sym, self.sym_to_vreg.get(&sym).copied()))
            .collect();

        let iterable = self.lower_expr(&gen.iter);
        let iter_obj = self.fresh_vreg();
        self.current_stmts.push(MirInst::CallExtern {
            dest: Some(iter_obj),
            name: "mb_iter".to_string(),
            args: vec![iterable],
            ty: self.tcx.any(),
        });

        let header = self.fresh_block();
        let body_block = self.fresh_block();
        let exit_block = self.fresh_block();

        self.finish_block(Terminator::Goto(header));
        self.start_block(header);
        // Lever A: single-call advance + sentinel check.
        let next_val = self.fresh_vreg();
        self.current_stmts.push(MirInst::CallExtern {
            dest: Some(next_val),
            name: "mb_next_or_stop".to_string(),
            args: vec![iter_obj],
            ty: self.tcx.any(),
        });
        let is_stop = self.fresh_vreg();
        self.current_stmts.push(MirInst::CallExtern {
            dest: Some(is_stop),
            name: "mb_is_stop_iter".to_string(),
            args: vec![next_val],
            ty: self.tcx.bool(),
        });
        self.finish_block(Terminator::Branch {
            cond: is_stop,
            then_block: exit_block,
            else_block: body_block,
        });

        self.start_block(body_block);
        // Unpack tuple/sequence targets: `for k, v in pairs` or `for (v,) in pairs`.
        if !gen.unpack_target {
            self.sym_to_vreg.insert(gen_var, next_val);
        } else {
            let any_ty = self.tcx.any();
            // First target
            let first = self.fresh_vreg();
            let idx0 = self.fresh_vreg();
            self.current_stmts.push(MirInst::LoadConst {
                dest: idx0,
                value: MirConst::Int(0),
                ty: self.tcx.int(),
            });
            self.current_stmts.push(MirInst::CallExtern {
                dest: Some(first),
                name: "mb_seq_getitem".to_string(),
                args: vec![next_val, idx0],
                ty: any_ty,
            });
            self.sym_to_vreg.insert(gen_var, first);
            // Remaining targets
            for (i, &sym) in extra_vars.iter().enumerate() {
                let idx = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest: idx,
                    value: MirConst::Int((i + 1) as i64),
                    ty: self.tcx.int(),
                });
                let val = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(val),
                    name: "mb_seq_getitem".to_string(),
                    args: vec![next_val, idx],
                    ty: any_ty,
                });
                self.sym_to_vreg.insert(sym, val);
            }
        }

        // Apply conditions (if clauses)
        if gen.conditions.is_empty() {
            // No conditions — recurse directly
            self.lower_comprehension_loops(rest, emit_body);
        } else {
            for condition in &gen.conditions {
                let cond_vreg = self.lower_cond_as_bool(condition);
                let next_condition = self.fresh_block();
                let skip_block = self.fresh_block();
                self.finish_block(Terminator::Branch {
                    cond: cond_vreg,
                    then_block: next_condition,
                    else_block: skip_block,
                });
                self.start_block(skip_block);
                self.finish_block(Terminator::Goto(header));
                self.start_block(next_condition);
            }
            self.lower_comprehension_loops(rest, emit_body);
            self.finish_block(Terminator::Goto(header));
            // Exit block
            self.start_block(exit_block);
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_iter_release".to_string(),
                args: vec![iter_obj],
                ty: self.tcx.none(),
            });
            // P0-R5: Restore outer binding after comprehension
            match saved_binding {
                Some(vreg) => {
                    self.sym_to_vreg.insert(gen_var, vreg);
                }
                None => {
                    self.sym_to_vreg.remove(&gen_var);
                }
            }
            for (sym, saved) in &saved_extras {
                match saved {
                    Some(vreg) => {
                        self.sym_to_vreg.insert(*sym, *vreg);
                    }
                    None => {
                        self.sym_to_vreg.remove(sym);
                    }
                }
            }
            return;
        }

        self.finish_block(Terminator::Goto(header));
        self.start_block(exit_block);
        self.current_stmts.push(MirInst::CallExtern {
            dest: None,
            name: "mb_iter_release".to_string(),
            args: vec![iter_obj],
            ty: self.tcx.none(),
        });
        // P0-R5: Restore outer binding after comprehension
        match saved_binding {
            Some(vreg) => {
                self.sym_to_vreg.insert(gen_var, vreg);
            }
            None => {
                self.sym_to_vreg.remove(&gen_var);
            }
        }
        for (sym, saved) in &saved_extras {
            match saved {
                Some(vreg) => {
                    self.sym_to_vreg.insert(*sym, *vreg);
                }
                None => {
                    self.sym_to_vreg.remove(sym);
                }
            }
        }
    }

    fn emit_bool_result(&mut self, result: VReg, value: bool, merge_block: BlockId) {
        let raw = self.fresh_vreg();
        self.current_stmts.push(MirInst::LoadConst {
            dest: raw,
            value: MirConst::Bool(value),
            ty: self.tcx.bool(),
        });
        self.current_stmts.push(MirInst::Copy {
            dest: result,
            source: raw,
        });
        self.finish_block(Terminator::Goto(merge_block));
    }

    fn lower_any_all_comp(
        &mut self,
        is_all: bool,
        element: &HirExpr,
        generators: &[HirComprehension],
    ) -> VReg {
        let result = self.fresh_vreg();
        let short_block = self.fresh_block();
        let merge_block = self.fresh_block();
        let mut cleanup_iters = Vec::new();

        self.lower_any_all_comp_loops(generators, element, is_all, short_block, &mut cleanup_iters);
        self.emit_bool_result(result, is_all, merge_block);

        self.start_block(short_block);
        for iter_obj in cleanup_iters.iter().rev() {
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_iter_release".to_string(),
                args: vec![*iter_obj],
                ty: self.tcx.none(),
            });
        }
        self.emit_bool_result(result, !is_all, merge_block);

        self.start_block(merge_block);
        result
    }

    fn lower_any_all_comp_loops(
        &mut self,
        generators: &[HirComprehension],
        element: &HirExpr,
        is_all: bool,
        short_block: BlockId,
        cleanup_iters: &mut Vec<VReg>,
    ) {
        if generators.is_empty() {
            let value = self.lower_expr(element);
            let truthy = self.fresh_vreg();
            self.current_stmts.push(MirInst::CallExtern {
                dest: Some(truthy),
                name: "mb_is_truthy".to_string(),
                args: vec![value],
                ty: self.tcx.int(),
            });
            let continue_block = self.fresh_block();
            let (then_block, else_block) = if is_all {
                (continue_block, short_block)
            } else {
                (short_block, continue_block)
            };
            self.finish_block(Terminator::Branch {
                cond: truthy,
                then_block,
                else_block,
            });
            self.start_block(continue_block);
            return;
        }

        let gen = &generators[0];
        let rest = &generators[1..];
        let gen_var = gen.var;
        let extra_vars = gen.extra_vars.clone();

        let saved_binding = self.sym_to_vreg.get(&gen_var).copied();
        let saved_extras: Vec<(SymbolId, Option<VReg>)> = extra_vars
            .iter()
            .map(|&sym| (sym, self.sym_to_vreg.get(&sym).copied()))
            .collect();

        let iterable = self.lower_expr(&gen.iter);
        let iter_obj = self.fresh_vreg();
        self.current_stmts.push(MirInst::CallExtern {
            dest: Some(iter_obj),
            name: "mb_iter".to_string(),
            args: vec![iterable],
            ty: self.tcx.any(),
        });
        cleanup_iters.push(iter_obj);

        let header = self.fresh_block();
        let body_block = self.fresh_block();
        let exit_block = self.fresh_block();

        self.finish_block(Terminator::Goto(header));
        self.start_block(header);
        let next_val = self.fresh_vreg();
        self.current_stmts.push(MirInst::CallExtern {
            dest: Some(next_val),
            name: "mb_next_or_stop".to_string(),
            args: vec![iter_obj],
            ty: self.tcx.any(),
        });
        let is_stop = self.fresh_vreg();
        self.current_stmts.push(MirInst::CallExtern {
            dest: Some(is_stop),
            name: "mb_is_stop_iter".to_string(),
            args: vec![next_val],
            ty: self.tcx.bool(),
        });
        self.finish_block(Terminator::Branch {
            cond: is_stop,
            then_block: exit_block,
            else_block: body_block,
        });

        self.start_block(body_block);
        if extra_vars.is_empty() {
            self.sym_to_vreg.insert(gen_var, next_val);
        } else {
            let any_ty = self.tcx.any();
            let idx0 = self.fresh_vreg();
            self.current_stmts.push(MirInst::LoadConst {
                dest: idx0,
                value: MirConst::Int(0),
                ty: self.tcx.int(),
            });
            let first = self.fresh_vreg();
            self.current_stmts.push(MirInst::CallExtern {
                dest: Some(first),
                name: "mb_seq_getitem".to_string(),
                args: vec![next_val, idx0],
                ty: any_ty,
            });
            self.sym_to_vreg.insert(gen_var, first);
            for (i, &sym) in extra_vars.iter().enumerate() {
                let idx = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest: idx,
                    value: MirConst::Int((i + 1) as i64),
                    ty: self.tcx.int(),
                });
                let val = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(val),
                    name: "mb_seq_getitem".to_string(),
                    args: vec![next_val, idx],
                    ty: any_ty,
                });
                self.sym_to_vreg.insert(sym, val);
            }
        }

        let skip_block = self.fresh_block();
        for cond in &gen.conditions {
            let cond_vreg = self.lower_cond_as_bool(cond);
            let pass_block = self.fresh_block();
            self.finish_block(Terminator::Branch {
                cond: cond_vreg,
                then_block: pass_block,
                else_block: skip_block,
            });
            self.start_block(pass_block);
        }

        self.lower_any_all_comp_loops(rest, element, is_all, short_block, cleanup_iters);
        self.finish_block(Terminator::Goto(header));

        self.start_block(skip_block);
        self.finish_block(Terminator::Goto(header));

        self.start_block(exit_block);
        self.current_stmts.push(MirInst::CallExtern {
            dest: None,
            name: "mb_iter_release".to_string(),
            args: vec![iter_obj],
            ty: self.tcx.none(),
        });

        match saved_binding {
            Some(vreg) => {
                self.sym_to_vreg.insert(gen_var, vreg);
            }
            None => {
                self.sym_to_vreg.remove(&gen_var);
            }
        }
        for (sym, saved) in &saved_extras {
            match saved {
                Some(vreg) => {
                    self.sym_to_vreg.insert(*sym, *vreg);
                }
                None => {
                    self.sym_to_vreg.remove(sym);
                }
            }
        }
    }

    fn lower_expr(&mut self, expr: &HirExpr) -> VReg {
        match expr {
            HirExpr::IntLit(i, ty) => {
                let dest = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest,
                    value: MirConst::Int(*i),
                    ty: *ty,
                });
                dest
            }
            HirExpr::FloatLit(f, ty) => {
                let dest = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest,
                    value: MirConst::Float(*f),
                    ty: *ty,
                });
                dest
            }
            HirExpr::StrLit(s, ty) => {
                let dest = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest,
                    value: MirConst::Str(s.clone()),
                    ty: *ty,
                });
                dest
            }
            HirExpr::BytesLit(bytes, ty) => {
                let dest = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest,
                    value: MirConst::Bytes(bytes.clone()),
                    ty: *ty,
                });
                dest
            }
            HirExpr::BoolLit(b, ty) => {
                let dest = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest,
                    value: MirConst::Bool(*b),
                    ty: *ty,
                });
                dest
            }
            HirExpr::NoneLit(ty) => {
                let dest = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest,
                    value: MirConst::None,
                    ty: *ty,
                });
                dest
            }
            HirExpr::Var(sym, ty) => {
                // NotImplemented / Ellipsis builtin constants → emit as MirConst
                if let Some(st) = self.symbol_table {
                    if (sym.0 as usize) < st.all_symbols().len() {
                        let sym_name = &st.get_symbol(*sym).name;
                        let builtin_const = match sym_name.as_str() {
                            "NotImplemented" => Some(MirConst::NotImplemented),
                            "Ellipsis" => Some(MirConst::Ellipsis),
                            _ => None,
                        };
                        if let Some(value) = builtin_const {
                            let dest = self.fresh_vreg();
                            self.current_stmts.push(MirInst::LoadConst {
                                dest,
                                value,
                                ty: *ty,
                            });
                            return dest;
                        }
                    }
                }
                // Exception type names (ValueError, TypeError, etc.) → emit as string constants
                // so issubclass(ValueError, Exception) receives string MbValues.
                // Builtin primitive type names (bool, int, list, …) → emit a call to
                // mb_builtin_type_obj() so the result is the *same* singleton as
                // returned by mb_type(), making `type(True) is bool` → True.
                if let Some(class_name) = self.class_syms.get(&sym.0).cloned() {
                    const BUILTIN_TYPE_NAMES: &[&str] = &[
                        "int",
                        "float",
                        "str",
                        "bool",
                        "list",
                        "dict",
                        "set",
                        "tuple",
                        "bytes",
                        "bytearray",
                        "frozenset",
                        "type",
                        "object",
                    ];
                    if BUILTIN_TYPE_NAMES.contains(&class_name.as_str()) {
                        // Emit: mb_builtin_type_obj(<string_const_name>)
                        let name_vreg = self.emit_str_const(&class_name);
                        let dest = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_builtin_type_obj".to_string(),
                            args: vec![name_vreg],
                            ty: self.tcx.any(),
                        });
                        return dest;
                    }
                    // Exception type names stay name-strings; their `is`
                    // identity is handled name-wise in mb_values_identical
                    // (CPython: builtin types are singletons).
                    return self.emit_str_const(&class_name);
                }
                // Builtin function symbols in non-call position (e.g. map(abs, [...])) →
                // load the canonical object from the `builtins` module so
                // `builtins.all is all` and callable metadata match CPython.
                if self.builtin_syms.contains_key(&sym.0) {
                    if let Some(st) = self.symbol_table {
                        let builtin_name = st.get_symbol(*sym).name.clone();
                        let name_vreg = self.emit_str_const(&builtin_name);
                        let dest = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_builtin_get".to_string(),
                            args: vec![name_vreg],
                            ty: self.tcx.any(),
                        });
                        return dest;
                    }
                    let extern_name = self.builtin_syms.get(&sym.0).cloned().unwrap();
                    let dest = self.fresh_vreg();
                    self.current_stmts.push(MirInst::LoadConst {
                        dest,
                        value: MirConst::ExternFuncRef(extern_name),
                        ty: *ty,
                    });
                    return dest;
                }
                // User function symbols in non-call position (e.g. callable(my_func)) →
                // emit FuncRef so the callee receives a TAG_FUNC-tagged pointer.
                // EXCEPT if the function is decorated: the decorator may have
                // replaced the global binding with a wrapper Instance (e.g.
                // `@lru_cache def double(n): ...` → `double` is an
                // `functools.lru_cache_wrapper`). Attribute access like
                // `double.cache_info` must see that wrapper, not the raw
                // FuncRef. Load from the global namespace instead.
                if self.user_funcs.contains(&sym.0) {
                    let dest = self.fresh_vreg();
                    if self.decorated_func_syms.contains(&sym.0) {
                        self.current_stmts.push(MirInst::LoadGlobal {
                            dest,
                            name: *sym,
                            ty: *ty,
                        });
                    } else {
                        self.current_stmts.push(MirInst::LoadConst {
                            dest,
                            value: MirConst::FuncRef(*sym),
                            ty: *ty,
                        });
                    }
                    return dest;
                }
                // Check cell_override first: synthetic (1M+) nonlocal-shared symbols
                // always load from global storage regardless of symbol_table classification.
                if self.cell_override.contains(&sym.0) {
                    let dest = self.fresh_vreg();
                    self.current_stmts.push(MirInst::LoadGlobal {
                        dest,
                        name: *sym,
                        ty: *ty,
                    });
                    return dest;
                }
                // Check variable classification for global/cell/free access
                let var_class = self
                    .symbol_table
                    .map(|st| st.get_var_class(*sym))
                    .unwrap_or(VariableClass::Local);
                match var_class {
                    VariableClass::Global => {
                        let dest = self.fresh_vreg();
                        self.current_stmts.push(MirInst::LoadGlobal {
                            dest,
                            name: *sym,
                            ty: *ty,
                        });
                        dest
                    }
                    VariableClass::Free => {
                        // Free variables are loaded from the outer Cell variable via global storage.
                        let outer_sym =
                            self.symbol_table.and_then(|st| st.get_nonlocal_outer(*sym));
                        if let Some(outer) = outer_sym {
                            let dest = self.fresh_vreg();
                            self.current_stmts.push(MirInst::LoadGlobal {
                                dest,
                                name: outer,
                                ty: *ty,
                            });
                            dest
                        } else {
                            // Fallback: treat as local
                            self.sym_to_vreg.get(sym).copied().unwrap_or_else(|| {
                                let dest = self.fresh_vreg();
                                self.sym_to_vreg.insert(*sym, dest);
                                dest
                            })
                        }
                    }
                    VariableClass::Cell => {
                        // Cell variables are captured by inner functions — use global storage to
                        // allow inner functions to observe mutations.
                        let dest = self.fresh_vreg();
                        self.current_stmts.push(MirInst::LoadGlobal {
                            dest,
                            name: *sym,
                            ty: *ty,
                        });
                        dest
                    }
                    VariableClass::Local => {
                        if let Some(&vreg) = self.sym_to_vreg.get(sym) {
                            vreg
                        } else if !self.in_module_scope {
                            // Inside a function body: the variable is not a local param/let.
                            // Fall back to LoadGlobal — this handles module-level variables
                            // read without a `global` declaration (valid Python, implicit
                            // global read; the resolver leaves them as Local).
                            let dest = self.fresh_vreg();
                            self.current_stmts.push(MirInst::LoadGlobal {
                                dest,
                                name: *sym,
                                ty: *ty,
                            });
                            dest
                        } else if self
                            .symbol_table
                            .and_then(|st| st.lookup("__name__"))
                            .map_or(false, |name_sym| name_sym == *sym)
                        {
                            // Module scope: __name__ dunder variable (#1133).
                            // Emit "__main__" string constant instead of uninitialized VReg.
                            // Also emit StoreGlobal so inner functions can access it
                            // via LoadGlobal.
                            let dest = self.emit_str_const("__main__");
                            self.current_stmts.push(MirInst::StoreGlobal {
                                name: *sym,
                                value: dest,
                            });
                            self.sym_to_vreg.insert(*sym, dest);
                            dest
                        } else {
                            // Module scope: variable not yet assigned (use before
                            // define). Read it from global storage rather than
                            // allocating a bare VReg with no source — `__file__`,
                            // `__doc__`, `__package__` and similar dunders, as
                            // well as ordinary use-before-define globals, would
                            // otherwise reference an uninitialised Cranelift
                            // variable and trip `self.map[&vreg]` in the codegen
                            // backend (compiled_blob.rs:90 panic on
                            // `[__file__]`-style list literals). LoadGlobal
                            // returns None when the symbol has not been set.
                            let dest = self.fresh_vreg();
                            self.current_stmts.push(MirInst::LoadGlobal {
                                dest,
                                name: *sym,
                                ty: *ty,
                            });
                            dest
                        }
                    }
                }
            }
            HirExpr::BinOp { op, lhs, rhs, ty } => {
                // Python and/or: short-circuit, return operand value.
                // a and b → evaluate a; if falsy return a, else evaluate and return b
                // a or b  → evaluate a; if truthy return a, else evaluate and return b
                if matches!(op, HirBinOp::And | HirBinOp::Or) {
                    let l = self.lower_expr(lhs);
                    let boxed_l = self.box_operand(l, lhs.ty());
                    // Python truthiness test — handles int, float, str, list, etc.
                    let truthy = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(truthy),
                        name: "mb_is_truthy".to_string(),
                        args: vec![boxed_l],
                        ty: self.tcx.int(),
                    });
                    let result = self.fresh_vreg();
                    let eval_rhs = self.fresh_block();
                    let merge = self.fresh_block();
                    // Copy l into result (used if we short-circuit)
                    self.current_stmts.push(MirInst::Copy {
                        dest: result,
                        source: boxed_l,
                    });
                    if matches!(op, HirBinOp::And) {
                        // and: if l is falsy, short-circuit (keep l); else evaluate rhs
                        self.finish_block(Terminator::Branch {
                            cond: truthy,
                            then_block: eval_rhs,
                            else_block: merge,
                        });
                    } else {
                        // or: if l is truthy, short-circuit (keep l); else evaluate rhs
                        self.finish_block(Terminator::Branch {
                            cond: truthy,
                            then_block: merge,
                            else_block: eval_rhs,
                        });
                    }
                    self.start_block(eval_rhs);
                    let r = self.lower_expr(rhs);
                    let boxed_r = self.box_operand(r, rhs.ty());
                    self.current_stmts.push(MirInst::Copy {
                        dest: result,
                        source: boxed_r,
                    });
                    self.finish_block(Terminator::Goto(merge));
                    self.start_block(merge);
                    return result;
                }

                let l = self.lower_expr(lhs);
                let r = self.lower_expr(rhs);

                // Route through runtime dispatch when primitive Cranelift ops won't work:
                // - Mixed numeric (int+float)
                // - Python true division (/)
                // - Non-primitive types (list, tuple, str, dict comparisons/ops)
                let lt = self.tcx.get(lhs.ty());
                let rt = self.tcx.get(rhs.ty());
                let is_mixed_numeric = matches!(
                    (lt, rt),
                    (crate::types::Ty::Int, crate::types::Ty::Float)
                        | (crate::types::Ty::Float, crate::types::Ty::Int)
                        | (crate::types::Ty::Bool, crate::types::Ty::Float)
                        | (crate::types::Ty::Float, crate::types::Ty::Bool)
                );
                let is_true_div = matches!(op, HirBinOp::Div)
                    && matches!(lt, crate::types::Ty::Int)
                    && matches!(rt, crate::types::Ty::Int);
                let needs_runtime = !matches!(
                    lt,
                    crate::types::Ty::Int | crate::types::Ty::Float | crate::types::Ty::Bool
                ) || !matches!(
                    rt,
                    crate::types::Ty::Int | crate::types::Ty::Float | crate::types::Ty::Bool
                );
                // Float-Float comparisons must go through runtime because the
                // JIT path stores Float vregs as I64 bit patterns (NaN-boxed
                // carrier), so the MirInst::BinOp primitive path would compare
                // IEEE 754 bits with native i-compares:
                //   - ordering (`<`/`>`/`<=`/`>=`) under signed icmp inverts
                //     negatives (`-2.0 < -1.0` → False);
                //   - equality (`==`/`!=`) under bitwise icmp mis-handles the
                //     two IEEE values whose numeric and bit equality disagree:
                //     `0.0 == -0.0` is True numerically but bits differ (False),
                //     and `nan == nan` is False numerically but identical bits
                //     compare equal (True).
                // Routing through mb_lt/mb_gt/mb_le/mb_ge/mb_eq/mb_ne gives
                // IEEE-aware comparison via `as_float()` in the runtime.
                let is_float_float_cmp =
                    matches!((lt, rt), (crate::types::Ty::Float, crate::types::Ty::Float))
                        && matches!(
                            op,
                            HirBinOp::Lt
                                | HirBinOp::Gt
                                | HirBinOp::LtEq
                                | HirBinOp::GtEq
                                | HirBinOp::Eq
                                | HirBinOp::NotEq
                        );

                if matches!(op, HirBinOp::Is | HirBinOp::IsNot) {
                    let boxed_l = self.box_operand(l, lhs.ty());
                    let boxed_r = self.box_operand(r, rhs.ty());
                    let dest = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(dest),
                        name: if matches!(op, HirBinOp::Is) {
                            "mb_is_identity".to_string()
                        } else {
                            "mb_is_not_identity".to_string()
                        },
                        args: vec![boxed_l, boxed_r],
                        ty: self.tcx.any(),
                    });
                    return dest;
                }

                // `in` / `not in` — always route through runtime with swapped args
                // Python: `value in container` → mb_list_contains(container, value)
                if matches!(op, HirBinOp::In | HirBinOp::NotIn) {
                    let boxed_l = self.box_operand(l, lhs.ty());
                    let boxed_r = self.box_operand(r, rhs.ty());
                    let dest = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(dest),
                        name: "mb_list_contains".to_string(),
                        args: vec![boxed_r, boxed_l], // swapped: container first
                        ty: self.tcx.any(),
                    });
                    if matches!(op, HirBinOp::NotIn) {
                        let negated = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(negated),
                            name: "mb_not".to_string(),
                            args: vec![dest],
                            ty: self.tcx.any(),
                        });
                        return negated;
                    }
                    return dest;
                }

                // Str + Str → mb_str_concat (string concatenation)
                if matches!(op, HirBinOp::Add)
                    && matches!(lt, crate::types::Ty::Str)
                    && matches!(rt, crate::types::Ty::Str)
                {
                    let boxed_l = self.box_operand(l, lhs.ty());
                    let boxed_r = self.box_operand(r, rhs.ty());
                    let dest = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(dest),
                        name: "mb_str_concat".to_string(),
                        args: vec![boxed_l, boxed_r],
                        ty: self.tcx.str(),
                    });
                    return dest;
                }

                // Non-primitive operand (Class, Any, List, Dict, Set, Tuple, ...) →
                // route through mb_dispatch_binop so __op__/__rop__ dunders are
                // consulted before falling back to primitive builtins. The dispatch
                // function falls through to mb_add/mb_sub/etc. for non-class values.
                // Keep the narrow mixed-numeric / true-div path for typed numerics.
                //
                // KNOWN GAP — #2129: stdlib types using the integer-handle pattern
                // (`fractions.Fraction`, `decimal.Decimal`, future Complex / Matrix
                // / money wrappers) return a NaN-boxed handle id whose static type
                // is `Ty::Int`. They take the native i64 add path below instead of
                // this dispatch branch, so `Fraction(1,3) + Fraction(1,6)` adds the
                // raw handle ids rather than dispatching `__add__`. Three escalating
                // fixes are catalogued on the issue (compile-time origin tracking,
                // `MbValue::TypedHandle` tag, or returning `MbObject::Instance` from
                // handle constructors). Phase 3 territory; ignored regressions live
                // in `tests/jit_tests.rs::test_jit_issue_2129_*`.
                //
                // 2026-05-16 update — a fourth, narrower carve-out option has
                // surfaced that does not touch the type system or value
                // representation: in the `(Int, Int)` Add/Sub/Mul lowering
                // below, instead of `CheckedAdd/Sub/Mul` emit a new
                // `mb_int_arith_with_handle_dispatch` runtime call. That
                // function performs the same INT48 fast-path as the JIT does
                // today, but first runs an `is_fraction_handle(id) ||
                // is_decimal_handle(id)` table lookup on either operand and
                // routes through `mb_dispatch_binop` whenever a handle is
                // detected. The runtime cost is two `HashSet<u64>::contains`
                // hits per int op — measurable but bounded; can be gated on
                // a module-level `cfg(feature = "stdlib_handle_dispatch")`
                // flag to keep the int-arith hot path untouched until the
                // perf regression is measured. The handle tables already
                // exist (`fractions_mod::is_fraction_handle`,
                // `decimal_mod::is_decimal_handle`), so wiring is local to
                // `builtins.rs` + the `CheckedAdd/Sub/Mul` codegen. This is
                // the minimum-surface fix consistent with no type-system or
                // ABI changes.
                let has_class = matches!(lt, crate::types::Ty::Class { .. })
                    || matches!(rt, crate::types::Ty::Class { .. });
                // Ty::TypeVar values are runtime-boxed MbValues (a generic
                // callee's params/returns compile to the boxed I64 ABI), so
                // they must take the dynamic-dispatch path like Any — a raw
                // icmp/fcmp would compare boxed bits against a raw primitive.
                let has_any = matches!(lt, crate::types::Ty::Any | crate::types::Ty::TypeVar(_))
                    || matches!(rt, crate::types::Ty::Any | crate::types::Ty::TypeVar(_));
                if (has_class || has_any) && binop_to_runtime(*op).is_some() {
                    let boxed_l = self.box_operand(l, lhs.ty());
                    let boxed_r = self.box_operand(r, rhs.ty());
                    let opcode = self.emit_int_const(lower_mir_binop(*op).to_opcode());
                    let dest = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(dest),
                        name: "mb_dispatch_binop".to_string(),
                        args: vec![opcode, boxed_l, boxed_r],
                        ty: self.tcx.any(),
                    });
                    return dest;
                }

                if is_mixed_numeric || is_true_div || needs_runtime || is_float_float_cmp {
                    if let Some(rt_func) = binop_to_runtime(*op) {
                        let boxed_l = self.box_operand(l, lhs.ty());
                        let boxed_r = self.box_operand(r, rhs.ty());
                        let dest = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: rt_func.to_string(),
                            args: vec![boxed_l, boxed_r],
                            ty: self.tcx.any(),
                        });
                        return dest;
                    }
                }

                // Canonicalize NaN-boxed Int/Bool operands before a primitive
                // comparison (len-eq bug). The JIT lowers `==`/`!=`/`<`/… on
                // Int/Bool operands to a native `icmp` over the raw register bits
                // (`emit_binop`), assuming each operand is a raw i64. But a
                // statically-Int/Bool operand can still carry a NaN-boxed inline
                // value when produced by a builtin/runtime call and stored to a
                // variable — `x = len([1,2,3])` leaves `x` holding boxed bits, not
                // raw `3`, so `icmp eq <boxed>, 3` is False even though the dynamic
                // rich-compare path is correct (visible only once bound to a name;
                // inline `len([1,2,3]) == 3` returns raw and worked). Route each
                // operand through `mb_unbox_{int,bool}_if_boxed` — unbox a tagged
                // inline value to raw i64, identity passthrough for an already-raw
                // operand — so the native `icmp` and the runtime path agree. This
                // lives in MIR lowering, shared by the JIT and AOT backends, so
                // they cannot diverge. Scoped to comparison ops; arithmetic keeps
                // its CheckedAdd/Sub/Mul boxing contract, Float cmp routes to
                // runtime above.
                let is_comparison = matches!(
                    *op,
                    HirBinOp::Eq
                        | HirBinOp::NotEq
                        | HirBinOp::Lt
                        | HirBinOp::Gt
                        | HirBinOp::LtEq
                        | HirBinOp::GtEq
                );
                let unbox_if_boxed =
                    |this: &mut Self, operand: VReg, oty: &crate::types::Ty| -> VReg {
                        let name = match oty {
                            crate::types::Ty::Int => "mb_unbox_int_if_boxed",
                            crate::types::Ty::Bool => "mb_unbox_bool_if_boxed",
                            _ => return operand,
                        };
                        let ty_id = if matches!(oty, crate::types::Ty::Bool) {
                            this.tcx.bool()
                        } else {
                            this.tcx.int()
                        };
                        let unboxed = this.fresh_vreg();
                        this.current_stmts.push(MirInst::CallExtern {
                            dest: Some(unboxed),
                            name: name.to_string(),
                            args: vec![operand],
                            ty: ty_id,
                        });
                        unboxed
                    };
                let (l, r) = if is_comparison {
                    let nl = unbox_if_boxed(self, l, lt);
                    let nr = unbox_if_boxed(self, r, rt);
                    (nl, nr)
                } else {
                    (l, r)
                };

                let dest = self.fresh_vreg();
                // For integer Add/Sub/Mul, emit checked variants that promote to BigInt on overflow (#833)
                let checked_op: Option<fn(VReg, VReg, VReg, TypeId) -> MirInst> =
                    if matches!((lt, rt), (crate::types::Ty::Int, crate::types::Ty::Int)) {
                        match op {
                            HirBinOp::Add => Some(|dest, lhs, rhs, ty| MirInst::CheckedAdd {
                                dest,
                                lhs,
                                rhs,
                                ty,
                            }),
                            HirBinOp::Sub => Some(|dest, lhs, rhs, ty| MirInst::CheckedSub {
                                dest,
                                lhs,
                                rhs,
                                ty,
                            }),
                            HirBinOp::Mul => Some(|dest, lhs, rhs, ty| MirInst::CheckedMul {
                                dest,
                                lhs,
                                rhs,
                                ty,
                            }),
                            _ => None,
                        }
                    } else {
                        None
                    };
                if let Some(make_inst) = checked_op {
                    self.current_stmts.push(make_inst(dest, l, r, *ty));
                } else {
                    self.current_stmts.push(MirInst::BinOp {
                        dest,
                        op: lower_mir_binop(*op),
                        lhs: l,
                        rhs: r,
                        ty: *ty,
                    });
                }
                dest
            }
            HirExpr::UnaryOp { op, operand, ty } => {
                let inner = self.lower_expr(operand);
                if matches!(*op, crate::hir::HirUnaryOp::Not) {
                    let boxed = self.box_operand(inner, operand.ty());
                    let dest = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(dest),
                        name: "mb_not".to_string(),
                        args: vec![boxed],
                        ty: self.tcx.any(),
                    });
                    return dest;
                }
                let dest = self.fresh_vreg();
                // Negation must be BigInt-aware: a raw MirUnaryOp::Neg negates
                // the NaN-boxed BigInt pointer bits (yielding garbage, e.g.
                // `-(2**63)`). The checker already rejects `-x` on non-numeric
                // operands, so here `operand` is always numeric.
                //   - statically Int: lower `-x` as the checked subtraction
                //     `0 - x` (the BigInt-aware path binary `-` already uses).
                //   - Any/Complex/etc. (a boxed numeric that may be a BigInt or
                //     complex): route through the runtime `mb_neg`.
                //   - Float/Bool: a raw MirUnaryOp::Neg is correct and fast.
                // #99
                let neg_int = matches!(*op, crate::hir::HirUnaryOp::Neg)
                    && matches!(self.tcx.get(operand.ty()), crate::types::Ty::Int);
                let neg_boxed = matches!(*op, crate::hir::HirUnaryOp::Neg)
                    && !matches!(
                        self.tcx.get(operand.ty()),
                        crate::types::Ty::Int | crate::types::Ty::Float | crate::types::Ty::Bool
                    );
                if neg_int {
                    let zero = self.fresh_vreg();
                    self.current_stmts.push(MirInst::LoadConst {
                        dest: zero,
                        value: MirConst::Int(0),
                        ty: self.tcx.int(),
                    });
                    self.current_stmts.push(MirInst::CheckedSub {
                        dest,
                        lhs: zero,
                        rhs: inner,
                        ty: *ty,
                    });
                } else if neg_boxed {
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(dest),
                        name: "mb_neg".to_string(),
                        args: vec![inner],
                        ty: *ty,
                    });
                } else {
                    self.current_stmts.push(MirInst::UnaryOp {
                        dest,
                        op: lower_mir_unaryop(*op),
                        operand: inner,
                        ty: *ty,
                    });
                }
                dest
            }
            HirExpr::Call { func, args, ty } => {
                // Method call: x.method(args) → mb_call_method(receiver, name, args_list)
                if let HirExpr::Attr { object, attr, .. } = func.as_ref() {
                    // Fast path: when the receiver type is statically Str and
                    // the method has a known direct runtime entry point with
                    // the same arity, bypass mb_call_method dispatch (which
                    // does extract_str + name match + args-list unwrap) and
                    // call the runtime fn directly. Saves one MakeList plus
                    // one mb_call_method/dispatch_str_method dispatch per
                    // call. Hot on `"".join(parts)` and similar.
                    // Only direct-call methods that have a registered runtime
                    // symbol with matching arity. Adding a new entry here
                    // requires a corresponding rt_sym! in runtime/symbols.rs.
                    let recv_ty_view = self.tcx.get(object.ty());
                    let direct_method_fn: Option<&'static str> = match recv_ty_view {
                        Ty::Str => match (attr.as_str(), args.len()) {
                            ("join", 1) => Some("mb_str_join"),
                            ("upper", 0) => Some("mb_str_upper"),
                            ("lower", 0) => Some("mb_str_lower"),
                            ("capitalize", 0) => Some("mb_str_capitalize"),
                            ("title", 0) => Some("mb_str_title"),
                            ("isdigit", 0) => Some("mb_str_isdigit"),
                            ("isalpha", 0) => Some("mb_str_isalpha"),
                            _ => None,
                        },
                        Ty::List(_) => match (attr.as_str(), args.len()) {
                            ("append", 1) => Some("mb_list_append"),
                            ("clear", 0) => Some("mb_list_clear"),
                            ("reverse", 0) => Some("mb_list_reverse"),
                            ("sort", 0) => Some("mb_list_sort"),
                            ("copy", 0) => Some("mb_list_copy"),
                            ("extend", 1) => Some("mb_list_extend"),
                            ("remove", 1) => Some("mb_list_remove"),
                            ("index", 1) => Some("mb_list_index"),
                            ("count", 1) => Some("mb_list_count"),
                            ("pop", 0) => Some("mb_list_pop"),
                            ("pop", 1) => Some("mb_list_pop_at"),
                            ("insert", 2) => Some("mb_list_insert"),
                            _ => None,
                        },
                        Ty::Dict(..) => match (attr.as_str(), args.len()) {
                            ("clear", 0) => Some("mb_dict_clear"),
                            ("copy", 0) => Some("mb_dict_copy"),
                            ("keys", 0) => Some("mb_dict_keys_view"),
                            ("values", 0) => Some("mb_dict_values_view"),
                            ("items", 0) => Some("mb_dict_items_view"),
                            ("update", 1) => Some("mb_dict_update"),
                            _ => None,
                        },
                        _ => None,
                    };
                    if let Some(fn_name) = direct_method_fn {
                        let recv_raw = self.lower_expr(object);
                        let receiver = self.box_operand(recv_raw, object.ty());
                        let mut call_args = vec![receiver];
                        for a in args.iter() {
                            let v = self.lower_expr(a);
                            call_args.push(self.box_operand(v, a.ty()));
                        }
                        let dest = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: fn_name.to_string(),
                            args: call_args,
                            ty: *ty,
                        });
                        self.emit_exception_propagate();
                        return dest;
                    }

                    let recv_raw = self.lower_expr(object);
                    let recv_ty = object.ty();
                    let receiver = self.box_operand(recv_raw, recv_ty);
                    let method_name = self.emit_str_const(attr);
                    let arg_vregs: Vec<VReg> = args
                        .iter()
                        .map(|a| {
                            let v = self.lower_expr(a);
                            self.box_operand(v, a.ty())
                        })
                        .collect();
                    let args_list = self.fresh_vreg();
                    self.current_stmts.push(MirInst::MakeList {
                        dest: args_list,
                        elements: arg_vregs,
                        ty: self.tcx.any(),
                    });
                    let dest = self.fresh_vreg();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(dest),
                        name: "mb_call_method".to_string(),
                        args: vec![receiver, method_name, args_list],
                        ty: *ty,
                    });
                    // If the method set a pending exception, short-circuit
                    // before its result reaches any outer call — without
                    // this, `print(s.index("missing"))` printed the raw
                    // None that index returned.
                    self.emit_exception_propagate();
                    return dest;
                }
                // Direct extern call: HirExpr::StrLit("mb_*", _) → CallExtern.
                // Used by mb_call_spread (star-call lowering in ast_to_hir).
                if let HirExpr::StrLit(extern_name, _) = func.as_ref() {
                    let arg_vregs: Vec<VReg> = args.iter().map(|a| self.lower_expr(a)).collect();
                    let dest = self.fresh_vreg();
                    let boxed_args: Vec<VReg> = args
                        .iter()
                        .zip(arg_vregs.iter())
                        .map(|(arg_expr, &vreg)| self.box_operand(vreg, arg_expr.ty()))
                        .collect();
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: Some(dest),
                        name: extern_name.clone(),
                        args: boxed_args,
                        ty: *ty,
                    });
                    return dest;
                }
                // Regular function call
                let arg_vregs: Vec<VReg> = args.iter().map(|a| self.lower_expr(a)).collect();
                // If evaluating an argument raised, short-circuit before the
                // outer call runs — otherwise `print(int("bad"))` would print
                // the raw None that int() returned and then also enter the
                // except handler.
                self.emit_exception_propagate();
                let dest = self.fresh_vreg();
                let func_sym = match func.as_ref() {
                    HirExpr::Var(sym, _) => *sym,
                    _ => SymbolId(u32::MAX), // indirect call placeholder
                };
                // Class instantiation: ClassName(args) → mb_instance_new_with_init
                if self.user_class_syms.contains(&func_sym.0) {
                    if let Some(class_name) = self.class_syms.get(&func_sym.0).cloned() {
                        let name_vreg = self.emit_str_const(&class_name);
                        let boxed_args: Vec<VReg> = args
                            .iter()
                            .zip(arg_vregs.iter())
                            .map(|(arg_expr, &vreg)| self.box_operand(vreg, arg_expr.ty()))
                            .collect();
                        let args_list = self.fresh_vreg();
                        self.current_stmts.push(MirInst::MakeList {
                            dest: args_list,
                            elements: boxed_args,
                            ty: self.tcx.any(),
                        });
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_instance_new_with_init".to_string(),
                            args: vec![name_vreg, args_list],
                            ty: *ty,
                        });
                        return dest;
                    }
                }
                // Descriptor type constructors called as values:
                // property(...) / staticmethod(f) / classmethod(f). These names
                // live in class_syms (so `isinstance(x, property)` works) but are
                // not user classes and have no builtin extern, so without this
                // branch they would fall into the exception-class fallback below
                // and build a bogus exception object. Route the CALL form to the
                // real descriptor constructors. (The @decorator form is handled
                // separately at class-registration time.)
                if !self.user_class_syms.contains(&func_sym.0) {
                    if let Some(class_name) = self.class_syms.get(&func_sym.0).cloned() {
                        match class_name.as_str() {
                            "property" => {
                                let boxed_args: Vec<VReg> = args
                                    .iter()
                                    .zip(arg_vregs.iter())
                                    .map(|(a, &v)| self.box_operand(v, a.ty()))
                                    .collect();
                                let args_list = self.fresh_vreg();
                                self.current_stmts.push(MirInst::MakeList {
                                    dest: args_list,
                                    elements: boxed_args,
                                    ty: self.tcx.any(),
                                });
                                self.current_stmts.push(MirInst::CallExtern {
                                    dest: Some(dest),
                                    name: "mb_property_from_args".to_string(),
                                    args: vec![args_list],
                                    ty: *ty,
                                });
                                return dest;
                            }
                            "staticmethod" | "classmethod" => {
                                let arg0 = if args.is_empty() {
                                    self.emit_none()
                                } else {
                                    self.box_operand(arg_vregs[0], args[0].ty())
                                };
                                let extern_name = if class_name == "staticmethod" {
                                    "mb_staticmethod_new"
                                } else {
                                    "mb_classmethod_new"
                                };
                                self.current_stmts.push(MirInst::CallExtern {
                                    dest: Some(dest),
                                    name: extern_name.to_string(),
                                    args: vec![arg0],
                                    ty: *ty,
                                });
                                return dest;
                            }
                            _ => {}
                        }
                    }
                }
                // Built-in exception class call: ExcType(msg) → mb_exception_new.
                // Skip if the symbol also has a builtin extern (type constructors like int, str).
                if !self.user_class_syms.contains(&func_sym.0)
                    && !self.builtin_syms.contains_key(&func_sym.0)
                {
                    if let Some(class_name) = self.class_syms.get(&func_sym.0).cloned() {
                        // ExceptionGroup(...) / BaseExceptionGroup(...) as an
                        // expression: pass ALL positional args so the constructor
                        // validates arity + argument types (CPython raises
                        // TypeError/ValueError on a bad shape).
                        if class_name == "ExceptionGroup"
                            || class_name == "BaseExceptionGroup" {
                            let boxed: Vec<VReg> = args.iter().zip(arg_vregs.iter())
                                .map(|(a, &v)| self.box_operand(v, a.ty()))
                                .collect();
                            let args_list = self.fresh_vreg();
                            self.current_stmts.push(MirInst::MakeList {
                                dest: args_list,
                                elements: boxed,
                                ty: self.tcx.any(),
                            });
                            let cn_vreg = self.emit_str_const(&class_name);
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(dest),
                                name: "mb_exception_group_construct".to_string(),
                                args: vec![args_list, cn_vreg],
                                ty: *ty,
                            });
                            return dest;
                        }
                        // Regular exception: ExcType(args...) → mb_exception_new_with_args(type_str, args_list)
                        // This preserves all constructor arguments in e.args (e.g. TypeError("bad", 42).args == ('bad', 42))
                        let type_vreg = self.emit_str_const(&class_name);
                        let boxed_exc_args: Vec<VReg> = args
                            .iter()
                            .zip(arg_vregs.iter())
                            .map(|(arg_expr, &vreg)| self.box_operand(vreg, arg_expr.ty()))
                            .collect();
                        let exc_args_list = self.fresh_vreg();
                        self.current_stmts.push(MirInst::MakeList {
                            dest: exc_args_list,
                            elements: boxed_exc_args,
                            ty: self.tcx.any(),
                        });
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_exception_new_with_args".to_string(),
                            args: vec![type_vreg, exc_args_list],
                            ty: *ty,
                        });
                        return dest;
                    }
                }
                // Check if this is a builtin call that maps to an extern
                if let Some(extern_name) = self.builtin_syms.get(&func_sym.0).cloned() {
                    // HANDWRITE-BEGIN gap="standardize:projects-mamba-src-lower-hir-to-mir-rs" tracker="standardize-gap-projects-mamba-src-lower-hir-to-mir-rs" reason="introspection-builtins (issue: enhancement-mamba-introspection-builtins-globals-locals-vars-dir)."
                    // locals() and vars() (zero-arg) snapshot the current frame's
                    // local bindings. mamba's JIT keeps locals in VRegs without a
                    // frame-dict, so we synthesize the dict inline at the call
                    // site by walking the lowerer's `sym_to_vreg` map. Outside a
                    // function (top-level / module scope) there is no per-frame
                    // dict — fall through to the runtime helper which returns
                    // module globals (CPython's contract for module-scope
                    // locals()).
                    // dir() with no args → return sorted names in the current module globals
                    if extern_name == "mb_dir" && args.is_empty() {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_dir_no_args".to_string(),
                            args: vec![],
                            ty: *ty,
                        });
                        return dest;
                    }
                    // dir() accepts at most one argument — CPython raises
                    // TypeError for dir(a, b); route to the raising stub.
                    if extern_name == "mb_dir" && args.len() > 1 {
                        let n_raw = self.emit_int_const(args.len() as i64);
                        let n_boxed = self.box_operand(n_raw, self.tcx.int());
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_dir_arity_error".to_string(),
                            args: vec![n_boxed],
                            ty: *ty,
                        });
                        return dest;
                    }
                    if (extern_name == "mb_locals" && args.is_empty())
                        || (extern_name == "mb_vars" && args.is_empty())
                    {
                        if !self.in_module_scope {
                            return self.emit_locals_snapshot_dict(dest, *ty);
                        }
                        // Module scope: locals() == globals(); call the runtime
                        // helper which reads MODULE_SYM_INFO + GLOBAL_ID_NAMESPACE.
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_locals".to_string(),
                            args: vec![],
                            ty: *ty,
                        });
                        return dest;
                    }
                    // HANDWRITE-END
                    // Special case: super() with no args → supply class_name and self
                    if extern_name == "mb_super" && args.is_empty() {
                        if let Some((ref class_name, self_sym)) = self.current_class_ctx.clone() {
                            let cls_vreg = self.emit_str_const(class_name);
                            let self_vreg = self
                                .sym_to_vreg
                                .get(&self_sym)
                                .copied()
                                .unwrap_or_else(|| self.emit_none());
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(dest),
                                name: "mb_super".to_string(),
                                args: vec![cls_vreg, self_vreg],
                                ty: *ty,
                            });
                            return dest;
                        }
                    }
                    // Box primitive arguments for runtime functions
                    let boxed_args: Vec<VReg> = args
                        .iter()
                        .zip(arg_vregs.iter())
                        .map(|(arg_expr, &vreg)| self.box_operand(vreg, arg_expr.ty()))
                        .collect();
                    // Special case: min/max called with multiple scalar args → pack into list.
                    if (extern_name == "mb_min" || extern_name == "mb_max") && boxed_args.len() >= 2
                    {
                        let list_vreg = self.fresh_vreg();
                        self.current_stmts.push(MirInst::MakeList {
                            dest: list_vreg,
                            elements: boxed_args,
                            ty: self.tcx.any(),
                        });
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: extern_name,
                            args: vec![list_vreg],
                            ty: *ty,
                        });
                        return dest;
                    }
                    // format(x) with 1 arg → format(x, "") (None sentinel;
                    // mb_format substitutes the empty spec).
                    if extern_name == "mb_format" && boxed_args.len() == 1 {
                        let none_vreg = self.emit_none();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: extern_name,
                            args: vec![boxed_args[0], none_vreg],
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: sorted(iterable) with 1 arg → supply None as reverse (False).
                    if extern_name == "mb_sorted" && boxed_args.len() == 1 {
                        let none_vreg = self.emit_none();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: extern_name,
                            args: vec![boxed_args[0], none_vreg],
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: sorted(iterable, reverse=True) → pass both args.
                    if extern_name == "mb_sorted" && boxed_args.len() >= 2 {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: extern_name,
                            args: vec![boxed_args[0], boxed_args[1]],
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: range(start, stop) → mb_range_iter(start, stop, step=1).
                    // range(start, stop, step) → mb_range_iter(start, stop, step).
                    if extern_name == "mb_range" && boxed_args.len() == 2 {
                        let one_raw = self.emit_int_const(1);
                        let one_boxed = self.box_operand(one_raw, self.tcx.int());
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_range_iter".to_string(),
                            args: vec![boxed_args[0], boxed_args[1], one_boxed],
                            ty: *ty,
                        });
                        return dest;
                    }
                    if extern_name == "mb_range" && boxed_args.len() == 3 {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_range_iter".to_string(),
                            args: boxed_args,
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: slice(stop) → mb_slice(None, stop, None);
                    // slice(start, stop) → mb_slice(start, stop, None); the
                    // 3-arg form passes through unchanged. Python's 1-arg
                    // form uniquely binds the lone arg to `stop`, not `start`.
                    if extern_name == "mb_slice" {
                        let none_vreg = self.emit_none();
                        // 0-arg form: CPython raises
                        // `TypeError: slice expected at least 1 argument, got 0`.
                        // Route to `mb_slice_no_args` which raises, then
                        // returns None (the dest is unused since control unwinds).
                        if boxed_args.is_empty() {
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(dest),
                                name: "mb_slice_no_args".to_string(),
                                args: vec![],
                                ty: *ty,
                            });
                            return dest;
                        }
                        // >3 args: CPython raises
                        // `TypeError: slice expected at most 3 arguments, got N`.
                        // Without this the extra args were silently dropped at
                        // the 3-arg `mb_slice` boundary (no_raise).
                        if boxed_args.len() > 3 {
                            let msg = self.emit_str_const(&format!(
                                "slice expected at most 3 arguments, got {}",
                                boxed_args.len()
                            ));
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(dest),
                                name: "mb_arg_bind_error".to_string(),
                                args: vec![msg],
                                ty: *ty,
                            });
                            return dest;
                        }
                        let args = match boxed_args.len() {
                            1 => vec![none_vreg, boxed_args[0], none_vreg],
                            2 => vec![boxed_args[0], boxed_args[1], none_vreg],
                            _ => boxed_args.clone(),
                        };
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: extern_name,
                            args,
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: __import__(name, globals, locals, fromlist, level)
                    // — mb_dunder_import takes only `name`; trailing args dropped.
                    if extern_name == "mb_dunder_import" {
                        let name_arg = boxed_args
                            .first()
                            .copied()
                            .unwrap_or_else(|| self.emit_none());
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: extern_name,
                            args: vec![name_arg],
                            ty: *ty,
                        });
                        return dest;
                    }
                    // exec(code, globals) needs the namespace dict at runtime.
                    // The historical mb_exec intrinsic is single-arg, so route
                    // the two-arg form through the dedicated runtime entry.
                    if extern_name == "mb_exec" {
                        let code_arg = boxed_args.first().copied().unwrap_or_else(|| self.emit_none());
                        if boxed_args.len() >= 2 {
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(dest),
                                name: "mb_exec_with_globals".to_string(),
                                args: vec![code_arg, boxed_args[1]],
                                ty: *ty,
                            });
                        } else {
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(dest),
                                name: extern_name,
                                args: vec![code_arg],
                                ty: *ty,
                            });
                        }
                        return dest;
                    }
                    // Special case: breakpoint(*args, **kwargs) — drop all
                    // args. mb_breakpoint takes zero args and returns None.
                    if extern_name == "mb_breakpoint" {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: extern_name,
                            args: vec![],
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: complex() / complex(real) / complex(real, imag).
                    // The runtime always takes (real, imag); fill in defaults
                    // so any arity from the call site lowers cleanly.
                    if extern_name == "mb_complex" {
                        // >2 args: CPython raises
                        // `TypeError: complex() takes at most 2 arguments (N given)`.
                        if boxed_args.len() > 2 {
                            let msg = self.emit_str_const(&format!(
                                "complex() takes at most 2 arguments ({} given)",
                                boxed_args.len()
                            ));
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(dest),
                                name: "mb_arg_bind_error".to_string(),
                                args: vec![msg],
                                ty: *ty,
                            });
                            return dest;
                        }
                        let none_vreg = self.emit_none();
                        let args = match boxed_args.len() {
                            0 => {
                                let zero_raw = self.emit_int_const(0);
                                let zero_boxed = self.box_operand(zero_raw, self.tcx.int());
                                vec![zero_boxed, none_vreg]
                            }
                            1 => vec![boxed_args[0], none_vreg],
                            _ => boxed_args.clone(),
                        };
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: extern_name,
                            args,
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: bytes/bytearray with an encoding argument.
                    // The one-arg runtime entry rejects bare str; the two-arg
                    // entry keeps bytes("x", "utf-8") and bytearray("x", "utf-8")
                    // on the CPython path.
                    if extern_name == "mb_bytes_new_checked" {
                        if boxed_args.is_empty() {
                            let none_vreg = self.emit_none();
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(dest),
                                name: extern_name,
                                args: vec![none_vreg],
                                ty: *ty,
                            });
                            return dest;
                        }
                        if boxed_args.len() >= 2 {
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(dest),
                                name: "mb_bytes_new_encoded".to_string(),
                                args: vec![boxed_args[0], boxed_args[1]],
                                ty: *ty,
                            });
                            return dest;
                        }
                    }
                    if extern_name == "mb_bytearray_new_checked" {
                        if boxed_args.is_empty() {
                            let none_vreg = self.emit_none();
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(dest),
                                name: extern_name,
                                args: vec![none_vreg],
                                ty: *ty,
                            });
                            return dest;
                        }
                        if boxed_args.len() >= 2 {
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(dest),
                                name: "mb_bytearray_new_encoded".to_string(),
                                args: vec![boxed_args[0], boxed_args[1]],
                                ty: *ty,
                            });
                            return dest;
                        }
                    }
                    // Special case: sum(iterable, start) → mb_sum_with_start(iterable, start).
                    if extern_name == "mb_sum" && boxed_args.len() == 2 {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_sum_with_start".to_string(),
                            args: boxed_args,
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: type(name, bases, dict) → mb_type3(name, bases, dict).
                    if extern_name == "mb_type" && boxed_args.is_empty() {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_type_no_args".to_string(),
                            args: vec![],
                            ty: *ty,
                        });
                        return dest;
                    }
                    if extern_name == "mb_type" && boxed_args.len() == 2 {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_type2".to_string(),
                            args: boxed_args,
                            ty: *ty,
                        });
                        return dest;
                    }
                    if extern_name == "mb_type" && boxed_args.len() == 3 {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_type3".to_string(),
                            args: boxed_args,
                            ty: *ty,
                        });
                        return dest;
                    }
                    if extern_name == "mb_type" && boxed_args.len() == 4 {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_type3_kwargs".to_string(),
                            args: boxed_args,
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: pow(base, exp, mod) → mb_pow_mod(base, exp, mod).
                    if extern_name == "mb_pow" && boxed_args.len() == 3 {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_pow_mod".to_string(),
                            args: boxed_args,
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: int(value, base) → mb_int_base(value, base).
                    if extern_name == "mb_int" && boxed_args.len() == 2 {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_int_base".to_string(),
                            args: boxed_args,
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: map with 3+ args (func + 2+ iterables) → pack
                    // iterables into a list and call mb_map_n(func, iterables).
                    if extern_name == "mb_map" && boxed_args.len() >= 3 {
                        let func_vreg = boxed_args[0];
                        let list_vreg = self.fresh_vreg();
                        self.current_stmts.push(MirInst::MakeList {
                            dest: list_vreg,
                            elements: boxed_args[1..].to_vec(),
                            ty: self.tcx.any(),
                        });
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_map_n".to_string(),
                            args: vec![func_vreg, list_vreg],
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: zip with any arg count other than 2 → pack
                    // into a list and call mb_zip_n. mb_zip takes exactly two
                    // iterables, so zip() (empty → []) and zip(x) (single →
                    // 1-tuples) would otherwise read garbage operands and raise
                    // a spurious "'float' object is not iterable".
                    if extern_name == "mb_zip" && boxed_args.len() != 2 {
                        let list_vreg = self.fresh_vreg();
                        self.current_stmts.push(MirInst::MakeList {
                            dest: list_vreg,
                            elements: boxed_args,
                            ty: self.tcx.any(),
                        });
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_zip_n".to_string(),
                            args: vec![list_vreg],
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: enumerate(iterable) with 1 arg → supply start=0.
                    if extern_name == "mb_enumerate" && boxed_args.len() == 1 {
                        let zero_raw = self.emit_int_const(0);
                        let zero_boxed = self.box_operand(zero_raw, self.tcx.int());
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: extern_name,
                            args: vec![boxed_args[0], zero_boxed],
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: round(x) with 1 arg → supply None as ndigits (default 0).
                    // mb_round(val, ndigits) uses ndigits.as_int().unwrap_or(0).
                    if extern_name == "mb_round" && boxed_args.len() == 1 {
                        let none_vreg = self.emit_none();
                        let args = vec![boxed_args[0], none_vreg];
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: extern_name,
                            args,
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: open(path) with 1 arg → supply None as mode (defaults to "r" in mb_open).
                    if extern_name == "mb_open" && boxed_args.len() == 1 {
                        let none_vreg = self.emit_none();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: extern_name,
                            args: vec![boxed_args[0], none_vreg],
                            ty: *ty,
                        });
                        return dest;
                    }
                    // R3 P1: getattr(obj, name, default) → mb_getattr_default(obj, name, default)
                    // 2-arg getattr maps to mb_getattr normally; 3-arg needs the _default variant.
                    if extern_name == "mb_getattr" && boxed_args.len() == 3 {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_getattr_default".to_string(),
                            args: boxed_args,
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: next(it, default) → call mb_next_default
                    if extern_name == "mb_next_raise" && boxed_args.len() == 2 {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_next_default".to_string(),
                            args: boxed_args,
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: next(it) — explicit 1-arg next() raises
                    // StopIteration on exhaustion. mb_next_raise SETS the
                    // StopIteration exception but also returns a dummy None; without
                    // an immediate propagate, `v = next(it)` inside a loop body lets
                    // that None leak into the following statements before the pending
                    // exception is observed at the next checkpoint (e.g. a manual
                    // `while True: out.append(next(it))` would append a trailing
                    // None). Emit the call then propagate so the dummy value is
                    // never consumed when exhausted, matching CPython.
                    if extern_name == "mb_next_raise" && boxed_args.len() == 1 {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_next_raise".to_string(),
                            args: boxed_args,
                            ty: *ty,
                        });
                        self.emit_exception_propagate();
                        return dest;
                    }
                    // Special case: iter(callable, sentinel) → mb_iter_sentinel.
                    // When the callable is a user function with a primitive return type
                    // (int/bool/float), the JIT compiles it to return a raw i64/f64, not a
                    // NaN-boxed MbValue. mb_call0 receives the raw bits which are then
                    // misinterpreted as a subnormal float. Fix: generate a boxing thunk that
                    // wraps the original callable and boxes its return value.
                    if extern_name == "mb_iter" && boxed_args.len() == 2 {
                        // Determine if callable is a user function with primitive return type.
                        let callable_sym = match &args[0] {
                            HirExpr::Var(sym, _) if self.user_funcs.contains(&sym.0) => Some(*sym),
                            _ => None,
                        };
                        let box_fn = callable_sym.and_then(|sym| {
                            self.user_func_return_tys
                                .get(&sym.0)
                                .and_then(|&ret_ty_id| match self.tcx.get(ret_ty_id) {
                                    crate::types::Ty::Int => Some("mb_box_int"),
                                    crate::types::Ty::Bool => Some("mb_box_bool"),
                                    crate::types::Ty::Float => Some("mb_box_float"),
                                    _ => None,
                                })
                        });
                        let callable_vreg =
                            if let (Some(sym), Some(box_fn_name)) = (callable_sym, box_fn) {
                                // Generate a boxing thunk: fn() -> MbValue { mb_box_*(sym()) }
                                // The thunk is a synthetic MirBody with a unique lambda SymbolId.
                                let thunk_id = 4_000_000 + self.next_lambda_id;
                                self.next_lambda_id += 1;
                                let thunk_sym = SymbolId(thunk_id);
                                let raw_ty = *self.user_func_return_tys.get(&sym.0).unwrap();
                                let any_ty = self.tcx.any();
                                let thunk_body = MirBody {
                                    name: thunk_sym,
                                    params: vec![],
                                    return_ty: any_ty,
                                    blocks: vec![BasicBlock {
                                        id: BlockId(0),
                                        stmts: vec![
                                            MirInst::Call {
                                                dest: Some(VReg(0)),
                                                func: sym,
                                                args: vec![],
                                                ty: raw_ty,
                                            },
                                            MirInst::CallExtern {
                                                dest: Some(VReg(1)),
                                                name: box_fn_name.to_string(),
                                                args: vec![VReg(0)],
                                                ty: any_ty,
                                            },
                                        ],
                                        terminator: Terminator::Return(Some(VReg(1))),
                                    }],
                                };
                                self.bodies.push(thunk_body);
                                // Emit LoadConst FuncRef for the thunk so mb_iter_sentinel
                                // calls the boxing wrapper instead of the raw function.
                                let thunk_vreg = self.fresh_vreg();
                                self.current_stmts.push(MirInst::LoadConst {
                                    dest: thunk_vreg,
                                    value: MirConst::FuncRef(thunk_sym),
                                    ty: any_ty,
                                });
                                thunk_vreg
                            } else {
                                boxed_args[0]
                            };
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_iter_sentinel".to_string(),
                            args: vec![callable_vreg, boxed_args[1]],
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Zero-arg arity guard: list()/tuple()/set()/dict() with 0 args →
                    // redirect to the _new variant (mb_list_new, mb_tuple_new, mb_set_new,
                    // mb_dict_new) instead of the _from_iterable/_from_pairs variant which
                    // expects 1 parameter and would cause a Cranelift verifier error.
                    if boxed_args.is_empty() {
                        let new_variant = match extern_name.as_str() {
                            "mb_list_from_iterable" => Some("mb_list_new"),
                            "mb_tuple_from_iterable" => Some("mb_tuple_new"),
                            "mb_set_from_iterable" => Some("mb_set_new"),
                            "mb_dict_from_pairs" => Some("mb_dict_new"),
                            "mb_frozenset_new" => Some("mb_frozenset_empty"),
                            _ => None,
                        };
                        if let Some(new_name) = new_variant {
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(dest),
                                name: new_name.to_string(),
                                args: vec![],
                                ty: *ty,
                            });
                            return dest;
                        }
                        // #1691: scalar constructors (bool/int/float/str) with 0 args.
                        // mb_bool/mb_int/mb_float/mb_str all take 1 i64 (a NaN-boxed
                        // MbValue), so a 0-arg call would emit a 0-arg call site
                        // against a 1-arg sig and trip the Cranelift verifier.
                        // CPython's contract is: bool() == False, int() == 0,
                        // float() == 0.0, str() == "". Synthesize the default arg
                        // (matching the runtime's idempotent self-coercion) so the
                        // extern call shape stays consistent with the 1-arg form.
                        let default_arg: Option<(VReg, TypeId)> = match extern_name.as_str() {
                            "mb_bool" => {
                                let raw = self.fresh_vreg();
                                self.current_stmts.push(MirInst::LoadConst {
                                    dest: raw,
                                    value: MirConst::Bool(false),
                                    ty: self.tcx.bool(),
                                });
                                Some((raw, self.tcx.bool()))
                            }
                            "mb_int" => {
                                let raw = self.emit_int_const(0);
                                Some((raw, self.tcx.int()))
                            }
                            "mb_float" => {
                                let raw = self.fresh_vreg();
                                self.current_stmts.push(MirInst::LoadConst {
                                    dest: raw,
                                    value: MirConst::Float(0.0),
                                    ty: self.tcx.float(),
                                });
                                Some((raw, self.tcx.float()))
                            }
                            "mb_str" => {
                                let raw = self.emit_str_const("");
                                Some((raw, self.tcx.str()))
                            }
                            _ => None,
                        };
                        if let Some((raw_vreg, raw_ty)) = default_arg {
                            let boxed = self.box_operand(raw_vreg, raw_ty);
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(dest),
                                name: extern_name,
                                args: vec![boxed],
                                ty: *ty,
                            });
                            return dest;
                        }
                    }
                    // Special case: print() with zero args → pass empty list to mb_print_args
                    // which prints just a newline (matching Python's print() behavior).
                    if extern_name == "mb_print" && boxed_args.is_empty() {
                        let list_vreg = self.fresh_vreg();
                        self.current_stmts.push(MirInst::MakeList {
                            dest: list_vreg,
                            elements: vec![],
                            ty: self.tcx.any(),
                        });
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_print_args".to_string(),
                            args: vec![list_vreg],
                            ty: *ty,
                        });
                        return dest;
                    }
                    // Special case: print with multiple args → pack into list, call mb_print_args
                    if extern_name == "mb_print" && boxed_args.len() > 1 {
                        let list_vreg = self.fresh_vreg();
                        self.current_stmts.push(MirInst::MakeList {
                            dest: list_vreg,
                            elements: boxed_args,
                            ty: self.tcx.any(),
                        });
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: "mb_print_args".to_string(),
                            args: vec![list_vreg],
                            ty: *ty,
                        });
                    } else {
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(dest),
                            name: extern_name,
                            args: boxed_args,
                            ty: *ty,
                        });
                    }
                } else if self.decorated_func_syms.contains(&func_sym.0) {
                    // Decorated function: load from global (may be replaced by decorator)
                    // then dispatch dynamically based on arg count.
                    let func_val = self.fresh_vreg();
                    self.current_stmts.push(MirInst::LoadGlobal {
                        dest: func_val,
                        name: func_sym,
                        ty: self.tcx.any(),
                    });
                    let boxed_args: Vec<VReg> = args
                        .iter()
                        .zip(arg_vregs.iter())
                        .map(|(arg_expr, &vreg)| self.box_operand(vreg, arg_expr.ty()))
                        .collect();
                    // raw_dest receives the return value before boxing.
                    let raw_dest = self.fresh_vreg();
                    match boxed_args.len() {
                        0 => {
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(raw_dest),
                                name: "mb_call0".to_string(),
                                args: vec![func_val],
                                ty: *ty,
                            });
                        }
                        1 => {
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(raw_dest),
                                name: "mb_call1_val".to_string(),
                                args: vec![func_val, boxed_args[0]],
                                ty: *ty,
                            });
                        }
                        _ => {
                            // N args: pack into a list and dispatch dynamically
                            // through the decorated (wrapper) function loaded from global.
                            let list_vreg = self.fresh_vreg();
                            self.current_stmts.push(MirInst::MakeList {
                                dest: list_vreg,
                                elements: boxed_args,
                                ty: self.tcx.any(),
                            });
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(raw_dest),
                                name: "mb_call_spread".to_string(),
                                args: vec![func_val, list_vreg],
                                ty: *ty,
                            });
                        }
                    }
                    // Box the return value if the function returns a raw primitive.
                    // Dynamic calls (mb_call0/mb_call1_val) return the raw function return value.
                    // When the function's declared return type is a primitive (int/bool/float),
                    // the raw value must be NaN-boxed before passing to runtime callers.
                    let func_ret_ty = self
                        .decorated_func_return_tys
                        .get(&func_sym.0)
                        .copied()
                        .unwrap_or(*ty);
                    let boxed = self.box_operand(raw_dest, func_ret_ty);
                    self.current_stmts.push(MirInst::Copy {
                        dest,
                        source: boxed,
                    });
                } else if func_sym.0 == u32::MAX || !self.user_funcs.contains(&func_sym.0) {
                    // Dynamic dispatch: the callee is a local variable (or a non-Var expression)
                    // holding a TAG_FUNC NaN-boxed function pointer, e.g. `f = outer(42); f()`.
                    // We cannot use MirInst::Call (requires a statically-known function ID) so
                    // we lower the callee expression to a vreg, box the arguments, and dispatch
                    // through mb_call0 / mb_call1_val / mb_call_spread.
                    let func_val = self.lower_expr(func);
                    let boxed_args: Vec<VReg> = args
                        .iter()
                        .zip(arg_vregs.iter())
                        .map(|(arg_expr, &vreg)| self.box_operand(vreg, arg_expr.ty()))
                        .collect();
                    match boxed_args.len() {
                        0 => {
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(dest),
                                name: "mb_call0".to_string(),
                                args: vec![func_val],
                                ty: *ty,
                            });
                        }
                        1 => {
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(dest),
                                name: "mb_call1_val".to_string(),
                                args: vec![func_val, boxed_args[0]],
                                ty: *ty,
                            });
                        }
                        _ => {
                            // N args: pack into a list and use mb_call_spread.
                            let list_vreg = self.fresh_vreg();
                            self.current_stmts.push(MirInst::MakeList {
                                dest: list_vreg,
                                elements: boxed_args,
                                ty: self.tcx.any(),
                            });
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: Some(dest),
                                name: "mb_call_spread".to_string(),
                                args: vec![func_val, list_vreg],
                                ty: *ty,
                            });
                        }
                    }
                } else {
                    // Selectively box primitive arguments destined for Any/object-typed
                    // parameters. Int/Bool/Float params use the raw calling convention so
                    // arithmetic in the callee works on native values. Any/object params
                    // need NaN-boxed MbValues so match-subject comparisons (mb_eq) and
                    // format-string dispatch work correctly (#827 R8).
                    // Clone callee param types eagerly to avoid a borrow conflict between
                    // the immutable borrow of user_func_param_types and the mutable borrow
                    // of self inside box_operand (which appends to current_stmts).
                    let callee_param_types: Vec<TypeId> = self
                        .user_func_param_types
                        .get(&func_sym.0)
                        .cloned()
                        .unwrap_or_default();
                    // Determine which args need boxing before processing (collect types).
                    let arg_info: Vec<(VReg, TypeId, bool)> = args
                        .iter()
                        .zip(arg_vregs.iter())
                        .enumerate()
                        .map(|(i, (arg_expr, &vreg))| {
                            let arg_ty = arg_expr.ty();
                            let arg_is_primitive = matches!(
                                self.tcx.get(arg_ty),
                                crate::types::Ty::Int
                                    | crate::types::Ty::Bool
                                    | crate::types::Ty::Float
                            );
                            let callee_param_is_primitive = callee_param_types
                                .get(i)
                                .map(|&p| {
                                    matches!(
                                        self.tcx.get(p),
                                        crate::types::Ty::Int
                                            | crate::types::Ty::Bool
                                            | crate::types::Ty::Float
                                    )
                                })
                                .unwrap_or(true); // unknown → keep raw (safe default)
                            let needs_box = arg_is_primitive && !callee_param_is_primitive;
                            (vreg, arg_ty, needs_box)
                        })
                        .collect();
                    let final_args: Vec<VReg> = arg_info
                        .into_iter()
                        .map(|(vreg, arg_ty, needs_box)| {
                            if needs_box {
                                self.box_operand(vreg, arg_ty)
                            } else {
                                vreg
                            }
                        })
                        .collect();
                    // For variadic (*args/**kwargs) calls: pack excess positional args into
                    // a MbList for *args, and pass the **kwargs dict as-is.
                    // The callee's Cranelift signature has (regular_params..., [*args_list], [**kwargs_dict]).
                    let (has_star, has_dstar) = self
                        .user_func_variadic_info
                        .get(&func_sym.0)
                        .copied()
                        .unwrap_or((false, false));
                    let n_regular = {
                        let ft = self.tcx.get(func.ty());
                        if let crate::types::Ty::Fn {
                            params: fp,
                            variadic: true,
                            ..
                        } = ft
                        {
                            fp.len()
                        } else {
                            final_args.len()
                        }
                    };
                    let is_variadic_call = has_star || has_dstar;
                    let final_args = if is_variadic_call {
                        let n_actual = final_args.len().min(n_regular);
                        let mut packed: Vec<VReg> = final_args[..n_actual].to_vec();
                        if has_star {
                            // Determine how many excess args go into *args.
                            // If has_dstar, the last HirExpr arg is the kwargs dict (already lowered).
                            let excess_end = if has_dstar && args.len() > n_regular {
                                args.len() - 1 // exclude last (kwargs dict)
                            } else {
                                args.len()
                            };
                            let variadic_elems: Vec<VReg> = if excess_end > n_regular {
                                args[n_regular..excess_end]
                                    .iter()
                                    .zip(arg_vregs[n_regular..excess_end].iter())
                                    .map(|(arg_expr, &vreg)| self.box_operand(vreg, arg_expr.ty()))
                                    .collect()
                            } else {
                                vec![]
                            };
                            let list_vreg = self.fresh_vreg();
                            let any_ty = self.tcx.any();
                            self.current_stmts.push(MirInst::MakeList {
                                dest: list_vreg,
                                elements: variadic_elems,
                                ty: any_ty,
                            });
                            packed.push(list_vreg);
                        }
                        if has_dstar {
                            // The kwargs dict was already constructed by AST lowering as the
                            // last argument. If present in the args list, pass it through;
                            // otherwise create an empty dict.
                            let last_idx = args.len().saturating_sub(1);
                            if args.len() > n_regular + if has_star { 0 } else { 0 } {
                                let kw_vreg = arg_vregs[last_idx];
                                let kw_boxed = self.box_operand(kw_vreg, args[last_idx].ty());
                                packed.push(kw_boxed);
                            } else {
                                // No kwargs dict in args — create empty dict.
                                let empty_dict = self.fresh_vreg();
                                let any_ty = self.tcx.any();
                                self.current_stmts.push(MirInst::MakeDict {
                                    dest: empty_dict,
                                    keys: vec![],
                                    values: vec![],
                                    ty: any_ty,
                                });
                                packed.push(empty_dict);
                            }
                        }
                        packed
                    } else {
                        final_args
                    };
                    self.current_stmts.push(MirInst::Call {
                        dest: Some(dest),
                        func: func_sym,
                        args: final_args,
                        ty: *ty,
                    });
                }
                dest
            }
            HirExpr::Attr { object, attr, ty } => {
                let obj_raw = self.lower_expr(object);
                // Box a primitive receiver (int/float/bool) so the runtime
                // getattr sees a proper NaN-boxed MbValue. Without this, a raw
                // i64 receiver (e.g. `(5).__class__`) is misread as a float.
                let obj = self.box_operand(obj_raw, object.ty());
                let dest = self.fresh_vreg();
                let attr_vreg = self.emit_str_const(attr);
                self.current_stmts.push(MirInst::GetAttr {
                    dest,
                    object: obj,
                    attr: attr.clone(),
                    ty: *ty,
                });
                let _ = attr_vreg; // attr is stored in GetAttr directly
                dest
            }
            HirExpr::Index { object, index, ty } => {
                let obj = self.lower_expr(object);
                let idx_ty = index.ty();
                let idx = self.lower_expr(index);
                // Box primitive index for runtime dispatch (mb_obj_getitem expects MbValue)
                let boxed_idx = self.box_operand(idx, idx_ty);
                let dest = self.fresh_vreg();
                self.current_stmts.push(MirInst::GetItem {
                    dest,
                    object: obj,
                    index: boxed_idx,
                    ty: *ty,
                });
                dest
            }
            HirExpr::List { elements, ty } => {
                let elem_vregs: Vec<VReg> = elements
                    .iter()
                    .map(|e| {
                        let v = self.lower_expr(e);
                        self.box_operand(v, e.ty())
                    })
                    .collect();
                let dest = self.fresh_vreg();
                self.current_stmts.push(MirInst::MakeList {
                    dest,
                    elements: elem_vregs,
                    ty: *ty,
                });
                dest
            }
            HirExpr::Set { elements, ty } => {
                // Create a list of elements, then convert to set via runtime
                let elem_vregs: Vec<VReg> = elements
                    .iter()
                    .map(|e| {
                        let v = self.lower_expr(e);
                        self.box_operand(v, e.ty())
                    })
                    .collect();
                let list_vreg = self.fresh_vreg();
                self.current_stmts.push(MirInst::MakeList {
                    dest: list_vreg,
                    elements: elem_vregs,
                    ty: *ty,
                });
                let dest = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(dest),
                    name: "mb_set_from_list".to_string(),
                    args: vec![list_vreg],
                    ty: *ty,
                });
                dest
            }
            HirExpr::Tuple { elements, ty } => {
                let elem_vregs: Vec<VReg> = elements
                    .iter()
                    .map(|e| {
                        let v = self.lower_expr(e);
                        self.box_operand(v, e.ty())
                    })
                    .collect();
                let dest = self.fresh_vreg();
                self.current_stmts.push(MirInst::MakeTuple {
                    dest,
                    elements: elem_vregs,
                    ty: *ty,
                });
                dest
            }
            HirExpr::Dict { entries, ty } => {
                // Dict unpack `{**v}` is encoded by ast_to_hir as a NoneLit
                // sentinel key tagged with the `Never` type. A user-written
                // `None: ...` entry carries a NoneLit with `NoneType` and is
                // a real key — see ast_to_hir.rs DictLit lowering.
                let never_ty = self.tcx.never();
                let is_unpack = |k: &HirExpr| matches!(k, HirExpr::NoneLit(t) if *t == never_ty);
                let has_unpack = entries.iter().any(|(k, _)| is_unpack(k));
                if has_unpack {
                    // Dict with unpack: create empty dict, then setitem / update for each entry.
                    let dest = self.fresh_vreg();
                    let any_ty = self.tcx.any();
                    self.current_stmts.push(MirInst::MakeDict {
                        dest,
                        keys: vec![],
                        values: vec![],
                        ty: any_ty,
                    });
                    for (k, v) in entries {
                        let vv_raw = self.lower_expr(v);
                        let vv = self.box_operand(vv_raw, v.ty());
                        if is_unpack(k) {
                            // Unpack: mb_dict_update(dest, value_dict)
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: None,
                                name: "mb_dict_update".to_string(),
                                args: vec![dest, vv],
                                ty: any_ty,
                            });
                        } else {
                            // Regular entry: mb_dict_setitem(dest, key, value)
                            let kk_raw = self.lower_expr(k);
                            let kk = self.box_operand(kk_raw, k.ty());
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: None,
                                name: "mb_dict_setitem".to_string(),
                                args: vec![dest, kk, vv],
                                ty: any_ty,
                            });
                        }
                    }
                    dest
                } else {
                    let keys: Vec<VReg> = entries
                        .iter()
                        .map(|(k, _)| {
                            let v = self.lower_expr(k);
                            self.box_operand(v, k.ty())
                        })
                        .collect();
                    let values: Vec<VReg> = entries
                        .iter()
                        .map(|(_, val)| {
                            let v = self.lower_expr(val);
                            self.box_operand(v, val.ty())
                        })
                        .collect();
                    let dest = self.fresh_vreg();
                    self.current_stmts.push(MirInst::MakeDict {
                        dest,
                        keys,
                        values,
                        ty: *ty,
                    });
                    dest
                }
            }
            HirExpr::Slice {
                start,
                stop,
                step,
                ty,
            } => {
                let s = start
                    .as_ref()
                    .map(|e| {
                        let v = self.lower_expr(e);
                        self.box_operand(v, e.ty())
                    })
                    .unwrap_or_else(|| self.emit_none());
                let e = stop
                    .as_ref()
                    .map(|e| {
                        let v = self.lower_expr(e);
                        self.box_operand(v, e.ty())
                    })
                    .unwrap_or_else(|| self.emit_none());
                let st = step
                    .as_ref()
                    .map(|e| {
                        let v = self.lower_expr(e);
                        self.box_operand(v, e.ty())
                    })
                    .unwrap_or_else(|| self.emit_none());
                let dest = self.fresh_vreg();
                // Emit a real `slice` object rather than a (start, stop, step)
                // tuple: a tuple is indistinguishable from `obj[1, 2, 3]` (a
                // genuine tuple key), so a user `__getitem__` could not tell a
                // slice subscript apart from a tuple subscript. mb_obj_getitem
                // normalizes the slice object back to the tuple form for
                // built-in containers and native-stub sequences (struct-seq,
                // etc.), so their fast slice handling is unchanged; only user
                // (non-native) dunders now receive a real slice.
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(dest),
                    name: "mb_slice".to_string(),
                    args: vec![s, e, st],
                    ty: *ty,
                });
                dest
            }
            HirExpr::IfExpr {
                cond,
                then_val,
                else_val,
                ty,
            } => {
                // Use lower_cond_as_bool so heap-backed / Any-typed conditions
                // route through mb_is_truthy. Without this, `x if obj else y`
                // where obj is an Instance always picked the else branch —
                // the raw NaN-boxed pointer's low bits looked like 0.
                let cond_vreg = self.lower_cond_as_bool(cond);
                let then_block = self.fresh_block();
                let else_block = self.fresh_block();
                let merge_block = self.fresh_block();
                let result = self.fresh_vreg();
                self.finish_block(Terminator::Branch {
                    cond: cond_vreg,
                    then_block,
                    else_block,
                });
                // Box both branches to NaN-boxed format so the result vreg
                // always holds a uniform representation regardless of branch types.
                self.start_block(then_block);
                let tv = self.lower_expr(then_val);
                let tv_boxed = self.box_operand(tv, then_val.ty());
                self.current_stmts.push(MirInst::Copy {
                    dest: result,
                    source: tv_boxed,
                });
                self.finish_block(Terminator::Goto(merge_block));
                self.start_block(else_block);
                let ev = self.lower_expr(else_val);
                let ev_boxed = self.box_operand(ev, else_val.ty());
                self.current_stmts.push(MirInst::Copy {
                    dest: result,
                    source: ev_boxed,
                });
                self.finish_block(Terminator::Goto(merge_block));
                self.start_block(merge_block);
                let _ = ty;
                result
            }
            HirExpr::Lambda {
                params,
                param_kinds,
                defaults,
                body,
                ty,
                span,
            } => {
                // Compile the lambda body as a separate MirBody (function), then create a
                // closure wrapping its entry point address so mb_map/mb_filter can call it.

                // Allocate a synthetic SymbolId for the lambda body function.
                self.next_lambda_id += 1;
                let lambda_sym = SymbolId(4_000_000 + self.next_lambda_id);
                let any_ty = self.tcx.any();

                // ── Evaluate default-arg expressions in the OUTER scope ──
                // Python semantics: default args are evaluated at function
                // creation time, not call time. We must lower them before
                // swapping out the outer state so they reference outer vregs.
                // Defaults bind to the trailing N params (Python rule), so we
                // emit only the Some(...) entries. The dispatcher pairs them
                // with `arity` (total param count) to fill missing trailing
                // args at call time.
                let any_default = defaults.iter().any(|d| d.is_some());
                let default_vregs: Vec<VReg> = if any_default {
                    defaults
                        .iter()
                        .filter_map(|d| d.as_ref())
                        .map(|expr| {
                            let raw = self.lower_expr(expr);
                            self.box_operand(raw, expr.ty())
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                let lambda_arity = params.len();

                // ── Store outer variables to global storage for lambda capture ──
                // Lambda bodies access outer variables via LoadGlobal (cell_override).
                // Box and store all live local variables so the lambda can read them.
                let param_syms: std::collections::HashSet<u32> =
                    params.iter().map(|(s, _)| s.0).collect();
                let outer_syms: Vec<(SymbolId, VReg)> = self
                    .sym_to_vreg
                    .iter()
                    .filter(|(sym, _)| !param_syms.contains(&sym.0))
                    .map(|(sym, vreg)| (*sym, *vreg))
                    .collect();
                for &(sym, vreg) in &outer_syms {
                    let boxed = self.box_operand(vreg, any_ty);
                    self.current_stmts.push(MirInst::StoreGlobal {
                        name: sym,
                        value: boxed,
                    });
                }

                // ── Save outer function compilation state ──
                let saved_next_vreg = self.next_vreg;
                let saved_next_block = self.next_block;
                let saved_blocks = std::mem::take(&mut self.blocks);
                let saved_stmts = std::mem::take(&mut self.current_stmts);
                let saved_sym_to_vreg = std::mem::take(&mut self.sym_to_vreg);
                let saved_loop_exit = self.loop_exit;
                let saved_loop_header = self.loop_header;
                let saved_block_id = self.current_block_id;
                let saved_async_coro = self.async_coro_vreg;
                let saved_is_gen = self.is_gen_body;
                let saved_try_stack = std::mem::take(&mut self.try_handler_stack);
                let saved_finally_stack = std::mem::take(&mut self.finally_body_stack);
                // The enclosing `with` exit blocks belong to the OUTER body's
                // block-id space; leaving them on the stack would make the
                // lambda body's exception edges Goto blocks that don't exist
                // in its own MirBody (codegen "no entry found for key").
                let saved_with_exit = std::mem::take(&mut self.with_exit_stack);
                let saved_with_ctx = std::mem::take(&mut self.with_ctx_stack);
                let saved_return_ty = self.current_return_ty;
                let saved_cell_override = std::mem::take(&mut self.cell_override);

                // ── Compile lambda body ──
                self.next_vreg = 0;
                self.next_block = 0;
                self.loop_exit = None;
                self.loop_header = None;
                self.current_block_id = None;
                self.async_coro_vreg = None;
                self.is_gen_body = false;
                self.current_return_ty = any_ty;

                // Mark outer variables for cell_override so lambda body reads
                // them via LoadGlobal instead of looking them up in sym_to_vreg.
                self.cell_override = outer_syms.iter().map(|(sym, _)| sym.0).collect();

                let entry = self.fresh_block();
                self.current_block_id = Some(entry);

                // Map lambda params to fresh vregs.
                let lambda_params: Vec<(VReg, TypeId)> = params
                    .iter()
                    .map(|(sym, par_ty)| {
                        let vreg = self.fresh_vreg();
                        self.sym_to_vreg.insert(*sym, vreg);
                        (vreg, *par_ty)
                    })
                    .collect();

                // Lower the body expression and box the result.
                let body_vreg = self.lower_expr(body);
                let boxed_result = self.box_operand(body_vreg, body.ty());
                if self.current_block_id.is_some() {
                    self.finish_block(Terminator::Return(Some(boxed_result)));
                }

                let lambda_body = MirBody {
                    name: lambda_sym,
                    params: lambda_params,
                    return_ty: any_ty,
                    blocks: std::mem::take(&mut self.blocks),
                };
                self.bodies.push(lambda_body);

                // ── Restore outer function state ──
                self.next_vreg = saved_next_vreg;
                self.next_block = saved_next_block;
                self.blocks = saved_blocks;
                self.current_stmts = saved_stmts;
                self.sym_to_vreg = saved_sym_to_vreg;
                self.loop_exit = saved_loop_exit;
                self.loop_header = saved_loop_header;
                self.current_block_id = saved_block_id;
                self.async_coro_vreg = saved_async_coro;
                self.is_gen_body = saved_is_gen;
                self.try_handler_stack = saved_try_stack;
                self.finally_body_stack = saved_finally_stack;
                self.with_exit_stack = saved_with_exit;
                self.with_ctx_stack = saved_with_ctx;
                self.current_return_ty = saved_return_ty;
                self.cell_override = saved_cell_override;

                // ── Create closure wrapping the lambda's entry point ──
                let name_vreg = self.emit_str_const("<lambda>");
                let fn_addr_vreg = self.fresh_vreg();
                self.current_stmts.push(MirInst::LoadConst {
                    dest: fn_addr_vreg,
                    value: MirConst::FuncRef(lambda_sym),
                    ty: any_ty,
                });
                let none_captures = self.emit_none();
                let closure_vreg = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(closure_vreg),
                    name: "mb_closure_new".to_string(),
                    args: vec![name_vreg, fn_addr_vreg, none_captures],
                    ty: *ty,
                });
                // Freeze the evaluated default values onto the closure so
                // `mb_call0` can dispatch with them when the caller supplies
                // no explicit args.
                let default_vregs_meta = default_vregs.clone();
                if !default_vregs.is_empty() {
                    let defaults_list = self.fresh_vreg();
                    self.current_stmts.push(MirInst::MakeList {
                        dest: defaults_list,
                        elements: default_vregs,
                        ty: self.tcx.any(),
                    });
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: None,
                        name: "mb_closure_set_defaults".to_string(),
                        args: vec![closure_vreg, defaults_list],
                        ty: self.tcx.none(),
                    });
                    let arity_vreg = self.fresh_vreg();
                    self.current_stmts.push(MirInst::LoadConst {
                        dest: arity_vreg,
                        value: MirConst::Int(lambda_arity as i64),
                        ty: self.tcx.int(),
                    });
                    let arity_boxed = self.box_operand(arity_vreg, self.tcx.int());
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: None,
                        name: "mb_closure_set_arity".to_string(),
                        args: vec![closure_vreg, arity_boxed],
                        ty: self.tcx.none(),
                    });
                }

                // ── Function-metadata registries (issue #20) ──
                // Lambdas never flow through the module-init def priming loop,
                // so register name/argcount/varnames/params/srcinfo here at
                // closure-creation time. This makes `(lambda: 1).__code__`
                // a real code object with a real co_firstlineno.
                {
                    let name_v = self.emit_str_const("<lambda>");
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: None,
                        name: "mb_func_set_name".to_string(),
                        args: vec![closure_vreg, name_v],
                        ty: self.tcx.none(),
                    });
                    let ac_raw = self.fresh_vreg();
                    self.current_stmts.push(MirInst::LoadConst {
                        dest: ac_raw,
                        value: MirConst::Int(lambda_arity as i64),
                        ty: self.tcx.int(),
                    });
                    let ac_boxed = self.box_operand(ac_raw, self.tcx.int());
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: None,
                        name: "mb_func_set_argcount".to_string(),
                        args: vec![closure_vreg, ac_boxed],
                        ty: self.tcx.none(),
                    });
                    let param_names: Vec<String> = params
                        .iter()
                        .enumerate()
                        .map(|(i, (sym, _))| {
                            self.sym_names
                                .get(sym)
                                .cloned()
                                .unwrap_or_else(|| format!("arg{i}"))
                        })
                        .collect();
                    let name_vregs: Vec<VReg> =
                        param_names.iter().map(|n| self.emit_str_const(n)).collect();
                    let names_list = self.fresh_vreg();
                    self.current_stmts.push(MirInst::MakeList {
                        dest: names_list,
                        elements: name_vregs,
                        ty: any_ty,
                    });
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: None,
                        name: "mb_func_set_varnames".to_string(),
                        args: vec![closure_vreg, names_list],
                        ty: self.tcx.none(),
                    });
                    // FUNC_PARAMS: (name, kind=1, has_default, default, None)
                    // per param — defaults reuse the already-evaluated outer
                    // vregs (positionally trailing, Python rule).
                    let n_defaults = defaults.iter().filter(|d| d.is_some()).count();
                    let first_default_idx = params.len().saturating_sub(n_defaults);
                    let mut default_iter = default_vregs_meta.iter();
                    let mut param_tup_vregs: Vec<VReg> = Vec::new();
                    for (i, pname) in param_names.iter().enumerate() {
                        let pn_vreg = self.emit_str_const(pname);
                        let kind_raw = self.fresh_vreg();
                        // Per-param CPython kind ordinal threaded from the AST
                        // (`/` → posonly, `*`/`**` → var, `*`-after → kwonly);
                        // fall back to POSITIONAL_OR_KEYWORD if unavailable.
                        let kind_ord = param_kinds.get(i).copied().unwrap_or(1) as i64;
                        self.current_stmts.push(MirInst::LoadConst {
                            dest: kind_raw,
                            value: MirConst::Int(kind_ord),
                            ty: self.tcx.int(),
                        });
                        let kind_vreg = self.box_operand(kind_raw, self.tcx.int());
                        let has_default = i >= first_default_idx;
                        let hd_raw = self.fresh_vreg();
                        self.current_stmts.push(MirInst::LoadConst {
                            dest: hd_raw,
                            value: MirConst::Int(if has_default { 1 } else { 0 }),
                            ty: self.tcx.int(),
                        });
                        let hd_vreg = self.box_operand(hd_raw, self.tcx.int());
                        let def_vreg = if has_default {
                            default_iter
                                .next()
                                .copied()
                                .unwrap_or_else(|| self.emit_none())
                        } else {
                            self.emit_none()
                        };
                        let anno_vreg = self.emit_none();
                        let tup = self.fresh_vreg();
                        self.current_stmts.push(MirInst::MakeTuple {
                            dest: tup,
                            elements: vec![pn_vreg, kind_vreg, hd_vreg, def_vreg, anno_vreg],
                            ty: any_ty,
                        });
                        param_tup_vregs.push(tup);
                    }
                    let params_list = self.fresh_vreg();
                    self.current_stmts.push(MirInst::MakeList {
                        dest: params_list,
                        elements: param_tup_vregs,
                        ty: any_ty,
                    });
                    self.current_stmts.push(MirInst::CallExtern {
                        dest: None,
                        name: "mb_func_set_params".to_string(),
                        args: vec![closure_vreg, params_list],
                        ty: self.tcx.none(),
                    });
                    if let Some(starts) = &self.src_line_starts {
                        if span.end > 0 {
                            let line = starts.partition_point(|&s| s <= span.start) as i64;
                            let filename = self.src_filename.clone().unwrap_or_default();
                            let line_raw = self.fresh_vreg();
                            self.current_stmts.push(MirInst::LoadConst {
                                dest: line_raw,
                                value: MirConst::Int(line),
                                ty: self.tcx.int(),
                            });
                            let line_vreg = self.box_operand(line_raw, self.tcx.int());
                            let file_vreg = self.emit_str_const(&filename);
                            self.current_stmts.push(MirInst::CallExtern {
                                dest: None,
                                name: "mb_func_set_srcinfo".to_string(),
                                args: vec![closure_vreg, line_vreg, file_vreg],
                                ty: self.tcx.none(),
                            });
                        }
                    }
                }
                closure_vreg
            }
            HirExpr::Yield { value, ty } => {
                let val = value
                    .as_ref()
                    .map(|v| {
                        let raw = self.lower_expr(v);
                        self.box_operand(raw, v.ty())
                    })
                    .unwrap_or_else(|| self.emit_none());
                let dest = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(dest),
                    name: "mb_generator_yield_value".to_string(),
                    args: vec![val],
                    ty: *ty,
                });
                // Post-yield exception check: if throw()/close() injected an
                // exception, branch to the enclosing try handler (or return).
                self.emit_post_yield_exc_check(dest);
                dest
            }
            HirExpr::YieldFrom { iter, ty } => {
                let raw_it = self.lower_expr(iter);
                let it = self.box_operand(raw_it, iter.ty());
                let dest = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(dest),
                    name: "mb_generator_yield_from".to_string(),
                    args: vec![it],
                    ty: *ty,
                });
                // Post-yield exception check (yield from can also receive throw)
                self.emit_post_yield_exc_check(dest);
                dest
            }
            HirExpr::Await { value, ty } => {
                let val = self.lower_expr(value);
                // GIL release before await (#313 R3)
                self.current_stmts.push(MirInst::CallExtern {
                    dest: None,
                    name: "mb_gil_release".to_string(),
                    args: vec![],
                    ty: self.tcx.none(),
                });
                let dest = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(dest),
                    name: "mb_await".to_string(),
                    args: vec![val],
                    ty: *ty,
                });
                // GIL acquire after await (#313 R3)
                self.current_stmts.push(MirInst::CallExtern {
                    dest: None,
                    name: "mb_gil_acquire".to_string(),
                    args: vec![],
                    ty: self.tcx.none(),
                });
                dest
            }
            HirExpr::ListComp {
                element,
                generators,
                ty,
            } => {
                // Snapshot the local vreg map so a comp-walrus target bound to an
                // enclosing/global symbol can be dropped afterwards (PEP 572:
                // `[(j := i) for ...]` leaves `j` in the enclosing scope). The comp
                // walrus also StoreGlobals such a target; the stale comp-internal
                // vreg must NOT shadow that global on the post-comp read, while
                // within-comp reads (accumulators `(acc := acc + i)`) still use the
                // live vreg. Loop vars are restored by lower_comprehension_loops, so
                // the only changed <1M/cell entries left are such walrus targets.
                let pre_vregs = self.sym_to_vreg.clone();
                let list = self.fresh_vreg();
                self.current_stmts.push(MirInst::MakeList {
                    dest: list,
                    elements: Vec::new(),
                    ty: *ty,
                });
                let none_ty = self.tcx.none();
                let element = element.clone();
                self.lower_comprehension_loops(generators, |this| {
                    let elem_raw = this.lower_expr(&element);
                    // NaN-box the element per its static type before appending, so a
                    // constant-literal element (e.g. `[1 for _ in ...]`, whose
                    // lower_expr yields a raw unboxed Int/Float/Bool) is stored as a
                    // tagged MbValue rather than a mistagged raw i64. box_operand is a
                    // no-op for already-boxed kinds (Any/str/list/loop-vars), matching
                    // the list-literal path.
                    let elem = this.box_operand(elem_raw, element.ty());
                    // Use unchecked append for comprehensions: the list is local to
                    // this scope and there are no concurrent readers, so we can skip
                    // the RwLock try_write dance.
                    this.current_stmts.push(MirInst::CallExtern {
                        dest: None,
                        name: "mb_list_append_unchecked".to_string(),
                        args: vec![list, elem],
                        ty: none_ty,
                    });
                });
                let changed: Vec<SymbolId> = self.sym_to_vreg.iter()
                    .filter(|(s, v)| pre_vregs.get(*s).copied() != Some(**v))
                    .map(|(s, _)| *s)
                    .collect();
                for s in changed {
                    if s.0 < 1_000_000 || self.cell_override.contains(&s.0) {
                        self.sym_to_vreg.remove(&s);
                    }
                }
                list
            }
            HirExpr::AnyAllComp {
                is_all,
                element,
                generators,
                ..
            } => self.lower_any_all_comp(*is_all, element, generators),
            HirExpr::SetComp {
                element,
                generators,
                ty,
            } => {
                // Build a list first, then convert to set
                let list = self.fresh_vreg();
                self.current_stmts.push(MirInst::MakeList {
                    dest: list,
                    elements: Vec::new(),
                    ty: *ty,
                });
                let none_ty = self.tcx.none();
                let element = element.clone();
                self.lower_comprehension_loops(generators, |this| {
                    let elem_raw = this.lower_expr(&element);
                    // NaN-box per static type (no-op for already-boxed kinds) so a
                    // constant-literal set element is tagged, not a raw i64.
                    let elem = this.box_operand(elem_raw, element.ty());
                    this.current_stmts.push(MirInst::CallExtern {
                        dest: None,
                        name: "mb_list_append".to_string(),
                        args: vec![list, elem],
                        ty: none_ty,
                    });
                });
                let set_vreg = self.fresh_vreg();
                self.current_stmts.push(MirInst::CallExtern {
                    dest: Some(set_vreg),
                    name: "mb_set_from_list".to_string(),
                    args: vec![list],
                    ty: *ty,
                });
                set_vreg
            }
            HirExpr::DictComp {
                key,
                value,
                generators,
                ty,
            } => {
                let dict = self.fresh_vreg();
                self.current_stmts.push(MirInst::MakeDict {
                    dest: dict,
                    keys: Vec::new(),
                    values: Vec::new(),
                    ty: *ty,
                });
                let none_ty = self.tcx.none();
                let key = key.clone();
                let value = value.clone();
                self.lower_comprehension_loops(generators, |this| {
                    let k_raw = this.lower_expr(&key);
                    let v_raw = this.lower_expr(&value);
                    // NaN-box key and value per their static types (no-op for already-
                    // boxed kinds) so constant-literal keys/values are tagged MbValues.
                    let k = this.box_operand(k_raw, key.ty());
                    let v = this.box_operand(v_raw, value.ty());
                    this.current_stmts.push(MirInst::CallExtern {
                        dest: None,
                        name: "mb_dict_setitem".to_string(),
                        args: vec![dict, k, v],
                        ty: none_ty,
                    });
                });
                dict
            }
            HirExpr::FString { parts, ty } => {
                // Concatenate all parts via runtime string ops
                let dest = self.fresh_vreg();
                let mut part_vregs = Vec::new();
                for part in parts {
                    match part {
                        HirFStringPart::Literal(s) => {
                            part_vregs.push(self.emit_str_const(s));
                        }
                        HirFStringPart::Expr(e, spec) => {
                            let expr_ty = e.ty();
                            let v = self.lower_expr(e);
                            let boxed = self.box_operand(v, expr_ty);
                            let sv = self.fresh_vreg();
                            if let Some(spec_parts) = spec {
                                // Build the spec string: static parts are
                                // constants, nested replacement fields
                                // ({value:{width}}) evaluate then stringify.
                                let mut spec_vregs: Vec<VReg> = Vec::new();
                                for sp in spec_parts {
                                    match sp {
                                        HirFStringPart::Literal(s) => {
                                            spec_vregs.push(self.emit_str_const(s));
                                        }
                                        HirFStringPart::Expr(se, _) => {
                                            let se_ty = se.ty();
                                            let sv2 = self.lower_expr(se);
                                            let sb = self.box_operand(sv2, se_ty);
                                            let out = self.fresh_vreg();
                                            self.current_stmts.push(MirInst::CallExtern {
                                                dest: Some(out),
                                                name: "mb_str".to_string(),
                                                args: vec![sb],
                                                ty: *ty,
                                            });
                                            spec_vregs.push(out);
                                        }
                                    }
                                }
                                let spec_vreg = if spec_vregs.is_empty() {
                                    self.emit_str_const("")
                                } else if spec_vregs.len() == 1 {
                                    spec_vregs[0]
                                } else {
                                    let mut acc = spec_vregs[0];
                                    for &pv in &spec_vregs[1..] {
                                        let next = self.fresh_vreg();
                                        self.current_stmts.push(MirInst::CallExtern {
                                            dest: Some(next),
                                            name: "mb_str_concat".to_string(),
                                            args: vec![acc, pv],
                                            ty: *ty,
                                        });
                                        acc = next;
                                    }
                                    acc
                                };
                                self.current_stmts.push(MirInst::CallExtern {
                                    dest: Some(sv),
                                    name: "mb_format_value".to_string(),
                                    args: vec![boxed, spec_vreg],
                                    ty: *ty,
                                });
                            } else {
                                // No spec: format(value, "") semantics — an
                                // instance with __format__ dispatches it;
                                // everything else takes the str() fast path.
                                self.current_stmts.push(MirInst::CallExtern {
                                    dest: Some(sv),
                                    name: "mb_fstring_value".to_string(),
                                    args: vec![boxed],
                                    ty: *ty,
                                });
                            }
                            part_vregs.push(sv);
                        }
                    }
                }
                // Build result by concatenating parts.
                // Each concat uses a fresh dest VReg to avoid aliasing the
                // accumulator as both input arg and output dest — the JIT
                // emits mb_release_value(old_dest) before reading args, which
                // would free the input string before mb_str_concat reads it.
                if part_vregs.is_empty() {
                    self.current_stmts.push(MirInst::LoadConst {
                        dest,
                        value: MirConst::Str(String::new()),
                        ty: *ty,
                    });
                } else if part_vregs.len() == 1 {
                    self.current_stmts.push(MirInst::Copy {
                        dest,
                        source: part_vregs[0],
                    });
                } else {
                    let mut acc = part_vregs[0];
                    for &pv in &part_vregs[1..] {
                        let next = self.fresh_vreg();
                        self.current_stmts.push(MirInst::CallExtern {
                            dest: Some(next),
                            name: "mb_str_concat".to_string(),
                            args: vec![acc, pv],
                            ty: *ty,
                        });
                        acc = next;
                    }
                    self.current_stmts.push(MirInst::Copy { dest, source: acc });
                }
                dest
            }
            HirExpr::Walrus {
                target,
                value,
                ty: _,
            } => {
                // PEP 572: evaluate value, assign to target, return value.
                let val_vreg = self.lower_expr(value);
                // Store to target's vreg (or create a new binding)
                if let Some(&existing) = self.sym_to_vreg.get(target) {
                    self.current_stmts.push(MirInst::Copy {
                        dest: existing,
                        source: val_vreg,
                    });
                } else {
                    self.sym_to_vreg.insert(*target, val_vreg);
                }
                // A walrus target binds in the *enclosing* scope (PEP 572). It
                // needs a global store to cross a scope boundary when: we're at
                // module scope (the enclosing scope IS global); the resolver
                // bound it to a real (non-synthetic, < 1_000_000) module/
                // enclosing symbol; or it is a nonlocal-shared cell variable
                // (in cell_override) — e.g. `nonlocal v; [(v := x) for ...]`
                // must propagate to the outer frame, mirroring the cell store a
                // regular assignment emits. A walrus inside a regular function
                // body binds a function-LOCAL (a synthetic sym >= 1_000_000
                // that is NOT a cell): it must NOT leak to module globals
                // (function_local_does_not_escape).
                if self.in_module_scope
                    || target.0 < 1_000_000
                    || self.cell_override.contains(&target.0)
                {
                    self.current_stmts.push(MirInst::StoreGlobal {
                        name: *target,
                        value: val_vreg,
                    });
                }
                val_vreg
            }
        }
    }

    fn finish_block(&mut self, terminator: Terminator) {
        let stmts = std::mem::take(&mut self.current_stmts);
        let id = self
            .current_block_id
            .take()
            .expect("finish_block called without an open block");
        self.blocks.push(BasicBlock {
            id,
            stmts,
            terminator,
        });
    }

    fn start_block(&mut self, id: BlockId) {
        self.current_block_id = Some(id);
    }

    /// Helper: emit a CallExtern with no args and optional dest.
    fn emit_extern_call(&mut self, dest: Option<VReg>, name: &str) {
        self.current_stmts.push(MirInst::CallExtern {
            dest,
            name: name.to_string(),
            args: Vec::new(),
            ty: self.tcx.none(),
        });
    }

    /// Inside a try block, emit `mb_has_exception()` check after a call.
    /// If an exception is pending, branch to the handler immediately
    /// (Python semantics: exceptions propagate at each call site).
    fn emit_try_exception_guard(&mut self) {
        if let Some(&(handler_block, _)) = self.try_handler_stack.last() {
            let exc_check = self.fresh_vreg();
            self.current_stmts.push(MirInst::CallExtern {
                dest: Some(exc_check),
                name: "mb_has_exception".to_string(),
                args: Vec::new(),
                ty: self.tcx.bool(),
            });
            let continue_block = self.fresh_block();
            self.finish_block(Terminator::Branch {
                cond: exc_check,
                then_block: handler_block,
                else_block: continue_block,
            });
            self.start_block(continue_block);
        }
    }

    /// After a try/except/finally completes, if the current CURRENT_EXCEPTION
    /// is still set (a handler reraised, or `except*` left unmatched sub-groups
    /// pending), propagate: jump to the nearest outer try's handler if any,
    /// otherwise return None so the caller's try-exception guard can pick up
    /// the pending exception.
    fn emit_exception_propagate(&mut self) {
        let exc_check = self.fresh_vreg();
        self.current_stmts.push(MirInst::CallExtern {
            dest: Some(exc_check),
            name: "mb_has_exception".to_string(),
            args: Vec::new(),
            ty: self.tcx.bool(),
        });
        let continue_block = self.fresh_block();
        let propagate_block = self.fresh_block();
        self.finish_block(Terminator::Branch {
            cond: exc_check,
            then_block: propagate_block,
            else_block: continue_block,
        });
        self.start_block(propagate_block);
        // When a `with` body is active and no `try` was pushed *after* it
        // (recorded try-depth equals the current try-handler depth), route the
        // pending exception to that `with`'s exit sequence so `__exit__` runs
        // (suppression / re-raise) before unwinding further. Otherwise prefer
        // the innermost `try` handler, then fall back to returning from the
        // function so the caller's exception guard observes the pending state.
        let with_exit = self.with_exit_stack.last().copied();
        let cur_try_depth = self.try_handler_stack.len();
        if let Some((exit_block, try_depth)) = with_exit {
            if try_depth == cur_try_depth {
                self.finish_block(Terminator::Goto(exit_block));
                self.start_block(continue_block);
                return;
            }
        }
        if let Some(&(handler_block, _)) = self.try_handler_stack.last() {
            self.emit_extern_call(None, "mb_pop_handler");
            self.finish_block(Terminator::Goto(handler_block));
        } else {
            let none_val = self.emit_none();
            self.finish_block(Terminator::Return(Some(none_val)));
        }
        self.start_block(continue_block);
    }

    /// Helper: emit an integer constant and return its vreg.
    fn emit_int_const(&mut self, n: i64) -> VReg {
        let dest = self.fresh_vreg();
        self.current_stmts.push(MirInst::LoadConst {
            dest,
            value: MirConst::Int(n),
            ty: self.tcx.int(),
        });
        dest
    }

    /// Helper: emit a string constant and return its vreg.
    fn emit_str_const(&mut self, s: &str) -> VReg {
        let dest = self.fresh_vreg();
        self.current_stmts.push(MirInst::LoadConst {
            dest,
            value: MirConst::Str(s.to_string()),
            ty: self.tcx.str(),
        });
        dest
    }

    /// Helper: emit a None constant.
    fn emit_none(&mut self) -> VReg {
        let dest = self.fresh_vreg();
        self.current_stmts.push(MirInst::LoadConst {
            dest,
            value: MirConst::None,
            ty: self.tcx.none(),
        });
        dest
    }

    // HANDWRITE-BEGIN gap="standardize:projects-mamba-src-lower-hir-to-mir-rs" tracker="standardize-gap-projects-mamba-src-lower-hir-to-mir-rs" reason="introspection-builtins (issue: enhancement-mamba-introspection-builtins-globals-locals-vars-dir)."
    /// Synthesize an inline `locals()` snapshot dict from the lowerer's
    /// current `sym_to_vreg` map. Emits:
    ///   dest = mb_dict_new()
    ///   for (sym, vreg) in sym_to_vreg:
    ///     mb_dict_setitem(dest, "name", boxed(vreg))
    /// Used by `locals()` and `vars()` (no-arg) inside function bodies.
    /// At module / class scope the runtime helper `mb_locals` is called instead
    /// (returns the module globals dict).
    /// @spec .aw/tech-design/cclab-mamba/logic/introspection-builtins.md#locals_impl
    fn emit_locals_snapshot_dict(&mut self, dest: VReg, ty: TypeId) -> VReg {
        // dest = mb_dict_new()
        self.current_stmts.push(MirInst::CallExtern {
            dest: Some(dest),
            name: "mb_dict_new".to_string(),
            args: vec![],
            ty,
        });
        // Snapshot the map up front; box_operand mutates current_stmts and may
        // need fresh_vreg, but should not see borrow conflicts with sym_to_vreg.
        let pairs: Vec<(SymbolId, VReg)> = self.sym_to_vreg.iter().map(|(s, v)| (*s, *v)).collect();
        for (sym_id, vreg) in pairs {
            // Resolve the local's source name. Prefer `hir.sym_names` (covers
            // synthetic / lowered IDs ≥ 1M); fall back to `symbol_table` for
            // canonical type-checker IDs (low IDs only — the table has no
            // entry for synthetic IDs and bare-indexes by id.0, so we must
            // bounds-check before calling get_symbol).
            let name = if let Some(n) = self.sym_names.get(&sym_id) {
                Some(n.clone())
            } else if let Some(st) = self.symbol_table {
                let all = st.all_symbols();
                if (sym_id.0 as usize) < all.len() {
                    Some(all[sym_id.0 as usize].name.clone())
                } else {
                    None
                }
            } else {
                None
            };
            let Some(name) = name else { continue };
            // Skip dunders and the implicit __main__ id; CPython's locals()
            // doesn't list them at the call point.
            if name.starts_with("__") {
                continue;
            }
            // Box primitive values so the dict holds NaN-boxed MbValues.
            let value_ty = self
                .sym_types
                .get(&sym_id)
                .copied()
                .unwrap_or(self.tcx.any());
            let boxed = self.box_operand(vreg, value_ty);
            let key_vreg = self.emit_str_const(&name);
            self.current_stmts.push(MirInst::CallExtern {
                dest: None,
                name: "mb_dict_setitem".to_string(),
                args: vec![dest, key_vreg, boxed],
                ty: self.tcx.none(),
            });
        }
        dest
    }
    // HANDWRITE-END

    /// Emit a post-yield exception check for generator throw()/close().
    ///
    /// After `mb_generator_yield_value` returns, if the caller used throw() or
    /// close(), the runtime sets a pending exception via `mb_raise()`.  The
    /// compiled code must detect this and jump to the enclosing try handler (or
    /// return from the body if there is no enclosing try).
    ///
    /// `result_vreg` is the VReg holding the yield return value; it is forwarded
    /// to the continuation block so that subsequent code still sees it.
    fn emit_post_yield_exc_check(&mut self, _result_vreg: VReg) {
        if !self.is_gen_body {
            return;
        }

        let exc_check = self.fresh_vreg();
        self.current_stmts.push(MirInst::CallExtern {
            dest: Some(exc_check),
            name: "mb_has_exception".to_string(),
            args: Vec::new(),
            ty: self.tcx.bool(),
        });

        let continue_block = self.fresh_block();

        if let Some(&(handler_block, _finally_block)) = self.try_handler_stack.last() {
            // Inside a try block — pop the handler and jump to the handler block.
            let exc_block = self.fresh_block();
            self.finish_block(Terminator::Branch {
                cond: exc_check,
                then_block: exc_block,
                else_block: continue_block,
            });
            self.start_block(exc_block);
            self.emit_extern_call(None, "mb_pop_handler");
            self.finish_block(Terminator::Goto(handler_block));
        } else {
            // Outside any try — return from the generator body so the exception
            // propagates via the Returned channel message.
            let exc_block = self.fresh_block();
            self.finish_block(Terminator::Branch {
                cond: exc_check,
                then_block: exc_block,
                else_block: continue_block,
            });
            self.start_block(exc_block);
            let none_val = self.emit_none();
            self.finish_block(Terminator::Return(Some(none_val)));
        }

        self.start_block(continue_block);
    }

    /// Box a primitive operand for runtime dispatch.
    fn box_operand(&mut self, vreg: VReg, ty_id: TypeId) -> VReg {
        let ty = self.tcx.get(ty_id);
        let box_fn = match ty {
            crate::types::Ty::Int => Some("mb_box_int"),
            crate::types::Ty::Float => Some("mb_box_float"),
            crate::types::Ty::Bool => Some("mb_box_bool"),
            _ => None, // already NaN-boxed (str, list, etc.)
        };
        if let Some(name) = box_fn {
            let boxed = self.fresh_vreg();
            self.current_stmts.push(MirInst::CallExtern {
                dest: Some(boxed),
                name: name.to_string(),
                args: vec![vreg],
                ty: self.tcx.any(),
            });
            boxed
        } else {
            vreg
        }
    }
}

/// Collect all SymbolIds bound by a pattern (for OR-pattern merge vreg allocation).
/// Does NOT include wildcards (they bind nothing by name).
fn collect_pattern_bindings(pattern: &HirPattern, out: &mut Vec<SymbolId>) {
    use HirPattern::*;
    match pattern {
        Wildcard | Literal(_) => {}
        Capture(sym) => out.push(*sym),
        Or(alts) => {
            for alt in alts {
                collect_pattern_bindings(alt, out);
            }
        }
        Sequence(pats) => {
            for pat in pats {
                collect_pattern_bindings(pat, out);
            }
        }
        Star(Some(sym)) => out.push(*sym),
        Star(None) => {}
        Class { args, .. } => {
            for (_, pat) in args {
                collect_pattern_bindings(pat, out);
            }
        }
        Mapping { pairs, rest } => {
            for (_, val_pat) in pairs {
                collect_pattern_bindings(val_pat, out);
            }
            if let Some(sym) = rest {
                out.push(*sym);
            }
        }
        As {
            pattern: inner,
            name,
        } => {
            collect_pattern_bindings(inner, out);
            out.push(*name);
        }
    }
}

fn lower_mir_binop(op: HirBinOp) -> MirBinOp {
    match op {
        HirBinOp::Add => MirBinOp::Add,
        HirBinOp::Sub => MirBinOp::Sub,
        HirBinOp::Mul => MirBinOp::Mul,
        HirBinOp::Div => MirBinOp::Div,
        HirBinOp::FloorDiv => MirBinOp::FloorDiv,
        HirBinOp::Mod => MirBinOp::Mod,
        HirBinOp::Pow => MirBinOp::Pow,
        HirBinOp::Eq => MirBinOp::Eq,
        HirBinOp::NotEq => MirBinOp::NotEq,
        HirBinOp::Lt => MirBinOp::Lt,
        HirBinOp::Gt => MirBinOp::Gt,
        HirBinOp::LtEq => MirBinOp::LtEq,
        HirBinOp::GtEq => MirBinOp::GtEq,
        HirBinOp::And => MirBinOp::And,
        HirBinOp::Or => MirBinOp::Or,
        HirBinOp::BitAnd => MirBinOp::BitAnd,
        HirBinOp::BitOr => MirBinOp::BitOr,
        HirBinOp::BitXor => MirBinOp::BitXor,
        HirBinOp::LShift => MirBinOp::LShift,
        HirBinOp::RShift => MirBinOp::RShift,
        HirBinOp::Is => MirBinOp::Is,
        HirBinOp::IsNot => MirBinOp::IsNot,
        HirBinOp::In => MirBinOp::In,
        HirBinOp::NotIn => MirBinOp::NotIn,
    }
}

/// Map HirBinOp to runtime mb_* function name for mixed-type dispatch.
fn binop_to_runtime(op: HirBinOp) -> Option<&'static str> {
    match op {
        HirBinOp::Add => Some("mb_add"),
        HirBinOp::Sub => Some("mb_sub"),
        HirBinOp::Mul => Some("mb_mul"),
        HirBinOp::Div => Some("mb_div"),
        HirBinOp::FloorDiv => Some("mb_floordiv"),
        HirBinOp::Mod => Some("mb_mod"),
        HirBinOp::Pow => Some("mb_pow"),
        HirBinOp::Eq => Some("mb_eq"),
        HirBinOp::NotEq => Some("mb_ne"),
        HirBinOp::Lt => Some("mb_lt"),
        HirBinOp::Gt => Some("mb_gt"),
        HirBinOp::LtEq => Some("mb_le"),
        HirBinOp::GtEq => Some("mb_ge"),
        HirBinOp::BitOr => Some("mb_bitor"),
        HirBinOp::BitAnd => Some("mb_bitand"),
        HirBinOp::BitXor => Some("mb_bitxor"),
        HirBinOp::LShift => Some("mb_lshift"),
        HirBinOp::RShift => Some("mb_rshift"),
        _ => None, // And, Or — stay primitive
    }
}

fn lower_mir_unaryop(op: HirUnaryOp) -> MirUnaryOp {
    match op {
        HirUnaryOp::Pos => MirUnaryOp::Pos,
        HirUnaryOp::Neg => MirUnaryOp::Neg,
        HirUnaryOp::Not => MirUnaryOp::Not,
        HirUnaryOp::BitNot => MirUnaryOp::BitNot,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::span::Span;

    #[test]
    fn test_lower_simple_function() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();

        let hir = HirModule {
            functions: vec![HirFunction {
                name: SymbolId(0),
                params: vec![(SymbolId(1), int_ty)],
                return_ty: int_ty,
                body: vec![HirStmt::Return {
                    value: Some(HirExpr::Var(SymbolId(1), int_ty)),
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
                captures: Vec::new(),
                is_async: false,
                is_generator: false,
                decorators: Vec::new(),
                has_star_args: false,
                star_param_pos: None,
                has_kwargs: false,
            }],
            classes: Vec::new(),
            top_level: Vec::new(),
            imports: Vec::new(),
            sym_names: std::collections::HashMap::new(),
            sym_types: std::collections::HashMap::new(),
            module_annotations: Vec::new(),
            func_sigs: HashMap::new(),
        };

        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
        assert_eq!(mir.bodies[0].params.len(), 1);
        // Should have at least one block with Return terminator
        assert!(!mir.bodies[0].blocks.is_empty());
    }

    #[test]
    fn test_lower_if_stmt() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let bool_ty = tcx.bool();

        let hir = HirModule {
            functions: vec![HirFunction {
                name: SymbolId(0),
                params: vec![(SymbolId(1), int_ty)],
                return_ty: int_ty,
                body: vec![HirStmt::If {
                    cond: HirExpr::BoolLit(true, bool_ty),
                    then_body: vec![HirStmt::Return {
                        value: Some(HirExpr::IntLit(1, int_ty)),
                        span: Span::dummy(),
                    }],
                    else_body: vec![HirStmt::Return {
                        value: Some(HirExpr::IntLit(0, int_ty)),
                        span: Span::dummy(),
                    }],
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
                captures: Vec::new(),
                is_async: false,
                is_generator: false,
                decorators: Vec::new(),
                has_star_args: false,
                star_param_pos: None,
                has_kwargs: false,
            }],
            classes: Vec::new(),
            top_level: Vec::new(),
            imports: Vec::new(),
            sym_names: std::collections::HashMap::new(),
            sym_types: std::collections::HashMap::new(),
            module_annotations: Vec::new(),
            func_sigs: HashMap::new(),
        };

        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
        // If statement should generate multiple blocks
        assert!(mir.bodies[0].blocks.len() >= 3, "expected ≥3 blocks for if");
    }

    fn make_top_level_hir(stmts: Vec<HirStmt>) -> HirModule {
        HirModule {
            functions: Vec::new(),
            classes: Vec::new(),
            top_level: stmts,
            imports: Vec::new(),
            sym_names: std::collections::HashMap::new(),
            sym_types: std::collections::HashMap::new(),
            module_annotations: Vec::new(),
            func_sigs: HashMap::new(),
        }
    }

    #[test]
    fn test_lower_raise_bare() {
        let tcx = TypeContext::new();
        let hir = make_top_level_hir(vec![HirStmt::Raise {
            value: None,
            from: None,
            span: Span::dummy(),
        }]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
        // bare raise emits MirInst::Raise { value: None }
        let stmts = &mir.bodies[0].blocks[0].stmts;
        assert!(stmts
            .iter()
            .any(|s| matches!(s, MirInst::Raise { value: None })));
    }

    #[test]
    fn test_lower_raise_with_value() {
        let tcx = TypeContext::new();
        let any_ty = tcx.any();
        let hir = make_top_level_hir(vec![HirStmt::Raise {
            value: Some(HirExpr::StrLit("oops".to_string(), any_ty)),
            from: None,
            span: Span::dummy(),
        }]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        // raise "oops" now lowers to CallExtern { name: "mb_raise_instance" }
        assert!(all_stmts.iter().any(|s| matches!(s,
            MirInst::CallExtern { name, .. } if name == "mb_raise_instance"
        )));
    }

    #[test]
    fn test_lower_assert_no_msg() {
        let tcx = TypeContext::new();
        let bool_ty = tcx.bool();
        let hir = make_top_level_hir(vec![HirStmt::Assert {
            test: HirExpr::BoolLit(true, bool_ty),
            msg: None,
            span: Span::dummy(),
        }]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
        // Assert without msg branches and calls mb_assertion_error_no_msg
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        assert!(all_stmts.iter().any(|s| matches!(
            s, MirInst::CallExtern { name, .. } if name == "mb_assertion_error_no_msg"
        )));
    }

    #[test]
    fn test_lower_assert_with_msg() {
        let tcx = TypeContext::new();
        let bool_ty = tcx.bool();
        let any_ty = tcx.any();
        let hir = make_top_level_hir(vec![HirStmt::Assert {
            test: HirExpr::BoolLit(false, bool_ty),
            msg: Some(HirExpr::StrLit("failed".to_string(), any_ty)),
            span: Span::dummy(),
        }]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        assert!(all_stmts.iter().any(|s| matches!(
            s, MirInst::CallExtern { name, .. } if name == "mb_assertion_error"
        )));
    }

    #[test]
    fn test_lower_binop_floordiv_int() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let hir = make_top_level_hir(vec![HirStmt::Expr {
            expr: HirExpr::BinOp {
                op: HirBinOp::FloorDiv,
                lhs: Box::new(HirExpr::IntLit(10, int_ty)),
                rhs: Box::new(HirExpr::IntLit(3, int_ty)),
                ty: int_ty,
            },
            span: Span::dummy(),
        }]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        assert!(all_stmts.iter().any(|s| matches!(
            s,
            MirInst::BinOp {
                op: MirBinOp::FloorDiv,
                ..
            }
        )));
    }

    #[test]
    fn test_lower_with_statement() {
        let tcx = TypeContext::new();
        let any_ty = tcx.any();
        let hir = make_top_level_hir(vec![HirStmt::With {
            items: vec![(HirExpr::StrLit("ctx".to_string(), any_ty), None)],
            body: vec![HirStmt::Expr {
                expr: HirExpr::IntLit(0, tcx.int()),
                span: Span::dummy(),
            }],
            is_async: false,
            span: Span::dummy(),
        }]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        // With desugars: calls mb_context_enter and mb_context_exit
        assert!(all_stmts.iter().any(|s| matches!(
            s, MirInst::CallExtern { name, .. } if name == "mb_context_enter"
        )));
        assert!(all_stmts.iter().any(|s| matches!(
            s, MirInst::CallExtern { name, .. } if name == "mb_context_exit"
        )));
    }

    #[test]
    fn test_lower_del_attr() {
        let tcx = TypeContext::new();
        let any_ty = tcx.any();
        let hir = make_top_level_hir(vec![HirStmt::Del {
            target: HirLValue::Attr {
                object: Box::new(HirExpr::StrLit("obj".to_string(), any_ty)),
                attr: "field".to_string(),
            },
            span: Span::dummy(),
        }]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        assert!(all_stmts.iter().any(|s| matches!(
            s, MirInst::CallExtern { name, .. } if name == "mb_delattr"
        )));
    }

    #[test]
    fn test_lower_del_index() {
        let tcx = TypeContext::new();
        let any_ty = tcx.any();
        let hir = make_top_level_hir(vec![HirStmt::Del {
            target: HirLValue::Index {
                object: Box::new(HirExpr::StrLit("lst".to_string(), any_ty)),
                index: Box::new(HirExpr::IntLit(0, tcx.int())),
            },
            span: Span::dummy(),
        }]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        assert!(all_stmts.iter().any(|s| matches!(
            s, MirInst::CallExtern { name, .. } if name == "mb_obj_delitem"
        )));
    }

    #[test]
    fn test_lower_del_var_unknown_sym_no_panic() {
        // REQ: del var — when the symbol has no vreg mapping (e.g. unknown sym),
        // no MIR is emitted and no panic occurs.
        let tcx = TypeContext::new();
        let hir = make_top_level_hir(vec![HirStmt::Del {
            target: HirLValue::Var(SymbolId(99)),
            span: Span::dummy(),
        }]);
        // SymbolId(99) has no vreg mapping — del emits nothing, no panic
        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
    }

    #[test]
    fn test_lower_del_var_known_sym_emits_mb_del_var() {
        // REQ: del var with known symbol emits CallExtern "mb_del_var"
        let tcx = TypeContext::new();
        let sym = SymbolId(0);
        let hir = make_top_level_hir(vec![
            HirStmt::Let {
                target: sym,
                ty: tcx.int(),
                value: HirExpr::IntLit(42, tcx.int()),
                span: Span::dummy(),
            },
            HirStmt::Del {
                target: HirLValue::Var(sym),
                span: Span::dummy(),
            },
        ]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        assert!(
            all_stmts.iter().any(|s| matches!(
                s, MirInst::CallExtern { name, .. } if name == "mb_del_var"
            )),
            "del on known var must emit mb_del_var CallExtern"
        );
    }

    #[test]
    fn test_lower_global_nonlocal_no_mir() {
        let tcx = TypeContext::new();
        // Global and Nonlocal are scope declarations — no MIR instructions emitted
        let hir = make_top_level_hir(vec![
            HirStmt::Global {
                names: vec![SymbolId(1)],
                span: Span::dummy(),
            },
            HirStmt::Nonlocal {
                names: vec![SymbolId(2)],
                span: Span::dummy(),
            },
        ]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
        // No Raise or CallExtern for global/nonlocal — just verify no panic
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        // Neither Global nor Nonlocal emits any MIR statement
        assert!(!all_stmts.iter().any(|s| matches!(s, MirInst::Raise { .. })));
    }

    #[test]
    fn test_lower_binop_pow() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let hir = make_top_level_hir(vec![HirStmt::Expr {
            expr: HirExpr::BinOp {
                op: HirBinOp::Pow,
                lhs: Box::new(HirExpr::IntLit(2, int_ty)),
                rhs: Box::new(HirExpr::IntLit(3, int_ty)),
                ty: int_ty,
            },
            span: Span::dummy(),
        }]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        assert!(all_stmts.iter().any(|s| matches!(
            s,
            MirInst::BinOp {
                op: MirBinOp::Pow,
                ..
            }
        )));
    }

    #[test]
    fn test_lower_binop_bitxor() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let hir = make_top_level_hir(vec![HirStmt::Expr {
            expr: HirExpr::BinOp {
                op: HirBinOp::BitXor,
                lhs: Box::new(HirExpr::IntLit(5, int_ty)),
                rhs: Box::new(HirExpr::IntLit(3, int_ty)),
                ty: int_ty,
            },
            span: Span::dummy(),
        }]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        assert!(all_stmts.iter().any(|s| matches!(
            s,
            MirInst::BinOp {
                op: MirBinOp::BitXor,
                ..
            }
        )));
    }

    #[test]
    fn test_lower_binop_bitor() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let hir = make_top_level_hir(vec![HirStmt::Expr {
            expr: HirExpr::BinOp {
                op: HirBinOp::BitOr,
                lhs: Box::new(HirExpr::IntLit(5, int_ty)),
                rhs: Box::new(HirExpr::IntLit(3, int_ty)),
                ty: int_ty,
            },
            span: Span::dummy(),
        }]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        assert!(all_stmts.iter().any(|s| matches!(
            s,
            MirInst::BinOp {
                op: MirBinOp::BitOr,
                ..
            }
        )));
    }

    #[test]
    fn test_lower_binop_bitand() {
        let tcx = TypeContext::new();
        let int_ty = tcx.int();
        let hir = make_top_level_hir(vec![HirStmt::Expr {
            expr: HirExpr::BinOp {
                op: HirBinOp::BitAnd,
                lhs: Box::new(HirExpr::IntLit(5, int_ty)),
                rhs: Box::new(HirExpr::IntLit(3, int_ty)),
                ty: int_ty,
            },
            span: Span::dummy(),
        }]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        assert!(all_stmts.iter().any(|s| matches!(
            s,
            MirInst::BinOp {
                op: MirBinOp::BitAnd,
                ..
            }
        )));
    }

    #[test]
    fn test_lower_await_expr() {
        let tcx = TypeContext::new();
        let any_ty = tcx.any();
        let hir = make_top_level_hir(vec![HirStmt::Expr {
            expr: HirExpr::Await {
                value: Box::new(HirExpr::StrLit("coro".to_string(), any_ty)),
                ty: any_ty,
            },
            span: Span::dummy(),
        }]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        assert!(all_stmts.iter().any(|s| matches!(
            s, MirInst::CallExtern { name, .. } if name == "mb_await"
        )));
        // GIL release/acquire should also be present
        assert!(all_stmts.iter().any(|s| matches!(
            s, MirInst::CallExtern { name, .. } if name == "mb_gil_release"
        )));
    }

    #[test]
    fn test_lower_yield_from_expr() {
        let tcx = TypeContext::new();
        let any_ty = tcx.any();
        let hir = make_top_level_hir(vec![HirStmt::Expr {
            expr: HirExpr::YieldFrom {
                iter: Box::new(HirExpr::StrLit("gen".to_string(), any_ty)),
                ty: any_ty,
            },
            span: Span::dummy(),
        }]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        assert!(all_stmts.iter().any(|s| matches!(
            s, MirInst::CallExtern { name, .. } if name == "mb_generator_yield_from"
        )));
    }

    // ── P0-R4: CallExtern return propagation tests ──────────────────────

    #[test]
    fn test_lower_call_extern_has_dest() {
        // Module-level function calls via CallExtern must store result to a dest register.
        // P0-R4.2: verify that CallExtern for module-level functions includes a dest VReg.
        let tcx = TypeContext::new();
        let any_ty = tcx.any();
        // Import a module, then call a function on it — the call should produce
        // a CallExtern with dest != None.
        let hir = HirModule {
            functions: vec![],
            classes: Vec::new(),
            top_level: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: Box::new(HirExpr::Attr {
                        object: Box::new(HirExpr::StrLit("math".to_string(), any_ty)),
                        attr: "sqrt".to_string(),
                        ty: any_ty,
                    }),
                    args: vec![HirExpr::IntLit(16, tcx.int())],
                    ty: any_ty,
                },
                span: Span::dummy(),
            }],
            imports: Vec::new(),
            sym_names: std::collections::HashMap::new(),
            sym_types: std::collections::HashMap::new(),
            module_annotations: Vec::new(),
            func_sigs: HashMap::new(),
        };
        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
        // The call chain should produce at least one CallExtern or CallMethod
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        // Should have at least one instruction (the call or its lowering)
        assert!(
            !all_stmts.is_empty(),
            "module function call should produce at least one MIR instruction"
        );
    }

    #[test]
    fn test_lower_with_statement_has_enter_exit_dest() {
        // P0-R2: With statement should emit CallExtern for both
        // mb_context_enter and mb_context_exit, with correct structure.
        let tcx = TypeContext::new();
        let any_ty = tcx.any();
        let hir = make_top_level_hir(vec![HirStmt::With {
            items: vec![(
                HirExpr::StrLit("ctx".to_string(), any_ty),
                Some(SymbolId(99)),
            )],
            body: vec![HirStmt::Expr {
                expr: HirExpr::IntLit(42, tcx.int()),
                span: Span::dummy(),
            }],
            is_async: false,
            span: Span::dummy(),
        }]);
        let mir = lower_hir_to_mir(&hir, &tcx);
        assert_eq!(mir.bodies.len(), 1);
        let all_stmts: Vec<_> = mir.bodies[0]
            .blocks
            .iter()
            .flat_map(|b| b.stmts.iter())
            .collect();
        // mb_context_enter should have a dest (to bind `as` variable)
        let enter_has_dest = all_stmts.iter().any(|s| {
            matches!(s, MirInst::CallExtern { dest: Some(_), name, .. } if name == "mb_context_enter")
        });
        assert!(
            enter_has_dest,
            "mb_context_enter should have a dest register for the `as` binding"
        );
        // mb_context_exit should also be present
        assert!(all_stmts.iter().any(|s| {
            matches!(s, MirInst::CallExtern { name, .. } if name == "mb_context_exit")
        }));
    }

    // ── Zero-arg constructor arity guard tests (#1109) ───────────────────────

    /// Helper: build an HIR with a builtin call expression and lower with symbol table.
    fn lower_builtin_call(builtin_name: &str, args: Vec<HirExpr>) -> MirModule {
        use crate::resolve::SymbolKind;
        let tcx = TypeContext::new();
        let any_ty = tcx.any();

        let mut symbols = SymbolTable::new();
        let sym = symbols.define(builtin_name.to_string(), SymbolKind::Function);

        let hir = HirModule {
            functions: Vec::new(),
            classes: Vec::new(),
            top_level: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: Box::new(HirExpr::Var(sym, any_ty)),
                    args,
                    ty: any_ty,
                },
                span: Span::dummy(),
            }],
            imports: Vec::new(),
            sym_names: std::collections::HashMap::new(),
            sym_types: std::collections::HashMap::new(),
            module_annotations: Vec::new(),
            func_sigs: HashMap::new(),
        };

        lower_hir_to_mir_with_symbols(&hir, &tcx, &symbols)
    }

    /// Helper: collect all CallExtern names from a MirModule.
    fn collect_extern_names(mir: &MirModule) -> Vec<String> {
        mir.bodies
            .iter()
            .flat_map(|b| b.blocks.iter())
            .flat_map(|blk| blk.stmts.iter())
            .filter_map(|s| match s {
                MirInst::CallExtern { name, .. } => Some(name.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn test_zero_arg_list_constructor_emits_mb_list_new() {
        let mir = lower_builtin_call("list", vec![]);
        let names = collect_extern_names(&mir);
        assert!(
            names.contains(&"mb_list_new".to_string()),
            "list() with 0 args should emit mb_list_new, got: {names:?}"
        );
        assert!(
            !names.contains(&"mb_list_from_iterable".to_string()),
            "list() with 0 args should NOT emit mb_list_from_iterable"
        );
    }

    #[test]
    fn test_zero_arg_tuple_constructor_emits_mb_tuple_new() {
        let mir = lower_builtin_call("tuple", vec![]);
        let names = collect_extern_names(&mir);
        assert!(
            names.contains(&"mb_tuple_new".to_string()),
            "tuple() with 0 args should emit mb_tuple_new, got: {names:?}"
        );
        assert!(
            !names.contains(&"mb_tuple_from_iterable".to_string()),
            "tuple() with 0 args should NOT emit mb_tuple_from_iterable"
        );
    }

    #[test]
    fn test_zero_arg_set_constructor_emits_mb_set_new() {
        let mir = lower_builtin_call("set", vec![]);
        let names = collect_extern_names(&mir);
        assert!(
            names.contains(&"mb_set_new".to_string()),
            "set() with 0 args should emit mb_set_new, got: {names:?}"
        );
        assert!(
            !names.contains(&"mb_set_from_iterable".to_string()),
            "set() with 0 args should NOT emit mb_set_from_iterable"
        );
    }

    #[test]
    fn test_zero_arg_dict_constructor_emits_mb_dict_new() {
        let mir = lower_builtin_call("dict", vec![]);
        let names = collect_extern_names(&mir);
        assert!(
            names.contains(&"mb_dict_new".to_string()),
            "dict() with 0 args should emit mb_dict_new, got: {names:?}"
        );
        assert!(
            !names.contains(&"mb_dict_from_pairs".to_string()),
            "dict() with 0 args should NOT emit mb_dict_from_pairs"
        );
    }

    #[test]
    fn test_one_arg_list_constructor_emits_mb_list_from_iterable() {
        let tcx = TypeContext::new();
        let any_ty = tcx.any();
        let mir = lower_builtin_call("list", vec![HirExpr::Var(SymbolId(999), any_ty)]);
        let names = collect_extern_names(&mir);
        assert!(
            names.contains(&"mb_list_from_iterable".to_string()),
            "list(x) with 1 arg should emit mb_list_from_iterable, got: {names:?}"
        );
        assert!(
            !names.contains(&"mb_list_new".to_string()),
            "list(x) with 1 arg should NOT emit mb_list_new"
        );
    }

    #[test]
    fn test_one_arg_tuple_constructor_emits_mb_tuple_from_iterable() {
        let tcx = TypeContext::new();
        let any_ty = tcx.any();
        let mir = lower_builtin_call("tuple", vec![HirExpr::Var(SymbolId(999), any_ty)]);
        let names = collect_extern_names(&mir);
        assert!(
            names.contains(&"mb_tuple_from_iterable".to_string()),
            "tuple(x) with 1 arg should emit mb_tuple_from_iterable, got: {names:?}"
        );
        assert!(
            !names.contains(&"mb_tuple_new".to_string()),
            "tuple(x) with 1 arg should NOT emit mb_tuple_new"
        );
    }

    #[test]
    fn test_one_arg_set_constructor_emits_mb_set_from_iterable() {
        let tcx = TypeContext::new();
        let any_ty = tcx.any();
        let mir = lower_builtin_call("set", vec![HirExpr::Var(SymbolId(999), any_ty)]);
        let names = collect_extern_names(&mir);
        assert!(
            names.contains(&"mb_set_from_iterable".to_string()),
            "set(x) with 1 arg should emit mb_set_from_iterable, got: {names:?}"
        );
        assert!(
            !names.contains(&"mb_set_new".to_string()),
            "set(x) with 1 arg should NOT emit mb_set_new"
        );
    }

    // REQ: tick-241 test-coverage — __name__ dunder init (#1133) emits StoreGlobal
    // when symbol_table contains "__name__" and top-level code references it.
    #[test]
    fn test_name_dunder_init_emits_store_global() {
        use crate::resolve::SymbolKind;
        let tcx = TypeContext::new();
        let any_ty = tcx.any();
        let mut symbols = SymbolTable::new();
        let name_sym = symbols.define("__name__".to_string(), SymbolKind::Variable);
        let hir = HirModule {
            functions: Vec::new(),
            classes: Vec::new(),
            top_level: vec![HirStmt::Expr {
                expr: HirExpr::Var(name_sym, any_ty),
                span: Span::dummy(),
            }],
            imports: Vec::new(),
            sym_names: std::collections::HashMap::new(),
            sym_types: std::collections::HashMap::new(),
            module_annotations: Vec::new(),
            func_sigs: HashMap::new(),
        };
        let mir = lower_hir_to_mir_with_symbols(&hir, &tcx, &symbols);
        let has_store = mir
            .bodies
            .iter()
            .flat_map(|b| b.blocks.iter())
            .flat_map(|blk| blk.stmts.iter())
            .any(|s| matches!(s, MirInst::StoreGlobal { name, .. } if *name == name_sym));
        assert!(
            has_store,
            "__name__ reference must emit StoreGlobal (#1133)"
        );
    }

    // REQ: tick-136 test-coverage — binop_to_runtime covers arithmetic/comparison/bit ops
    // with mb_* runtime extern names AND returns None for And/Or (primitive short-circuit).
    #[test]
    fn test_binop_to_runtime_short_circuit_ops_return_none() {
        assert_eq!(binop_to_runtime(HirBinOp::And), None);
        assert_eq!(binop_to_runtime(HirBinOp::Or), None);
        assert_eq!(binop_to_runtime(HirBinOp::Add), Some("mb_add"));
        assert_eq!(binop_to_runtime(HirBinOp::Eq), Some("mb_eq"));
        assert_eq!(binop_to_runtime(HirBinOp::BitXor), Some("mb_bitxor"));
    }

    // REQ: tick-144 test-coverage — lower_mir_unaryop round-trips all 4 HirUnaryOp variants
    // (Pos/Neg/Not/BitNot) to matching MirUnaryOp — pure exhaustive mapping.
    #[test]
    fn test_lower_mir_unaryop_exhaustive_four_variants() {
        assert!(matches!(
            lower_mir_unaryop(HirUnaryOp::Pos),
            MirUnaryOp::Pos
        ));
        assert!(matches!(
            lower_mir_unaryop(HirUnaryOp::Neg),
            MirUnaryOp::Neg
        ));
        assert!(matches!(
            lower_mir_unaryop(HirUnaryOp::Not),
            MirUnaryOp::Not
        ));
        assert!(matches!(
            lower_mir_unaryop(HirUnaryOp::BitNot),
            MirUnaryOp::BitNot
        ));
    }

    // REQ: tick-148 test-coverage — lower_mir_binop preserves operator kind across 6 op groups
    // (arith, compare, logical, bitwise, shift, identity/membership) via 8 representative variants.
    #[test]
    fn test_lower_mir_binop_representative_across_six_op_groups() {
        assert!(matches!(lower_mir_binop(HirBinOp::Add), MirBinOp::Add));
        assert!(matches!(
            lower_mir_binop(HirBinOp::FloorDiv),
            MirBinOp::FloorDiv
        ));
        assert!(matches!(lower_mir_binop(HirBinOp::Eq), MirBinOp::Eq));
        assert!(matches!(lower_mir_binop(HirBinOp::Lt), MirBinOp::Lt));
        assert!(matches!(lower_mir_binop(HirBinOp::And), MirBinOp::And));
        assert!(matches!(
            lower_mir_binop(HirBinOp::BitXor),
            MirBinOp::BitXor
        ));
        assert!(matches!(
            lower_mir_binop(HirBinOp::RShift),
            MirBinOp::RShift
        ));
        assert!(matches!(lower_mir_binop(HirBinOp::IsNot), MirBinOp::IsNot));
    }
}
