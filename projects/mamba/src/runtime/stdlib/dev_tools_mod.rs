use super::super::rc::MbObject;
use super::super::value::MbValue;
/// Dev-tools stdlib modules for Mamba (#1261 long-tail).
///
/// Bundles surface-only shims for the introspection / profiling /
/// debugging modules that real-world libraries probe at import time but
/// Mamba doesn't yet implement natively: pyclbr, symtable, modulefinder,
/// runpy, pkgutil, timeit, trace, pstats, doctest, pdb, tabnanny,
/// py_compile. (`profile` / `cProfile` / `_lsprof` moved to a real
/// deterministic-profiler backend in `cprofile_mod` — see #878.)
///
/// Each module gets a callable-shell surface so `from doctest import
/// testmod` resolves and returns a sentinel rather than crashing. The
/// underlying functionality (running tests, profiling, debugger entry)
/// is not yet hosted on Mamba — these are import-resolution stubs.
use std::collections::HashMap;

// #1040 follow-up: this file's `dispatch_class_shell` used to be handed out
// as the SAME function address to every class-shell name registered here,
// across every `register_*` call in this file. Because FUNC_NAMES/
// NATIVE_FUNC_ADDRS are address-keyed, whichever name registered last (in
// HashMap iteration order, which is nondeterministic per process) won
// `X.__name__` for every other class sharing that address -- the same
// #962/#954 symptom (e.g. `pyclbr.Class` and `symtable.Symbol` colliding).
// The fix: give every class-shell name a genuinely distinct function
// pointer, drawn from a pool of `SHELL_POOL_SIZE` individually fold-immune
// trivial stub functions, indexed via a thread-local "next free slot"
// counter (`next_shell_slot`) so every call site simply draws a fresh slot
// per name -- no manual per-call `pool_start` bookkeeping required, since
// `register()` runs registration sequentially on a single thread at
// module-init time.
//
// IMPORTANT: this pool does NOT use `icf_guard!()` directly. That macro
// derives its fingerprint from `module_path!()`/`line!()`/`column!()`, which
// are resolved at the span of the *macro definition's* literal tokens -- for
// a single `macro_rules!` invocation that expands a `$(...)* ` repetition
// into N functions, every repetition shares that ONE span, so
// `line!()`/`column!()` come back IDENTICAL for all N and `icf_guard!()`
// silently fails to discriminate them. LLVM then folds all "distinct"
// shells back onto a single address, reproducing the exact bug one level
// down. The fix here instead fingerprints on `stringify!($name)`, which DOES
// vary per repetition (driven by the captured `$name` token's text, not by
// span), giving every pool slot a genuinely distinct compiled body.
const SHELL_POOL_SIZE: usize = 48;
type ShellFn = unsafe extern "C" fn(*const MbValue, usize) -> MbValue;

macro_rules! def_shell_pool {
    ($($name:ident),* $(,)?) => {
        $(
            unsafe extern "C" fn $name(_a: *const MbValue, _n: usize) -> MbValue {
                ::std::hint::black_box(crate::runtime::module::icf_fingerprint(concat!(
                    module_path!(),
                    "::",
                    stringify!($name)
                )));
                MbValue::from_ptr(MbObject::new_dict())
            }
        )*
        const SHELL_POOL: [ShellFn; SHELL_POOL_SIZE] = [$($name),*];
    };
}
def_shell_pool!(
    shell_00, shell_01, shell_02, shell_03, shell_04, shell_05, shell_06, shell_07, shell_08,
    shell_09, shell_10, shell_11, shell_12, shell_13, shell_14, shell_15, shell_16, shell_17,
    shell_18, shell_19, shell_20, shell_21, shell_22, shell_23, shell_24, shell_25, shell_26,
    shell_27, shell_28, shell_29, shell_30, shell_31, shell_32, shell_33, shell_34, shell_35,
    shell_36, shell_37, shell_38, shell_39, shell_40, shell_41, shell_42, shell_43, shell_44,
    shell_45, shell_46, shell_47,
);

/// Pool slot at `idx` as a raw function-pointer address.
fn shell_addr(idx: usize) -> usize {
    SHELL_POOL[idx] as usize
}

thread_local! {
    static NEXT_SHELL_SLOT: std::cell::Cell<usize> = std::cell::Cell::new(0);
}

/// Draw the next unused pool slot index. `register()` runs sequentially on
/// a single thread at module-init time, so a simple monotonic counter gives
/// every class-shell name a fresh, non-overlapping slot with no manual
/// per-call range bookkeeping.
fn next_shell_slot() -> usize {
    NEXT_SHELL_SLOT.with(|c| {
        let v = c.get();
        assert!(
            v < SHELL_POOL_SIZE,
            "shell pool exhausted (SHELL_POOL_SIZE={}); bump it",
            SHELL_POOL_SIZE
        );
        c.set(v + 1);
        v
    })
}

unsafe extern "C" fn dispatch_empty_list(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

unsafe extern "C" fn dispatch_empty_str(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

unsafe extern "C" fn dispatch_int_zero(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_int(0)
}

unsafe extern "C" fn dispatch_noop(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn dispatch_test_results(_a: *const MbValue, _n: usize) -> MbValue {
    // doctest.testmod returns a TestResults(attempted, failed) named tuple;
    // shim returns a 2-element list [0, 0].
    MbValue::from_ptr(MbObject::new_list(vec![
        MbValue::from_int(0),
        MbValue::from_int(0),
    ]))
}

pub fn register() {
    register_pyclbr();
    register_symtable();
    register_modulefinder();
    register_runpy();
    register_pkgutil();
    register_timeit();
    register_trace();
    register_pstats();
    // profile / cProfile / _lsprof: real deterministic profiler backend
    // now lives in cprofile_mod (#878) — no longer stubs here.
    register_doctest();
    register_pdb();
    register_tabnanny();
    register_py_compile();
}

fn add_addrs(addrs: &[usize]) {
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for a in addrs {
            set.insert(*a as u64);
        }
    });
}

fn register_pyclbr() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("readmodule", shell_addr(next_shell_slot())),
        ("readmodule_ex", shell_addr(next_shell_slot())),
        ("Class", shell_addr(next_shell_slot())),
        ("Function", shell_addr(next_shell_slot())),
    ];
    for (n, a) in dispatchers {
        attrs.insert((*n).into(), MbValue::from_func(*a));
    }
    add_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    super::register_module("pyclbr", attrs);
}

fn register_symtable() {
    let int_zero = dispatch_int_zero as *const () as usize;
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("symtable", shell_addr(next_shell_slot())),
        ("SymbolTable", shell_addr(next_shell_slot())),
        ("Function", shell_addr(next_shell_slot())),
        ("Class", shell_addr(next_shell_slot())),
        ("Symbol", shell_addr(next_shell_slot())),
    ];
    for (n, a) in dispatchers {
        attrs.insert((*n).into(), MbValue::from_func(*a));
    }
    // Type / scope constants used in symtable internals.
    for (name, value) in &[
        ("USE", 0x10),
        ("DEF_GLOBAL", 0x01),
        ("DEF_LOCAL", 0x02),
        ("DEF_PARAM", 0x04),
        ("DEF_NONLOCAL", 0x08),
        ("DEF_FREE", 0x80),
        ("DEF_IMPORT", 0x40),
        ("DEF_BOUND", 0x07),
        ("LOCAL", 1),
        ("GLOBAL_EXPLICIT", 2),
        ("GLOBAL_IMPLICIT", 3),
        ("FREE", 4),
        ("CELL", 5),
    ] {
        attrs.insert((*name).into(), MbValue::from_int(*value));
    }
    add_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    add_addrs(&[int_zero]);
    // surface: missing CPython module constants (auto-added)
    attrs.insert("DEF_ANNOT".into(), MbValue::from_int(256));
    attrs.insert("SCOPE_MASK".into(), MbValue::from_int(15));
    attrs.insert("SCOPE_OFF".into(), MbValue::from_int(12));
    super::register_module("symtable", attrs);
}

fn register_modulefinder() {
    let mut attrs = HashMap::new();
    let module_finder_addr = shell_addr(next_shell_slot());
    let module_addr = shell_addr(next_shell_slot());
    attrs.insert(
        "ModuleFinder".into(),
        MbValue::from_func(module_finder_addr),
    );
    attrs.insert("Module".into(), MbValue::from_func(module_addr));
    attrs.insert(
        "AddPackagePath".into(),
        MbValue::from_func(dispatch_noop as *const () as usize),
    );
    attrs.insert(
        "ReplacePackage".into(),
        MbValue::from_func(dispatch_noop as *const () as usize),
    );
    add_addrs(&[
        module_finder_addr,
        module_addr,
        dispatch_noop as *const () as usize,
    ]);
    super::register_module("modulefinder", attrs);
}

fn register_runpy() {
    let mut attrs = HashMap::new();
    let run_module_addr = shell_addr(next_shell_slot());
    let run_path_addr = shell_addr(next_shell_slot());
    let run_code_addr = shell_addr(next_shell_slot());
    let run_module_code_addr = shell_addr(next_shell_slot());
    attrs.insert("run_module".into(), MbValue::from_func(run_module_addr));
    attrs.insert("run_path".into(), MbValue::from_func(run_path_addr));
    attrs.insert("_run_code".into(), MbValue::from_func(run_code_addr));
    attrs.insert(
        "_run_module_code".into(),
        MbValue::from_func(run_module_code_addr),
    );
    add_addrs(&[
        run_module_addr,
        run_path_addr,
        run_code_addr,
        run_module_code_addr,
    ]);
    super::register_module("runpy", attrs);
}

fn register_pkgutil() {
    let empty_list = dispatch_empty_list as *const () as usize;
    let empty_str = dispatch_empty_str as *const () as usize;
    let noop = dispatch_noop as *const () as usize;
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("get_data", empty_str),
        ("iter_modules", empty_list),
        ("walk_packages", empty_list),
        ("find_loader", noop),
        ("get_importer", noop),
        ("get_loader", noop),
        ("extend_path", empty_list),
        ("resolve_name", noop),
        ("ImpImporter", shell_addr(next_shell_slot())),
        ("ImpLoader", shell_addr(next_shell_slot())),
        ("ModuleInfo", shell_addr(next_shell_slot())),
    ];
    for (n, a) in dispatchers {
        attrs.insert((*n).into(), MbValue::from_func(*a));
    }
    add_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    super::register_module("pkgutil", attrs);
}

fn register_timeit() {
    let int_zero = dispatch_int_zero as *const () as usize;
    let empty_list = dispatch_empty_list as *const () as usize;
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("timeit", int_zero),
        ("repeat", empty_list),
        ("Timer", shell_addr(next_shell_slot())),
        ("default_timer", int_zero),
        ("default_number", int_zero),
    ];
    for (n, a) in dispatchers {
        attrs.insert((*n).into(), MbValue::from_func(*a));
    }
    add_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    // surface: missing CPython module constants (auto-added)
    attrs.insert("default_repeat".into(), MbValue::from_int(5));
    attrs.insert(
        "dummy_src_name".into(),
        MbValue::from_ptr(MbObject::new_str("<timeit-src>".to_string())),
    );
    attrs.insert("template".into(), MbValue::from_ptr(MbObject::new_str("\ndef inner(_it, _timer{init}):\n    {setup}\n    _t0 = _timer()\n    for _i in _it:\n        {stmt}\n        pass\n    _t1 = _timer()\n    return _t1 - _t0\n".to_string())));
    super::register_module("timeit", attrs);
}

fn register_trace() {
    let mut attrs = HashMap::new();
    let trace_addr = shell_addr(next_shell_slot());
    let coverage_addr = shell_addr(next_shell_slot());
    attrs.insert("Trace".into(), MbValue::from_func(trace_addr));
    attrs.insert("CoverageResults".into(), MbValue::from_func(coverage_addr));
    add_addrs(&[trace_addr, coverage_addr]);
    // surface: missing CPython module constants (auto-added)
    attrs.insert(
        "PRAGMA_NOCOVER".into(),
        MbValue::from_ptr(MbObject::new_str("#pragma NO COVER".to_string())),
    );
    super::register_module("trace", attrs);
}

fn register_pstats() {
    let mut attrs = HashMap::new();
    let stats_addr = shell_addr(next_shell_slot());
    let sortkey_addr = shell_addr(next_shell_slot());
    let func_profile_addr = shell_addr(next_shell_slot());
    let stats_profile_addr = shell_addr(next_shell_slot());
    attrs.insert("Stats".into(), MbValue::from_func(stats_addr));
    attrs.insert("SortKey".into(), MbValue::from_func(sortkey_addr));
    // surface: missing CPython classes (auto-added)
    attrs.insert(
        "FunctionProfile".into(),
        MbValue::from_func(func_profile_addr),
    );
    attrs.insert(
        "StatsProfile".into(),
        MbValue::from_func(stats_profile_addr),
    );
    add_addrs(&[
        stats_addr,
        sortkey_addr,
        func_profile_addr,
        stats_profile_addr,
    ]);
    super::register_module("pstats", attrs);
}

fn register_doctest() {
    let test_results = dispatch_test_results as *const () as usize;
    let empty_list = dispatch_empty_list as *const () as usize;
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("testmod", test_results),
        ("testfile", test_results),
        (
            "run_docstring_examples",
            dispatch_noop as *const () as usize,
        ),
        ("DocTestFinder", shell_addr(next_shell_slot())),
        ("DocTestParser", shell_addr(next_shell_slot())),
        ("DocTestRunner", shell_addr(next_shell_slot())),
        ("DebugRunner", shell_addr(next_shell_slot())),
        ("OutputChecker", shell_addr(next_shell_slot())),
        ("Example", shell_addr(next_shell_slot())),
        ("DocTest", shell_addr(next_shell_slot())),
        ("DocTestCase", shell_addr(next_shell_slot())),
        ("DocFileCase", shell_addr(next_shell_slot())),
        ("DocTestSuite", shell_addr(next_shell_slot())),
        ("DocFileSuite", shell_addr(next_shell_slot())),
        ("DocTestFailure", shell_addr(next_shell_slot())),
        ("UnexpectedException", shell_addr(next_shell_slot())),
        (
            "set_unittest_reportflags",
            dispatch_noop as *const () as usize,
        ),
        (
            "register_optionflag",
            dispatch_int_zero as *const () as usize,
        ),
        (
            "script_from_examples",
            dispatch_empty_str as *const () as usize,
        ),
        ("testsource", dispatch_empty_str as *const () as usize),
        ("debug", dispatch_noop as *const () as usize),
        ("debug_script", dispatch_noop as *const () as usize),
        ("debug_src", dispatch_noop as *const () as usize),
        ("master", dispatch_noop as *const () as usize),
        ("Tester", shell_addr(next_shell_slot())),
        ("REPORTING_FLAGS", dispatch_int_zero as *const () as usize),
        ("COMPARISON_FLAGS", dispatch_int_zero as *const () as usize),
    ];
    for (n, a) in dispatchers {
        attrs.insert((*n).into(), MbValue::from_func(*a));
    }
    // Option flags used by doctest decorators / runners.
    for (name, value) in &[
        ("DONT_ACCEPT_TRUE_FOR_1", 1),
        ("DONT_ACCEPT_BLANKLINE", 2),
        ("NORMALIZE_WHITESPACE", 4),
        ("ELLIPSIS", 8),
        ("SKIP", 16),
        ("IGNORE_EXCEPTION_DETAIL", 32),
        ("FAIL_FAST", 1024),
        ("REPORT_UDIFF", 64),
        ("REPORT_CDIFF", 128),
        ("REPORT_NDIFF", 256),
        ("REPORT_ONLY_FIRST_FAILURE", 512),
    ] {
        attrs.insert((*name).into(), MbValue::from_int(*value));
    }
    attrs.insert(
        "__file__".into(),
        MbValue::from_ptr(MbObject::new_str("<doctest>".to_string())),
    );
    attrs.insert("_unittest_reportflags".into(), MbValue::from_int(0));
    add_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    add_addrs(&[empty_list]);
    // surface: missing CPython module constants (auto-added)
    attrs.insert(
        "BLANKLINE_MARKER".into(),
        MbValue::from_ptr(MbObject::new_str("<BLANKLINE>".to_string())),
    );
    attrs.insert(
        "ELLIPSIS_MARKER".into(),
        MbValue::from_ptr(MbObject::new_str("...".to_string())),
    );
    super::register_module("doctest", attrs);
}

fn register_pdb() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("set_trace", dispatch_noop as *const () as usize),
        ("post_mortem", dispatch_noop as *const () as usize),
        ("pm", dispatch_noop as *const () as usize),
        ("run", dispatch_noop as *const () as usize),
        ("runeval", dispatch_noop as *const () as usize),
        ("runctx", dispatch_noop as *const () as usize),
        ("runcall", dispatch_noop as *const () as usize),
        ("help", dispatch_noop as *const () as usize),
        ("Pdb", shell_addr(next_shell_slot())),
        ("Restart", shell_addr(next_shell_slot())),
    ];
    for (n, a) in dispatchers {
        attrs.insert((*n).into(), MbValue::from_func(*a));
    }
    add_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    // surface: missing CPython module constants (auto-added)
    attrs.insert(
        "TESTCMD".into(),
        MbValue::from_ptr(MbObject::new_str("import x; x.main()".to_string())),
    );
    attrs.insert(
        "line_prefix".into(),
        MbValue::from_ptr(MbObject::new_str("\n-> ".to_string())),
    );
    super::register_module("pdb", attrs);
}

fn register_tabnanny() {
    let mut attrs = HashMap::new();
    attrs.insert(
        "check".into(),
        MbValue::from_func(dispatch_noop as *const () as usize),
    );
    attrs.insert(
        "process_tokens".into(),
        MbValue::from_func(dispatch_noop as *const () as usize),
    );
    let nannynag_addr = shell_addr(next_shell_slot());
    attrs.insert("NannyNag".into(), MbValue::from_func(nannynag_addr));
    attrs.insert("verbose".into(), MbValue::from_int(0));
    attrs.insert("filename_only".into(), MbValue::from_int(0));
    add_addrs(&[nannynag_addr, dispatch_noop as *const () as usize]);
    super::register_module("tabnanny", attrs);
}

fn register_py_compile() {
    let empty_str = dispatch_empty_str as *const () as usize;
    let mut attrs = HashMap::new();
    attrs.insert("compile".into(), MbValue::from_func(empty_str));
    attrs.insert(
        "main".into(),
        MbValue::from_func(dispatch_noop as *const () as usize),
    );
    let py_compile_error_addr = shell_addr(next_shell_slot());
    let pyc_invalidation_mode_addr = shell_addr(next_shell_slot());
    attrs.insert(
        "PyCompileError".into(),
        MbValue::from_func(py_compile_error_addr),
    );
    attrs.insert(
        "PycInvalidationMode".into(),
        MbValue::from_func(pyc_invalidation_mode_addr),
    );
    // PycInvalidationMode constants used by py_compile callers.
    attrs.insert("CHECKED_HASH".into(), MbValue::from_int(2));
    attrs.insert("UNCHECKED_HASH".into(), MbValue::from_int(3));
    attrs.insert("TIMESTAMP".into(), MbValue::from_int(1));
    add_addrs(&[
        py_compile_error_addr,
        pyc_invalidation_mode_addr,
        empty_str,
        dispatch_noop as *const () as usize,
    ]);
    super::register_module("py_compile", attrs);
}
