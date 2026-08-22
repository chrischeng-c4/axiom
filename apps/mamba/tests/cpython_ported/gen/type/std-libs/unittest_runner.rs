use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/unittest_runner/TextTestResult__getDescription__test_as_TestCase_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_runner_TextTestResult__getDescription__test_as_TestCase_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_runner"
# dimension = "type"
# case = "TextTestResult__getDescription__test_as_TestCase_wrong"
# subject = "unittest.runner.TextTestResult.getDescription(test: TestCase)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/runner.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.runner.TextTestResult.getDescription(test: TestCase); call it with the wrong type.

typeshed contract: test is TestCase. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.runner import TextTestResult
obj = object.__new__(TextTestResult)
try:
    obj.getDescription(_W())  # test: TestCase <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_runner/TextTestResult__init__stream_as__StreamT_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_runner_TextTestResult__init__stream_as__StreamT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_runner"
# dimension = "type"
# case = "TextTestResult__init__stream_as__StreamT_wrong"
# subject = "unittest.runner.TextTestResult.__init__(stream: _StreamT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/runner.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.runner.TextTestResult.__init__(stream: _StreamT); call it with the wrong type.

typeshed contract: stream is _StreamT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.runner import TextTestResult
try:
    TextTestResult(_W(), True, 0)  # stream: _StreamT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_runner/TextTestResult__printErrorList__flavour_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_runner_TextTestResult__printErrorList__flavour_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_runner"
# dimension = "type"
# case = "TextTestResult__printErrorList__flavour_as_str_wrong"
# subject = "unittest.runner.TextTestResult.printErrorList(flavour: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/runner.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.runner.TextTestResult.printErrorList(flavour: str); call it with the wrong type.

typeshed contract: flavour is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.runner import TextTestResult
obj = object.__new__(TextTestResult)
try:
    obj.printErrorList(12345, None)  # flavour: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_runner/TextTestRunner__init__stream_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_runner_TextTestRunner__init__stream_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_runner"
# dimension = "type"
# case = "TextTestRunner__init__stream_as_typed_wrong"
# subject = "unittest.runner.TextTestRunner.__init__(stream: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/runner.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.runner.TextTestRunner.__init__(stream: typed); call it with the wrong type.

typeshed contract: stream is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.runner import TextTestRunner
try:
    TextTestRunner(_W())  # stream: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_runner/TextTestRunner__run__test_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_runner_TextTestRunner__run__test_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_runner"
# dimension = "type"
# case = "TextTestRunner__run__test_as_typed_wrong"
# subject = "unittest.runner.TextTestRunner.run(test: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/runner.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.runner.TextTestRunner.run(test: typed); call it with the wrong type.

typeshed contract: test is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.runner import TextTestRunner
obj = object.__new__(TextTestRunner)
try:
    obj.run(_W())  # test: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
