use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/importlib/find_spec_missing_returns_none.py`.
#[test]
fn test_gen_behavior_std_libs_importlib_find_spec_missing_returns_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "behavior"
# case = "find_spec_missing_returns_none"
# subject = "importlib.util.find_spec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_importlib"
# status = "filled"
# ///
"""importlib.util.find_spec: find_spec for a non-existent module name returns None rather than raising"""
import importlib.util

spec = importlib.util.find_spec("no_such_module_for_find_spec")
assert spec is None, "find_spec for a missing module returns None"
print("find_spec_missing_returns_none OK")
"###);
    assert_output(&out, r###"find_spec_missing_returns_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/importlib/import_module_imports_real_module.py`.
#[test]
fn test_gen_behavior_std_libs_importlib_import_module_imports_real_module() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "behavior"
# case = "import_module_imports_real_module"
# subject = "importlib.import_module"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_importlib"
# status = "filled"
# ///
"""importlib.import_module: import_module("json") returns the json module object whose __name__ is 'json', equivalent to a plain import"""
import importlib

mod = importlib.import_module("json")
assert mod.__name__ == "json", mod.__name__
import json
assert mod is json, "import_module returns the same module object as a plain import"
print("import_module_imports_real_module OK")
"###);
    assert_output(&out, r###"import_module_imports_real_module OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/importlib/invalidate_caches_returns_none.py`.
#[test]
fn test_gen_behavior_std_libs_importlib_invalidate_caches_returns_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "behavior"
# case = "invalidate_caches_returns_none"
# subject = "importlib.invalidate_caches"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""importlib.invalidate_caches: invalidate_caches() runs without error and returns None"""
import importlib

assert importlib.invalidate_caches() is None, "invalidate_caches() returns None"
print("invalidate_caches_returns_none OK")
"###);
    assert_output(&out, r###"invalidate_caches_returns_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/importlib/module_not_found_error_is_import_error.py`.
#[test]
fn test_gen_behavior_std_libs_importlib_module_not_found_error_is_import_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "behavior"
# case = "module_not_found_error_is_import_error"
# subject = "importlib.import_module"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_importlib"
# status = "filled"
# ///
"""importlib.import_module: ModuleNotFoundError is a subclass of ImportError, so import_module misses can be caught as ImportError"""
import importlib

assert issubclass(ModuleNotFoundError, ImportError), "ModuleNotFoundError <: ImportError"

caught_as_import_error = False
try:
    importlib.import_module("no_such_module_xyzzy_123")
except ImportError:
    caught_as_import_error = True
assert caught_as_import_error, "a missing import is catchable as ImportError"
print("module_not_found_error_is_import_error OK")
"###);
    assert_output(&out, r###"module_not_found_error_is_import_error OK
"###);
}
