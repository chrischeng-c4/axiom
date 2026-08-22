use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/unittest_loader/TestLoader__discover__start_dir_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_loader_TestLoader__discover__start_dir_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_loader"
# dimension = "type"
# case = "TestLoader__discover__start_dir_as_str_wrong"
# subject = "unittest.loader.TestLoader.discover(start_dir: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/loader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.loader.TestLoader.discover(start_dir: str); call it with the wrong type.

typeshed contract: start_dir is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.loader import TestLoader
obj = object.__new__(TestLoader)
try:
    obj.discover(12345)  # start_dir: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_loader/TestLoader__loadTestsFromModule__module_as_ModuleType_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_loader_TestLoader__loadTestsFromModule__module_as_ModuleType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_loader"
# dimension = "type"
# case = "TestLoader__loadTestsFromModule__module_as_ModuleType_wrong"
# subject = "unittest.loader.TestLoader.loadTestsFromModule(module: ModuleType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/loader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.loader.TestLoader.loadTestsFromModule(module: ModuleType); call it with the wrong type.

typeshed contract: module is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.loader import TestLoader
obj = object.__new__(TestLoader)
try:
    obj.loadTestsFromModule(_W())  # module: ModuleType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_loader/TestLoader__loadTestsFromName__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_loader_TestLoader__loadTestsFromName__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_loader"
# dimension = "type"
# case = "TestLoader__loadTestsFromName__name_as_str_wrong"
# subject = "unittest.loader.TestLoader.loadTestsFromName(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/loader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.loader.TestLoader.loadTestsFromName(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.loader import TestLoader
obj = object.__new__(TestLoader)
try:
    obj.loadTestsFromName(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_loader/findTestCases__module_as_ModuleType_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_loader_findTestCases__module_as_ModuleType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_loader"
# dimension = "type"
# case = "findTestCases__module_as_ModuleType_wrong"
# subject = "unittest.loader.findTestCases(module: ModuleType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/loader.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.loader.findTestCases(module: ModuleType); call it with the wrong type.

typeshed contract: module is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.loader import findTestCases
try:
    findTestCases(_W())  # module: ModuleType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
