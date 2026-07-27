use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/unittest_result/TestResult__addDuration__test_as_TestCase_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_result_TestResult__addDuration__test_as_TestCase_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_result"
# dimension = "type"
# case = "TestResult__addDuration__test_as_TestCase_wrong"
# subject = "unittest.result.TestResult.addDuration(test: TestCase)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/result.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.result.TestResult.addDuration(test: TestCase); call it with the wrong type.

typeshed contract: test is TestCase. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.result import TestResult
obj = object.__new__(TestResult)
try:
    obj.addDuration(_W(), 0.0)  # test: TestCase <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_result/TestResult__addError__test_as_TestCase_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_result_TestResult__addError__test_as_TestCase_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_result"
# dimension = "type"
# case = "TestResult__addError__test_as_TestCase_wrong"
# subject = "unittest.result.TestResult.addError(test: TestCase)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/result.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.result.TestResult.addError(test: TestCase); call it with the wrong type.

typeshed contract: test is TestCase. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.result import TestResult
obj = object.__new__(TestResult)
try:
    obj.addError(_W(), None)  # test: TestCase <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_result/TestResult__addExpectedFailure__test_as_TestCase_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_result_TestResult__addExpectedFailure__test_as_TestCase_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_result"
# dimension = "type"
# case = "TestResult__addExpectedFailure__test_as_TestCase_wrong"
# subject = "unittest.result.TestResult.addExpectedFailure(test: TestCase)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/result.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.result.TestResult.addExpectedFailure(test: TestCase); call it with the wrong type.

typeshed contract: test is TestCase. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.result import TestResult
obj = object.__new__(TestResult)
try:
    obj.addExpectedFailure(_W(), None)  # test: TestCase <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_result/TestResult__addFailure__test_as_TestCase_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_result_TestResult__addFailure__test_as_TestCase_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_result"
# dimension = "type"
# case = "TestResult__addFailure__test_as_TestCase_wrong"
# subject = "unittest.result.TestResult.addFailure(test: TestCase)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/result.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.result.TestResult.addFailure(test: TestCase); call it with the wrong type.

typeshed contract: test is TestCase. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.result import TestResult
obj = object.__new__(TestResult)
try:
    obj.addFailure(_W(), None)  # test: TestCase <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_result/TestResult__addSkip__test_as_TestCase_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_result_TestResult__addSkip__test_as_TestCase_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_result"
# dimension = "type"
# case = "TestResult__addSkip__test_as_TestCase_wrong"
# subject = "unittest.result.TestResult.addSkip(test: TestCase)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/result.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.result.TestResult.addSkip(test: TestCase); call it with the wrong type.

typeshed contract: test is TestCase. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.result import TestResult
obj = object.__new__(TestResult)
try:
    obj.addSkip(_W(), "")  # test: TestCase <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_result/TestResult__addSubTest__test_as_TestCase_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_result_TestResult__addSubTest__test_as_TestCase_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_result"
# dimension = "type"
# case = "TestResult__addSubTest__test_as_TestCase_wrong"
# subject = "unittest.result.TestResult.addSubTest(test: TestCase)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/result.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.result.TestResult.addSubTest(test: TestCase); call it with the wrong type.

typeshed contract: test is TestCase. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.result import TestResult
obj = object.__new__(TestResult)
try:
    obj.addSubTest(_W(), None, None)  # test: TestCase <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_result/TestResult__addSuccess__test_as_TestCase_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_result_TestResult__addSuccess__test_as_TestCase_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_result"
# dimension = "type"
# case = "TestResult__addSuccess__test_as_TestCase_wrong"
# subject = "unittest.result.TestResult.addSuccess(test: TestCase)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/result.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.result.TestResult.addSuccess(test: TestCase); call it with the wrong type.

typeshed contract: test is TestCase. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.result import TestResult
obj = object.__new__(TestResult)
try:
    obj.addSuccess(_W())  # test: TestCase <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_result/TestResult__addUnexpectedSuccess__test_as_TestCase_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_result_TestResult__addUnexpectedSuccess__test_as_TestCase_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_result"
# dimension = "type"
# case = "TestResult__addUnexpectedSuccess__test_as_TestCase_wrong"
# subject = "unittest.result.TestResult.addUnexpectedSuccess(test: TestCase)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/result.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.result.TestResult.addUnexpectedSuccess(test: TestCase); call it with the wrong type.

typeshed contract: test is TestCase. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.result import TestResult
obj = object.__new__(TestResult)
try:
    obj.addUnexpectedSuccess(_W())  # test: TestCase <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_result/TestResult__init__stream_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_result_TestResult__init__stream_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_result"
# dimension = "type"
# case = "TestResult__init__stream_as_typed_wrong"
# subject = "unittest.result.TestResult.__init__(stream: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/result.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.result.TestResult.__init__(stream: typed); call it with the wrong type.

typeshed contract: stream is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.result import TestResult
try:
    TestResult(_W())  # stream: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_result/TestResult__startTest__test_as_TestCase_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_result_TestResult__startTest__test_as_TestCase_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_result"
# dimension = "type"
# case = "TestResult__startTest__test_as_TestCase_wrong"
# subject = "unittest.result.TestResult.startTest(test: TestCase)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/result.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.result.TestResult.startTest(test: TestCase); call it with the wrong type.

typeshed contract: test is TestCase. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.result import TestResult
obj = object.__new__(TestResult)
try:
    obj.startTest(_W())  # test: TestCase <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_result/TestResult__stopTest__test_as_TestCase_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_result_TestResult__stopTest__test_as_TestCase_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_result"
# dimension = "type"
# case = "TestResult__stopTest__test_as_TestCase_wrong"
# subject = "unittest.result.TestResult.stopTest(test: TestCase)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/result.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.result.TestResult.stopTest(test: TestCase); call it with the wrong type.

typeshed contract: test is TestCase. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.result import TestResult
obj = object.__new__(TestResult)
try:
    obj.stopTest(_W())  # test: TestCase <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
