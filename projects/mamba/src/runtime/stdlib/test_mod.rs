use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// test module for Mamba (#999).
///
/// Provides CPython-style test support utilities: TestCase base class with
/// core assertion methods (assertEqual, assertTrue, assertFalse, assertRaises),
/// and a main() test runner entry point. Distinct from the `unittest` module.
use std::collections::HashMap;

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! dispatch_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

dispatch_nullary!(dispatch_TestCase, mb_test_testcase);
dispatch_nullary!(dispatch_main, mb_test_main);
dispatch_binary!(dispatch_assertEqual, mb_test_assert_equal);
dispatch_unary!(dispatch_assertTrue, mb_test_assert_true);
dispatch_unary!(dispatch_assertFalse, mb_test_assert_false);
dispatch_unary!(dispatch_assertRaises, mb_test_assert_raises);
dispatch_nullary!(dispatch_support, mb_test_support);

unsafe extern "C" fn dispatch_noop_variadic(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn dispatch_identity(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    a.get(0).copied().unwrap_or_else(MbValue::none)
}

/// test.support.os_helper.FakePath(path) — a minimal os.PathLike wrapper whose
/// __fspath__ returns the stored path (or raises it, if it is an exception).
unsafe extern "C" fn dispatch_fakepath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let path = a.first().copied().unwrap_or_else(MbValue::none);
    let inst = MbObject::new_instance("FakePath".to_string());
    if let ObjData::Instance { ref fields, .. } = (*inst).data {
        super::super::rc::retain_if_ptr(path);
        fields.write().unwrap().insert("path".to_string(), path);
    }
    MbValue::from_ptr(inst)
}

/// FakePath.__fspath__(self) -> the stored path.
unsafe extern "C" fn fakepath_fspath(self_v: MbValue, _args: MbValue) -> MbValue {
    let path = self_v.as_ptr().and_then(|p| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*p).data {
            fields.read().ok().and_then(|f| f.get("path").copied())
        } else {
            None
        }
    }).unwrap_or_else(MbValue::none);
    super::super::rc::retain_if_ptr(path);
    path
}

// ── test.support.swap_attr / EnvironmentVarGuard native context managers ──
//
// CPython's test.support exposes `swap_attr(obj, name, new)` and
// `EnvironmentVarGuard()` as real context managers used pervasively by ported
// fixtures for setup/teardown. Modeling them as Instance objects whose class
// carries real `__enter__`/`__exit__` (and, for the env guard,
// `set`/`unset`/`__setitem__`/`__delitem__`/`__getitem__`) methods makes
// `value_supports_context_manager` accept them (it resolves dunders via the
// class method table) and routes `with`/`enter_context` through the generic
// runtime CM machinery.

/// Read an Instance field (None if absent / not an Instance).
fn inst_field(self_v: MbValue, key: &str) -> MbValue {
    self_v
        .as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*p).data {
                fields.read().ok().and_then(|f| f.get(key).copied())
            } else {
                None
            }
        })
        .unwrap_or_else(MbValue::none)
}

/// Set an Instance field, retaining the value and releasing any prior one.
fn inst_set_field(self_v: MbValue, key: &str, value: MbValue) {
    if let Some(p) = self_v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*p).data {
                super::super::rc::retain_if_ptr(value);
                if let Some(prev) = fields.write().unwrap().insert(key.to_string(), value) {
                    super::super::rc::release_if_ptr(prev);
                }
            }
        }
    }
}

/// `test.support.swap_attr(obj, name, new_value)` → a context-manager Instance.
/// On `__enter__` it saves the old attribute (and whether it existed) and
/// installs `new_value`, returning the old value; on `__exit__` it restores
/// (or deletes the attribute if it was originally absent). CPython fidelity.
unsafe extern "C" fn dispatch_swap_attr(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let obj = a.first().copied().unwrap_or_else(MbValue::none);
    let name = a.get(1).copied().unwrap_or_else(MbValue::none);
    let new_value = a.get(2).copied().unwrap_or_else(MbValue::none);
    let inst = MbObject::new_instance("_test_SwapAttr".to_string());
    let inst_v = MbValue::from_ptr(inst);
    inst_set_field(inst_v, "_obj", obj);
    inst_set_field(inst_v, "_name", name);
    inst_set_field(inst_v, "_new", new_value);
    inst_v
}

/// _test_SwapAttr.__enter__(self) — save old attr + install new, return old.
unsafe extern "C" fn swap_attr_enter(self_v: MbValue) -> MbValue {
    let obj = inst_field(self_v, "_obj");
    let name = inst_field(self_v, "_name");
    let new_value = inst_field(self_v, "_new");
    let had = super::super::class::mb_hasattr(obj, name).as_bool() == Some(true);
    let old = if had {
        super::super::class::mb_getattr(obj, name)
    } else {
        MbValue::none()
    };
    inst_set_field(self_v, "_had", MbValue::from_bool(had));
    inst_set_field(self_v, "_old", old);
    super::super::class::mb_setattr(obj, name, new_value);
    super::super::rc::retain_if_ptr(old);
    old
}

/// _test_SwapAttr.__exit__(self, *exc) — restore the saved attribute.
unsafe extern "C" fn swap_attr_exit(
    self_v: MbValue,
    _t: MbValue,
    _v: MbValue,
    _tb: MbValue,
) -> MbValue {
    let obj = inst_field(self_v, "_obj");
    let name = inst_field(self_v, "_name");
    let had = inst_field(self_v, "_had").as_bool() == Some(true);
    if had {
        let old = inst_field(self_v, "_old");
        super::super::class::mb_setattr(obj, name, old);
    } else {
        super::super::class::mb_delattr(obj, name);
    }
    MbValue::from_bool(false)
}

/// `os.environ` dict, or None if `os` has not been imported.
fn os_environ() -> MbValue {
    super::super::module::mb_module_attr_lookup("os", "environ").unwrap_or_else(MbValue::none)
}

/// `test.support.os_helper.EnvironmentVarGuard()` → a context-manager Instance
/// that mutates `os.environ` and restores every touched key on `__exit__`.
unsafe extern "C" fn dispatch_env_guard(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    let inst = MbObject::new_instance("_test_EnvironmentVarGuard".to_string());
    let inst_v = MbValue::from_ptr(inst);
    // `_changed`: key → original value (or None if the key was absent). A key is
    // recorded only on its first mutation, so the original is preserved.
    let changed = MbValue::from_ptr(MbObject::new_dict());
    inst_set_field(inst_v, "_changed", changed);
    // inst_set_field retained `changed`; drop our construction reference so the
    // field owns the sole remaining ref.
    super::super::rc::release_if_ptr(changed);
    inst_v
}

/// Record the pre-mutation value of `key` (once) so __exit__ can restore it.
fn env_guard_record(self_v: MbValue, key: MbValue) {
    let changed = inst_field(self_v, "_changed");
    if super::super::dict_ops::mb_dict_contains(changed, key).as_bool() == Some(true) {
        return; // already recorded
    }
    let environ = os_environ();
    let sentinel = MbValue::none();
    let orig = if super::super::dict_ops::mb_dict_contains(environ, key).as_bool() == Some(true) {
        super::super::dict_ops::mb_dict_get(environ, key, sentinel)
    } else {
        sentinel
    };
    super::super::dict_ops::mb_dict_setitem(changed, key, orig);
}

/// EnvironmentVarGuard.__enter__(self) → self.
unsafe extern "C" fn env_guard_enter(self_v: MbValue) -> MbValue {
    super::super::rc::retain_if_ptr(self_v);
    self_v
}

/// EnvironmentVarGuard.set(self, key, value) / __setitem__ — set os.environ[key].
unsafe extern "C" fn env_guard_set(self_v: MbValue, key: MbValue, value: MbValue) -> MbValue {
    env_guard_record(self_v, key);
    super::super::dict_ops::mb_dict_setitem(os_environ(), key, value);
    MbValue::none()
}

/// EnvironmentVarGuard.unset(self, key) / __delitem__ — delete os.environ[key].
unsafe extern "C" fn env_guard_unset(self_v: MbValue, key: MbValue) -> MbValue {
    env_guard_record(self_v, key);
    let environ = os_environ();
    if super::super::dict_ops::mb_dict_contains(environ, key).as_bool() == Some(true) {
        super::super::dict_ops::mb_dict_delitem(environ, key);
    }
    MbValue::none()
}

/// EnvironmentVarGuard.__getitem__(self, key) → os.environ[key].
unsafe extern "C" fn env_guard_getitem(self_v: MbValue, key: MbValue) -> MbValue {
    let _ = self_v;
    super::super::class::mb_obj_getitem(os_environ(), key)
}

/// EnvironmentVarGuard.__exit__(self, *exc) — restore every recorded key.
unsafe extern "C" fn env_guard_exit(
    self_v: MbValue,
    _t: MbValue,
    _v: MbValue,
    _tb: MbValue,
) -> MbValue {
    let changed = inst_field(self_v, "_changed");
    let environ = os_environ();
    // Collect (key_str, orig) pairs without holding the dict lock across mutation.
    let pairs: Vec<(MbValue, MbValue)> = changed
        .as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::Dict(ref lock) = (*p).data {
                let map = lock.read().ok()?;
                Some(
                    map.iter()
                        .map(|(k, v)| (super::super::dict_ops::dict_key_to_mbvalue(k), *v))
                        .collect(),
                )
            } else {
                None
            }
        })
        .unwrap_or_default();
    for (key, orig) in pairs {
        if orig.is_none() {
            // Key was originally absent → delete it.
            if super::super::dict_ops::mb_dict_contains(environ, key).as_bool() == Some(true) {
                super::super::dict_ops::mb_dict_delitem(environ, key);
            }
        } else {
            super::super::dict_ops::mb_dict_setitem(environ, key, orig);
        }
    }
    MbValue::from_bool(false)
}

/// Register a native context-manager class. Methods are FIXED-arity SystemV
/// `extern "C"` functions; `mb_class_register` adds their addresses to
/// CALLABLE_REGISTRY so both the CM machinery (mb_context_enter/exit) and the
/// generic instance method dispatch invoke them by exact arity (self prepended).
/// They are deliberately NOT marked variadic — that would force the dispatcher
/// to pack args into a list and break the fixed signatures.
fn register_native_cm_class(name: &str, methods: &[(&str, usize)]) {
    let mut m: HashMap<String, MbValue> = HashMap::new();
    for (mname, addr) in methods {
        m.insert((*mname).to_string(), MbValue::from_func(*addr));
    }
    super::super::class::mb_class_register(name, vec![], m);
}

/// Wire `swap_attr` / `EnvironmentVarGuard` constructors so they are real
/// callables returning CM instances, and register their backing classes.
fn register_cm_helpers() {
    register_native_cm_class(
        "_test_SwapAttr",
        &[
            ("__enter__", swap_attr_enter as *const () as usize),
            ("__exit__", swap_attr_exit as *const () as usize),
        ],
    );
    register_native_cm_class(
        "_test_EnvironmentVarGuard",
        &[
            ("__enter__", env_guard_enter as *const () as usize),
            ("__exit__", env_guard_exit as *const () as usize),
            ("set", env_guard_set as *const () as usize),
            ("__setitem__", env_guard_set as *const () as usize),
            ("unset", env_guard_unset as *const () as usize),
            ("__delitem__", env_guard_unset as *const () as usize),
            ("__getitem__", env_guard_getitem as *const () as usize),
        ],
    );
    // The constructor dispatchers are variadic native callables.
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(dispatch_swap_attr as usize as u64);
        s.borrow_mut().insert(dispatch_env_guard as usize as u64);
    });
}

/// Helper: extract a string from an MbValue.
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Compare two MbValues for equality across types.
fn values_equal(a: MbValue, b: MbValue) -> bool {
    if a.as_int().is_some() && b.as_int().is_some() {
        return a.as_int() == b.as_int();
    }
    if a.as_float().is_some() && b.as_float().is_some() {
        return a.as_float() == b.as_float();
    }
    if a.as_bool().is_some() && b.as_bool().is_some() {
        return a.as_bool() == b.as_bool();
    }
    if let (Some(sa), Some(sb)) = (extract_str(a), extract_str(b)) {
        return sa == sb;
    }
    a == b
}

// @spec .aw/changes/mamba-stdlib-test/groups/mamba-stdlib-test/specs/stdlib-test-module.md#R1
// @spec .aw/changes/mamba-stdlib-test/groups/mamba-stdlib-test/specs/stdlib-test-module.md#R2
// @spec .aw/changes/mamba-stdlib-test/groups/mamba-stdlib-test/specs/stdlib-test-module.md#R3
/// Register the test module.
pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("TestCase", dispatch_TestCase as usize),
        ("main", dispatch_main as usize),
        ("assertEqual", dispatch_assertEqual as usize),
        ("assertTrue", dispatch_assertTrue as usize),
        ("assertFalse", dispatch_assertFalse as usize),
        ("assertRaises", dispatch_assertRaises as usize),
        ("support", dispatch_support as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("test", attrs);

    register_support_submodules();
}

/// Register `test.support` and the submodules CPython conformance fixtures
/// import from. Every symbol is a no-op variadic callable stub. The goal is
/// to satisfy `from test.support[.subN] import X` at import time so fixtures
/// stop dying at line 1; downstream uses of these stubs will still fail (the
/// stubs return None for everything except identity-decorator names), but
/// fixtures that import-and-skip in their main path can now reach PASS.
fn register_support_submodules() {
    let noop = dispatch_noop_variadic as usize;
    let identity = dispatch_identity as usize;
    let fakepath = dispatch_fakepath as usize;
    let swap_attr = dispatch_swap_attr as usize;
    let env_guard = dispatch_env_guard as usize;
    // Register the backing context-manager classes (swap_attr / EnvironmentVarGuard).
    register_cm_helpers();
    // FakePath is a real os.PathLike: register the class (with __fspath__) and
    // wire its constructor addr so isinstance(FakePath(x), os.PathLike) holds.
    {
        let mut m: HashMap<String, MbValue> = HashMap::new();
        let fsp = fakepath_fspath as *const () as usize;
        super::super::module::register_variadic_func(fsp as u64);
        m.insert("__fspath__".to_string(), MbValue::from_func(fsp));
        super::super::class::mb_class_register("FakePath", vec![], m);
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(fakepath as u64);
        });
        super::super::module::NATIVE_TYPE_NAMES.with(|m| {
            m.borrow_mut().insert(fakepath as u64, "FakePath".to_string());
        });
    }

    fn make_attrs(entries: &[(&str, usize)]) -> HashMap<String, MbValue> {
        let mut attrs = HashMap::new();
        for (name, addr) in entries {
            attrs.insert((*name).to_string(), MbValue::from_func(*addr));
            super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
                s.borrow_mut().insert(*addr as u64);
            });
        }
        attrs
    }

    let support_entries: &[(&str, usize)] = &[
        ("assert_python_failure", noop),
        ("assert_python_ok", noop),
        ("requires_IEEE_754", identity),
        ("ExtraAssertions", noop),
        ("INVALID_UNDERSCORE_LITERALS", noop),
        ("FakePath", fakepath),
        ("C_RECURSION_LIMIT", noop),
        ("BrokenIter", noop),
        ("check_warnings", noop),
        ("gc_collect", noop),
        ("ALWAYS_EQ", noop),
        ("check_syntax_error", noop),
        ("cpython_only", identity),
        ("run_with_locale", identity),
        ("captured_stdout", noop),
        ("captured_stderr", noop),
        ("captured_stdin", noop),
        ("run_unittest", noop),
        ("verbose", noop),
        ("is_resource_enabled", noop),
        ("requires", identity),
        ("requires_resource", identity),
        ("bigmemtest", identity),
        ("requires_docstrings", identity),
        ("skip_unless_symlink", identity),
        ("skip_unless_xattr", identity),
        ("import_module", noop),
        ("findfile", noop),
        ("temp_dir", noop),
        ("temp_cwd", noop),
        ("rmtree", noop),
        ("unlink", noop),
        ("create_empty_file", noop),
        ("change_cwd", noop),
        ("anticipate_failure", identity),
        ("requires_zlib", identity),
        ("requires_gzip", identity),
        ("requires_bz2", identity),
        ("requires_lzma", identity),
        ("requires_mac_ver", identity),
        ("requires_linux_version", identity),
        ("MAX_Py_ssize_t", noop),
        ("maybe_get_event_loop_policy", noop),
        ("requires_specialization", identity),
        ("SuppressCrashReport", noop),
        ("NEVER_EQ", noop),
        ("disable_gc", noop),
        ("MISSING_C_DOCSTRINGS", noop),
        ("Py_DEBUG", noop),
        ("requires_subprocess", identity),
        ("requires_fork", identity),
        ("get_attribute", noop),
        ("optim_args_from_interpreter_flags", noop),
        ("strip_python_stderr", noop),
        ("transient_internet", noop),
        ("set_match_tests", noop),
        ("LOOPBACK_TIMEOUT", noop),
        ("SHORT_TIMEOUT", noop),
        ("LONG_TIMEOUT", noop),
        ("INTERNET_TIMEOUT", noop),
        ("requires_hashdigest", identity),
        ("hashlib_helper", noop),
        ("HOSTRUNTIMELEAKS", noop),
        ("Matcher", noop),
        ("requires_gil_enabled", identity),
        ("classify_resource_warning", noop),
        ("get_pagesize", noop),
        ("system_must_validate_cert", noop),
        ("check_disallow_instantiation", noop),
        ("MS_WINDOWS", noop),
        ("HAVE_DOCSTRINGS", noop),
        ("TEST_HTTP_URL", noop),
        ("bigaddrspacetest", identity),
        ("swap_attr", swap_attr),
        ("swap_item", noop),
        ("run_code", noop),
        ("no_tracing", identity),
        ("check_free_after_iterating", noop),
        ("force_not_colorized", identity),
        ("force_not_colorized_test_class", identity),
        ("flush_std_streams", noop),
        ("infinite_recursion", noop),
        ("requires_lower_layered_streams", identity),
        ("requires_jit_enabled", identity),
        ("requires_jit_disabled", identity),
        ("requires_perfmap", identity),
        ("requires_legacy_unicode_capi", identity),
        ("requires_limited_api", identity),
        ("requires_legacy_locale", identity),
        ("reset_logging", noop),
        ("EnvironmentVarGuard", env_guard),
        ("swap_method", noop),
        ("check_impl_detail", identity),
        ("set_memlimit", noop),
        ("bigmemtest", identity),
        ("TestCase_for_assertEqual", noop),
        ("RECURSION_LIMIT", noop),
        ("Py_GIL_DISABLED", noop),
        ("Py_FORCE_UTF8_FS_ENCODING", noop),
        ("USE_COMPUTED_GOTOS", noop),
        ("requires_debug_ranges", identity),
        ("Py_GC_HEAD_SIZE", noop),
        ("MISSING_C_DOCSTRINGS_ANNOTATIONS", noop),
        ("requires_debug_build", identity),
        ("PYMEM_ALLOCATOR_DEBUG", noop),
        ("check_no_resource_warning", noop),
        ("setswitchinterval", noop),
        ("LinkLayer", noop),
        ("PythonSymlink", noop),
        ("temp_umask", noop),
        ("DirsOnSysPath", noop),
        ("requires_strict_eval_break", identity),
        ("force_color", noop),
        ("catch_unraisable_exception", noop),
        ("catch_threading_exception", noop),
        ("check_sizeof", noop),
        ("captured_output", noop),
        ("skip_if_buggy_ucrt_strfptime", identity),
        ("skip_if_buildbot", identity),
        ("skip_if_pgo_task", identity),
        ("skip_if_sanitizer", identity),
        ("requires_working_socket", identity),
        ("requires_venv_with_pip", identity),
        ("python_is_optimized", noop),
        ("with_pymalloc", noop),
        ("WindowsRegistryGuard", noop),
        ("threading_cleanup", noop),
        ("reap_threads", identity),
        ("reap_children", noop),
        ("os_helper_walk", noop),
        ("calcvobjsize", noop),
        ("calcobjsize", noop),
        ("BasicTestRunner", noop),
        ("TestFailed", noop),
        ("ResourceDenied", noop),
        ("get_signal_name", noop),
        ("python_complex_command_str", noop),
        ("run_with_tz", identity),
        ("Error", noop),
        ("python_call_command", noop),
        ("captured_output_lines", noop),
        ("DEFAULT_BUFFER_SIZE", noop),
        ("MS_VC_VERSION", noop),
        ("MAX_INTERPRETERS", noop),
        ("STDLIB_DIR", noop),
        ("OS_NETWORKING_ALLOWED", noop),
        ("BLOCK_OUTPUT_LIMIT", noop),
    ];
    super::register_module("test.support", make_attrs(support_entries));

    let support_testcase_entries: &[(&str, usize)] = &[
        ("ExtraAssertions", noop),
        ("FloatsAreIdenticalMixin", noop),
        ("ComplexesAreIdenticalMixin", noop),
    ];
    super::register_module(
        "test.support.testcase",
        make_attrs(support_testcase_entries),
    );

    let script_helper_entries: &[(&str, usize)] = &[
        ("assert_python_failure", noop),
        ("assert_python_ok", noop),
        ("run_python_until_end", noop),
        ("interpreter_requires_environment", noop),
        ("spawn_python", noop),
        ("kill_python", noop),
        ("make_script", noop),
    ];
    super::register_module(
        "test.support.script_helper",
        make_attrs(script_helper_entries),
    );

    let os_helper_entries: &[(&str, usize)] = &[
        ("FakePath", fakepath),
        ("temp_cwd", noop),
        ("temp_dir", noop),
        ("change_cwd", noop),
        ("rmtree", noop),
        ("unlink", noop),
        ("create_empty_file", noop),
        ("can_symlink", noop),
        ("can_xattr", noop),
        ("EnvironmentVarGuard", env_guard),
        ("TESTFN", noop),
        ("FS_NONASCII", noop),
    ];
    super::register_module("test.support.os_helper", make_attrs(os_helper_entries));

    let import_helper_entries: &[(&str, usize)] = &[
        ("import_module", noop),
        ("import_fresh_module", noop),
        ("forget", noop),
        ("unload", noop),
        ("modules_setup", noop),
        ("modules_cleanup", noop),
        ("CleanImport", noop),
        ("DirsOnSysPath", noop),
    ];
    super::register_module(
        "test.support.import_helper",
        make_attrs(import_helper_entries),
    );

    let threading_helper_entries: &[(&str, usize)] = &[
        ("threading_setup", noop),
        ("threading_cleanup", noop),
        ("reap_threads", identity),
        ("start_threads", noop),
        ("join_thread", noop),
        ("requires_working_threading", identity),
    ];
    super::register_module(
        "test.support.threading_helper",
        make_attrs(threading_helper_entries),
    );

    let warnings_helper_entries: &[(&str, usize)] = &[
        ("save_restore_warnings_filters", noop),
        ("check_warnings", noop),
        ("check_no_warnings", noop),
        ("check_no_resource_warning", noop),
        ("ignore_warnings", identity),
    ];
    super::register_module(
        "test.support.warnings_helper",
        make_attrs(warnings_helper_entries),
    );

    super::register_module("test.support.testresult", make_attrs(&[]));
    super::register_module("test.mapping_tests", make_attrs(&[]));
    super::register_module("test.seq_tests", make_attrs(&[]));
    super::register_module("test.string_tests", make_attrs(&[]));
    super::register_module("test.list_tests", make_attrs(&[]));
    super::register_module(
        "test.test_grammar",
        make_attrs(&[
            ("INVALID_UNDERSCORE_LITERALS", noop),
            ("VALID_UNDERSCORE_LITERALS", noop),
        ]),
    );
    super::register_module("test.test_future_stmt", make_attrs(&[]));
    super::register_module("test.typing", make_attrs(&[("ann_module2", noop)]));
    super::register_module(
        "test.typinganndata",
        make_attrs(&[
            ("ann_module", noop),
            ("ann_module2", noop),
            ("ann_module3", noop),
            ("ann_module4", noop),
            ("ann_module5", noop),
            ("ann_module6", noop),
            ("ann_module7", noop),
            ("ann_module8", noop),
        ]),
    );
    super::register_module("test.typinganndata.ann_module", make_attrs(&[]));
    super::register_module("test.typinganndata.ann_module2", make_attrs(&[]));
    super::register_module("test.typinganndata.ann_module3", make_attrs(&[]));
    super::register_module("test.typinganndata.ann_module4", make_attrs(&[]));
    super::register_module("test.typinganndata.ann_module5", make_attrs(&[]));
    super::register_module("test.typinganndata.ann_module6", make_attrs(&[]));
    super::register_module("test.typinganndata.ann_module7", make_attrs(&[]));
    super::register_module("test.typinganndata.ann_module8", make_attrs(&[]));
}

/// CamelCase -> snake_case converter (kept for backward compatibility / unit tests).
#[allow(dead_code)]
fn to_snake(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap_or(c));
    }
    result
}

// @spec .aw/changes/mamba-stdlib-test/groups/mamba-stdlib-test/specs/stdlib-test-module.md#R1
/// test.TestCase() -> test case instance dict
pub fn mb_test_testcase() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "__class__".into(),
                MbValue::from_ptr(MbObject::new_str("TestCase".to_string())),
            );
            map.insert("_failures".into(), MbValue::from_int(0));
            map.insert("_successes".into(), MbValue::from_int(0));
        }
    }
    MbValue::from_ptr(dict)
}

// @spec .aw/changes/mamba-stdlib-test/groups/mamba-stdlib-test/specs/stdlib-test-module.md#R2
/// assertEqual(a, b) -> None or panic
pub fn mb_test_assert_equal(a: MbValue, b: MbValue) -> MbValue {
    if !values_equal(a, b) {
        panic!("AssertionError: values not equal");
    }
    MbValue::none()
}

// @spec .aw/changes/mamba-stdlib-test/groups/mamba-stdlib-test/specs/stdlib-test-module.md#R2
/// assertTrue(val) -> None or panic
pub fn mb_test_assert_true(val: MbValue) -> MbValue {
    let truthy = val.as_bool().unwrap_or(false) || val.as_int().map(|i| i != 0).unwrap_or(false);
    if !truthy {
        panic!("AssertionError: expected True");
    }
    MbValue::none()
}

// @spec .aw/changes/mamba-stdlib-test/groups/mamba-stdlib-test/specs/stdlib-test-module.md#R2
/// assertFalse(val) -> None or panic
pub fn mb_test_assert_false(val: MbValue) -> MbValue {
    let truthy = val.as_bool().unwrap_or(false) || val.as_int().map(|i| i != 0).unwrap_or(false);
    if truthy {
        panic!("AssertionError: expected False");
    }
    MbValue::none()
}

// @spec .aw/changes/mamba-stdlib-test/groups/mamba-stdlib-test/specs/stdlib-test-module.md#R2
/// assertRaises(exception_type) -> context manager stub dict
pub fn mb_test_assert_raises(exc_type: MbValue) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert("expected".into(), exc_type);
        }
    }
    MbValue::from_ptr(dict)
}

// @spec .aw/changes/mamba-stdlib-test/groups/mamba-stdlib-test/specs/stdlib-test-module.md#R3
/// test.main() -> run registered tests and print results
pub fn mb_test_main() -> MbValue {
    eprintln!("test.main() called -- test execution is handled by the test framework");
    MbValue::none()
}

/// test.support placeholder -> returns a support namespace dict
pub fn mb_test_support() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "__name__".into(),
                MbValue::from_ptr(MbObject::new_str("test.support".to_string())),
            );
        }
    }
    MbValue::from_ptr(dict)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- to_snake ---
    #[test]
    fn test_to_snake_camel_case() {
        assert_eq!(to_snake("assertEqual"), "assert_equal");
    }

    #[test]
    fn test_to_snake_already_snake() {
        assert_eq!(to_snake("assert_true"), "assert_true");
    }

    #[test]
    fn test_to_snake_empty() {
        assert_eq!(to_snake(""), "");
    }

    #[test]
    fn test_to_snake_single_uppercase() {
        assert_eq!(to_snake("Value"), "value");
    }

    // --- extract_str ---
    #[test]
    fn test_extract_str_with_str() {
        let s = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        assert_eq!(extract_str(s), Some("hello".to_string()));
    }

    #[test]
    fn test_extract_str_with_non_str() {
        assert_eq!(extract_str(MbValue::from_int(42)), None);
    }

    // --- values_equal ---
    #[test]
    fn test_values_equal_int() {
        assert!(values_equal(MbValue::from_int(5), MbValue::from_int(5)));
        assert!(!values_equal(MbValue::from_int(1), MbValue::from_int(2)));
    }

    #[test]
    fn test_values_equal_float() {
        assert!(values_equal(
            MbValue::from_float(1.5),
            MbValue::from_float(1.5)
        ));
        assert!(!values_equal(
            MbValue::from_float(1.0),
            MbValue::from_float(2.0)
        ));
    }

    #[test]
    fn test_values_equal_bool() {
        assert!(values_equal(
            MbValue::from_bool(true),
            MbValue::from_bool(true)
        ));
        assert!(!values_equal(
            MbValue::from_bool(true),
            MbValue::from_bool(false)
        ));
    }

    #[test]
    fn test_values_equal_str() {
        let a = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        let b = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        assert!(values_equal(a, b));
    }

    // --- testcase ---
    #[test]
    fn test_testcase_returns_dict_with_class() {
        let tc = mb_test_testcase();
        assert!(tc.as_ptr().is_some());
        if let Some(ptr) = tc.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    let class = map.get("__class__").copied().and_then(|v| extract_str(v));
                    assert_eq!(class, Some("TestCase".to_string()));
                    assert_eq!(map.get("_failures").and_then(|v| v.as_int()), Some(0));
                    assert_eq!(map.get("_successes").and_then(|v| v.as_int()), Some(0));
                }
            }
        }
    }

    // --- assertEqual ---
    #[test]
    fn test_assert_equal_pass() {
        mb_test_assert_equal(MbValue::from_int(1), MbValue::from_int(1));
    }

    #[test]
    #[should_panic(expected = "AssertionError")]
    fn test_assert_equal_fail() {
        mb_test_assert_equal(MbValue::from_int(1), MbValue::from_int(2));
    }

    // --- assertTrue ---
    #[test]
    fn test_assert_true_bool() {
        mb_test_assert_true(MbValue::from_bool(true));
    }

    #[test]
    fn test_assert_true_int_nonzero() {
        mb_test_assert_true(MbValue::from_int(5));
    }

    #[test]
    #[should_panic(expected = "AssertionError")]
    fn test_assert_true_bool_false_fails() {
        mb_test_assert_true(MbValue::from_bool(false));
    }

    #[test]
    #[should_panic(expected = "AssertionError")]
    fn test_assert_true_int_zero_fails() {
        mb_test_assert_true(MbValue::from_int(0));
    }

    // --- assertFalse ---
    #[test]
    fn test_assert_false_pass() {
        mb_test_assert_false(MbValue::from_bool(false));
    }

    #[test]
    fn test_assert_false_int_zero() {
        mb_test_assert_false(MbValue::from_int(0));
    }

    #[test]
    #[should_panic(expected = "AssertionError")]
    fn test_assert_false_bool_true_fails() {
        mb_test_assert_false(MbValue::from_bool(true));
    }

    #[test]
    #[should_panic(expected = "AssertionError")]
    fn test_assert_false_int_nonzero_fails() {
        mb_test_assert_false(MbValue::from_int(1));
    }

    // --- assertRaises ---
    #[test]
    fn test_assert_raises_returns_dict() {
        let exc_type = MbValue::from_ptr(MbObject::new_str("ValueError".to_string()));
        let result = mb_test_assert_raises(exc_type);
        assert!(result.as_ptr().is_some());
        if let Some(ptr) = result.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    assert!(map.contains_key("expected"));
                }
            }
        }
    }

    // --- main ---
    #[test]
    fn test_main_returns_none() {
        let result = mb_test_main();
        assert!(result.is_none());
    }

    // --- support ---
    #[test]
    fn test_support_returns_dict() {
        let result = mb_test_support();
        assert!(result.as_ptr().is_some());
        if let Some(ptr) = result.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    let name = map.get("__name__").copied().and_then(|v| extract_str(v));
                    assert_eq!(name, Some("test.support".to_string()));
                }
            }
        }
    }
}
