use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/importlib/import_module_empty_name_raises.py`.
#[test]
fn test_gen_errors_std_libs_importlib_import_module_empty_name_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "errors"
# case = "import_module_empty_name_raises"
# subject = "importlib.import_module"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_importlib"
# status = "filled"
# ///
"""importlib.import_module: import_module_empty_name_raises (errors)."""
import importlib

_raised = False
try:
    importlib.import_module("")
except ValueError:
    _raised = True
assert _raised, "import_module_empty_name_raises: expected ValueError"
print("import_module_empty_name_raises OK")
"###);
    assert_output(&out, r###"import_module_empty_name_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/importlib/import_module_missing_raises.py`.
#[test]
fn test_gen_errors_std_libs_importlib_import_module_missing_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "errors"
# case = "import_module_missing_raises"
# subject = "importlib.import_module"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_importlib"
# status = "filled"
# ///
"""importlib.import_module: import_module_missing_raises (errors)."""
import importlib

_raised = False
try:
    importlib.import_module("no_such_module_xyzzy_123")
except ModuleNotFoundError:
    _raised = True
assert _raised, "import_module_missing_raises: expected ModuleNotFoundError"
print("import_module_missing_raises OK")
"###);
    assert_output(&out, r###"import_module_missing_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/importlib/import_module_relative_no_package_raises.py`.
#[test]
fn test_gen_errors_std_libs_importlib_import_module_relative_no_package_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "errors"
# case = "import_module_relative_no_package_raises"
# subject = "importlib.import_module"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_importlib"
# status = "filled"
# ///
"""importlib.import_module: import_module_relative_no_package_raises (errors)."""
import importlib

_raised = False
try:
    importlib.import_module(".relative_no_pkg")
except TypeError:
    _raised = True
assert _raised, "import_module_relative_no_package_raises: expected TypeError"
print("import_module_relative_no_package_raises OK")
"###);
    assert_output(&out, r###"import_module_relative_no_package_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/importlib/reload_non_module_raises.py`.
#[test]
fn test_gen_errors_std_libs_importlib_reload_non_module_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib"
# dimension = "errors"
# case = "reload_non_module_raises"
# subject = "importlib.reload"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_importlib"
# status = "filled"
# ///
"""importlib.reload: reload_non_module_raises (errors)."""
import importlib

_raised = False
try:
    importlib.reload(42)
except TypeError:
    _raised = True
assert _raised, "reload_non_module_raises: expected TypeError"
print("reload_non_module_raises OK")
"###);
    assert_output(&out, r###"reload_non_module_raises OK
"###);
}
