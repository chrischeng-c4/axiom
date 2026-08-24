use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/unittest_suite/BaseTestSuite____call____result_as_TestResult_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_suite_BaseTestSuite____call____result_as_TestResult_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_suite"
# dimension = "type"
# case = "BaseTestSuite____call____result_as_TestResult_wrong"
# subject = "unittest.suite.BaseTestSuite.__call__(result: TestResult)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/suite.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.suite.BaseTestSuite.__call__(result: TestResult); call it with the wrong type.

typeshed contract: result is TestResult. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.suite import BaseTestSuite
obj = object.__new__(BaseTestSuite)
try:
    obj.__call__(_W())  # result: TestResult <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_suite/BaseTestSuite__addTest__test_as__TestType_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_suite_BaseTestSuite__addTest__test_as__TestType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_suite"
# dimension = "type"
# case = "BaseTestSuite__addTest__test_as__TestType_wrong"
# subject = "unittest.suite.BaseTestSuite.addTest(test: _TestType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/suite.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.suite.BaseTestSuite.addTest(test: _TestType); call it with the wrong type.

typeshed contract: test is _TestType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.suite import BaseTestSuite
obj = object.__new__(BaseTestSuite)
try:
    obj.addTest(_W())  # test: _TestType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_suite/BaseTestSuite__addTests__tests_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_suite_BaseTestSuite__addTests__tests_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_suite"
# dimension = "type"
# case = "BaseTestSuite__addTests__tests_as_Iterable_wrong"
# subject = "unittest.suite.BaseTestSuite.addTests(tests: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/suite.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.suite.BaseTestSuite.addTests(tests: Iterable); call it with the wrong type.

typeshed contract: tests is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.suite import BaseTestSuite
obj = object.__new__(BaseTestSuite)
try:
    obj.addTests(_W())  # tests: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_suite/BaseTestSuite__init__tests_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_suite_BaseTestSuite__init__tests_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_suite"
# dimension = "type"
# case = "BaseTestSuite__init__tests_as_Iterable_wrong"
# subject = "unittest.suite.BaseTestSuite.__init__(tests: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/suite.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.suite.BaseTestSuite.__init__(tests: Iterable); call it with the wrong type.

typeshed contract: tests is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.suite import BaseTestSuite
try:
    BaseTestSuite(_W())  # tests: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_suite/BaseTestSuite__run__result_as_TestResult_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_suite_BaseTestSuite__run__result_as_TestResult_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_suite"
# dimension = "type"
# case = "BaseTestSuite__run__result_as_TestResult_wrong"
# subject = "unittest.suite.BaseTestSuite.run(result: TestResult)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/suite.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.suite.BaseTestSuite.run(result: TestResult); call it with the wrong type.

typeshed contract: result is TestResult. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.suite import BaseTestSuite
obj = object.__new__(BaseTestSuite)
try:
    obj.run(_W())  # result: TestResult <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_suite/TestSuite__run__result_as_TestResult_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_suite_TestSuite__run__result_as_TestResult_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_suite"
# dimension = "type"
# case = "TestSuite__run__result_as_TestResult_wrong"
# subject = "unittest.suite.TestSuite.run(result: TestResult)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/suite.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.suite.TestSuite.run(result: TestResult); call it with the wrong type.

typeshed contract: result is TestResult. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.suite import TestSuite
obj = object.__new__(TestSuite)
try:
    obj.run(_W())  # result: TestResult <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
