use super::super::rc::MbObject;
use super::super::value::MbValue;
/// Dev-tools stdlib modules for Mamba (#1261 long-tail).
///
/// Bundles surface-only shims for the introspection / profiling /
/// debugging modules that real-world libraries probe at import time but
/// Mamba doesn't yet implement natively: pyclbr, symtable, modulefinder,
/// runpy, pkgutil, timeit, trace, pstats, profile, cProfile, doctest,
/// pdb, tabnanny, py_compile.
///
/// Each module gets a callable-shell surface so `from doctest import
/// testmod` resolves and returns a sentinel rather than crashing. The
/// underlying functionality (running tests, profiling, debugger entry)
/// is not yet hosted on Mamba — these are import-resolution stubs.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_class_shell(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
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
    register_profile();
    register_cprofile();
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
    let shell = dispatch_class_shell as *const () as usize;
    let dict_ret = dispatch_class_shell as *const () as usize;
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("readmodule", dict_ret),
        ("readmodule_ex", dict_ret),
        ("Class", shell),
        ("Function", shell),
    ];
    for (n, a) in dispatchers {
        attrs.insert((*n).into(), MbValue::from_func(*a));
    }
    add_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    super::register_module("pyclbr", attrs);
}

fn register_symtable() {
    let shell = dispatch_class_shell as *const () as usize;
    let int_zero = dispatch_int_zero as *const () as usize;
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("symtable", shell),
        ("SymbolTable", shell),
        ("Function", shell),
        ("Class", shell),
        ("Symbol", shell),
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
    add_addrs(&[shell, int_zero]);
    super::register_module("symtable", attrs);
}

fn register_modulefinder() {
    let shell = dispatch_class_shell as *const () as usize;
    let mut attrs = HashMap::new();
    attrs.insert("ModuleFinder".into(), MbValue::from_func(shell));
    attrs.insert("Module".into(), MbValue::from_func(shell));
    attrs.insert(
        "AddPackagePath".into(),
        MbValue::from_func(dispatch_noop as *const () as usize),
    );
    attrs.insert(
        "ReplacePackage".into(),
        MbValue::from_func(dispatch_noop as *const () as usize),
    );
    add_addrs(&[shell, dispatch_noop as *const () as usize]);
    super::register_module("modulefinder", attrs);
}

fn register_runpy() {
    let shell = dispatch_class_shell as *const () as usize;
    let mut attrs = HashMap::new();
    attrs.insert("run_module".into(), MbValue::from_func(shell));
    attrs.insert("run_path".into(), MbValue::from_func(shell));
    attrs.insert("_run_code".into(), MbValue::from_func(shell));
    attrs.insert("_run_module_code".into(), MbValue::from_func(shell));
    add_addrs(&[shell]);
    super::register_module("runpy", attrs);
}

fn register_pkgutil() {
    let shell = dispatch_class_shell as *const () as usize;
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
        ("ImpImporter", shell),
        ("ImpLoader", shell),
        ("ModuleInfo", shell),
    ];
    for (n, a) in dispatchers {
        attrs.insert((*n).into(), MbValue::from_func(*a));
    }
    add_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    super::register_module("pkgutil", attrs);
}

fn register_timeit() {
    let shell = dispatch_class_shell as *const () as usize;
    let int_zero = dispatch_int_zero as *const () as usize;
    let empty_list = dispatch_empty_list as *const () as usize;
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("timeit", int_zero),
        ("repeat", empty_list),
        ("Timer", shell),
        ("default_timer", int_zero),
        ("default_number", int_zero),
    ];
    for (n, a) in dispatchers {
        attrs.insert((*n).into(), MbValue::from_func(*a));
    }
    add_addrs(&dispatchers.iter().map(|(_, a)| *a).collect::<Vec<_>>());
    super::register_module("timeit", attrs);
}

fn register_trace() {
    let shell = dispatch_class_shell as *const () as usize;
    let mut attrs = HashMap::new();
    attrs.insert("Trace".into(), MbValue::from_func(shell));
    attrs.insert("CoverageResults".into(), MbValue::from_func(shell));
    add_addrs(&[shell]);
    super::register_module("trace", attrs);
}

fn register_pstats() {
    let shell = dispatch_class_shell as *const () as usize;
    let mut attrs = HashMap::new();
    attrs.insert("Stats".into(), MbValue::from_func(shell));
    attrs.insert("SortKey".into(), MbValue::from_func(shell));
    add_addrs(&[shell]);
    super::register_module("pstats", attrs);
}

fn register_profile() {
    let shell = dispatch_class_shell as *const () as usize;
    let mut attrs = HashMap::new();
    attrs.insert("Profile".into(), MbValue::from_func(shell));
    attrs.insert(
        "run".into(),
        MbValue::from_func(dispatch_noop as *const () as usize),
    );
    attrs.insert(
        "runctx".into(),
        MbValue::from_func(dispatch_noop as *const () as usize),
    );
    add_addrs(&[shell, dispatch_noop as *const () as usize]);
    super::register_module("profile", attrs);
}

fn register_cprofile() {
    let shell = dispatch_class_shell as *const () as usize;
    let mut attrs = HashMap::new();
    attrs.insert("Profile".into(), MbValue::from_func(shell));
    attrs.insert(
        "run".into(),
        MbValue::from_func(dispatch_noop as *const () as usize),
    );
    attrs.insert(
        "runctx".into(),
        MbValue::from_func(dispatch_noop as *const () as usize),
    );
    add_addrs(&[shell, dispatch_noop as *const () as usize]);
    super::register_module("cProfile", attrs);
}

fn register_doctest() {
    let shell = dispatch_class_shell as *const () as usize;
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
        ("DocTestFinder", shell),
        ("DocTestParser", shell),
        ("DocTestRunner", shell),
        ("DebugRunner", shell),
        ("OutputChecker", shell),
        ("Example", shell),
        ("DocTest", shell),
        ("DocTestCase", shell),
        ("DocFileCase", shell),
        ("DocTestSuite", shell),
        ("DocFileSuite", shell),
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
        ("debug", dispatch_noop as *const () as usize),
        ("debug_script", dispatch_noop as *const () as usize),
        ("debug_src", dispatch_noop as *const () as usize),
        ("master", dispatch_noop as *const () as usize),
        ("Tester", shell),
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
    add_addrs(&[
        shell,
        test_results,
        empty_list,
        dispatch_noop as *const () as usize,
        dispatch_int_zero as *const () as usize,
        dispatch_empty_str as *const () as usize,
    ]);
    super::register_module("doctest", attrs);
}

fn register_pdb() {
    let shell = dispatch_class_shell as *const () as usize;
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("set_trace", dispatch_noop as *const () as usize),
        ("post_mortem", dispatch_noop as *const () as usize),
        ("pm", dispatch_noop as *const () as usize),
        ("run", dispatch_noop as *const () as usize),
        ("runeval", dispatch_noop as *const () as usize),
        ("runctx", dispatch_noop as *const () as usize),
        ("runcall", dispatch_noop as *const () as usize),
        ("Pdb", shell),
        ("Restart", shell),
    ];
    for (n, a) in dispatchers {
        attrs.insert((*n).into(), MbValue::from_func(*a));
    }
    add_addrs(&[shell, dispatch_noop as *const () as usize]);
    super::register_module("pdb", attrs);
}

fn register_tabnanny() {
    let shell = dispatch_class_shell as *const () as usize;
    let mut attrs = HashMap::new();
    attrs.insert(
        "check".into(),
        MbValue::from_func(dispatch_noop as *const () as usize),
    );
    attrs.insert(
        "process_tokens".into(),
        MbValue::from_func(dispatch_noop as *const () as usize),
    );
    attrs.insert("NannyNag".into(), MbValue::from_func(shell));
    attrs.insert("verbose".into(), MbValue::from_int(0));
    attrs.insert("filename_only".into(), MbValue::from_int(0));
    add_addrs(&[shell, dispatch_noop as *const () as usize]);
    super::register_module("tabnanny", attrs);
}

fn register_py_compile() {
    let shell = dispatch_class_shell as *const () as usize;
    let empty_str = dispatch_empty_str as *const () as usize;
    let mut attrs = HashMap::new();
    attrs.insert("compile".into(), MbValue::from_func(empty_str));
    attrs.insert(
        "main".into(),
        MbValue::from_func(dispatch_noop as *const () as usize),
    );
    attrs.insert("PyCompileError".into(), MbValue::from_func(shell));
    attrs.insert("PycInvalidationMode".into(), MbValue::from_func(shell));
    // PycInvalidationMode constants used by py_compile callers.
    attrs.insert("CHECKED_HASH".into(), MbValue::from_int(2));
    attrs.insert("UNCHECKED_HASH".into(), MbValue::from_int(3));
    attrs.insert("TIMESTAMP".into(), MbValue::from_int(1));
    add_addrs(&[shell, empty_str, dispatch_noop as *const () as usize]);
    super::register_module("py_compile", attrs);
}
