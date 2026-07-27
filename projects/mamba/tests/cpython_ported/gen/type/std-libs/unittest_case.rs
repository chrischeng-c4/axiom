use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/unittest_case/FunctionTestCase__init__testFunc_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_FunctionTestCase__init__testFunc_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "FunctionTestCase__init__testFunc_as_Callable_wrong"
# subject = "unittest.case.FunctionTestCase.__init__(testFunc: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.FunctionTestCase.__init__(testFunc: Callable); call it with the wrong type.

typeshed contract: testFunc is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import FunctionTestCase
try:
    FunctionTestCase(_W())  # testFunc: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/SkipTest__init__reason_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_SkipTest__init__reason_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "SkipTest__init__reason_as_str_wrong"
# subject = "unittest.case.SkipTest.__init__(reason: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.SkipTest.__init__(reason: str); call it with the wrong type.

typeshed contract: reason is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.case import SkipTest
try:
    SkipTest(12345)  # reason: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase____call____result_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase____call____result_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase____call____result_as_typed_wrong"
# subject = "unittest.case.TestCase.__call__(result: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.__call__(result: typed); call it with the wrong type.

typeshed contract: result is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.__call__(_W())  # result: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__addClassCleanup__function_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__addClassCleanup__function_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__addClassCleanup__function_as_Callable_wrong"
# subject = "unittest.case.TestCase.addClassCleanup(function: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.addClassCleanup(function: Callable); call it with the wrong type.

typeshed contract: function is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
try:
    TestCase.addClassCleanup(_W())  # function: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__addCleanup__function_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__addCleanup__function_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__addCleanup__function_as_Callable_wrong"
# subject = "unittest.case.TestCase.addCleanup(function: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.addCleanup(function: Callable); call it with the wrong type.

typeshed contract: function is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.addCleanup(_W())  # function: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__addTypeEqualityFunc__typeobj_as_type_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__addTypeEqualityFunc__typeobj_as_type_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__addTypeEqualityFunc__typeobj_as_type_wrong"
# subject = "unittest.case.TestCase.addTypeEqualityFunc(typeobj: type)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.addTypeEqualityFunc(typeobj: type); call it with the wrong type.

typeshed contract: typeobj is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.addTypeEqualityFunc(_W(), None)  # typeobj: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertAlmostEqual__first_as_SupportsSub_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertAlmostEqual__first_as_SupportsSub_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertAlmostEqual__first_as_SupportsSub_wrong"
# subject = "unittest.case.TestCase.assertAlmostEqual(first: SupportsSub)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertAlmostEqual(first: SupportsSub); call it with the wrong type.

typeshed contract: first is SupportsSub. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertAlmostEqual(_W(), None)  # first: SupportsSub <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertAlmostEqual__first_as__S_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertAlmostEqual__first_as__S_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertAlmostEqual__first_as__S_wrong"
# subject = "unittest.case.TestCase.assertAlmostEqual(first: _S)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertAlmostEqual(first: _S); call it with the wrong type.

typeshed contract: first is _S. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertAlmostEqual(_W(), None, None, None, None)  # first: _S <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertAlmostEqual__first_as__T_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertAlmostEqual__first_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertAlmostEqual__first_as__T_wrong"
# subject = "unittest.case.TestCase.assertAlmostEqual(first: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertAlmostEqual(first: _T); call it with the wrong type.

typeshed contract: first is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertAlmostEqual(_W(), None)  # first: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertCountEqual__first_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertCountEqual__first_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertCountEqual__first_as_Iterable_wrong"
# subject = "unittest.case.TestCase.assertCountEqual(first: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertCountEqual(first: Iterable); call it with the wrong type.

typeshed contract: first is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertCountEqual(_W(), None)  # first: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertDictContainsSubset__subset_as_Mapping_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertDictContainsSubset__subset_as_Mapping_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertDictContainsSubset__subset_as_Mapping_wrong"
# subject = "unittest.case.TestCase.assertDictContainsSubset(subset: Mapping)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertDictContainsSubset(subset: Mapping); call it with the wrong type.

typeshed contract: subset is Mapping. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertDictContainsSubset(_W(), None)  # subset: Mapping <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertDictEqual__d1_as_Mapping_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertDictEqual__d1_as_Mapping_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertDictEqual__d1_as_Mapping_wrong"
# subject = "unittest.case.TestCase.assertDictEqual(d1: Mapping)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertDictEqual(d1: Mapping); call it with the wrong type.

typeshed contract: d1 is Mapping. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertDictEqual(_W(), None)  # d1: Mapping <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertEndsWith__s_as__SB_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertEndsWith__s_as__SB_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertEndsWith__s_as__SB_wrong"
# subject = "unittest.case.TestCase.assertEndsWith(s: _SB)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertEndsWith(s: _SB); call it with the wrong type.

typeshed contract: s is _SB. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertEndsWith(_W(), None)  # s: _SB <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertGreaterEqual__a_as_SupportsDunderGE_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertGreaterEqual__a_as_SupportsDunderGE_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertGreaterEqual__a_as_SupportsDunderGE_wrong"
# subject = "unittest.case.TestCase.assertGreaterEqual(a: SupportsDunderGE)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertGreaterEqual(a: SupportsDunderGE); call it with the wrong type.

typeshed contract: a is SupportsDunderGE. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertGreaterEqual(_W(), None)  # a: SupportsDunderGE <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertGreaterEqual__a_as__T_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertGreaterEqual__a_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertGreaterEqual__a_as__T_wrong"
# subject = "unittest.case.TestCase.assertGreaterEqual(a: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertGreaterEqual(a: _T); call it with the wrong type.

typeshed contract: a is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertGreaterEqual(_W(), None)  # a: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertGreater__a_as_SupportsDunderGT_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertGreater__a_as_SupportsDunderGT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertGreater__a_as_SupportsDunderGT_wrong"
# subject = "unittest.case.TestCase.assertGreater(a: SupportsDunderGT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertGreater(a: SupportsDunderGT); call it with the wrong type.

typeshed contract: a is SupportsDunderGT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertGreater(_W(), None)  # a: SupportsDunderGT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertGreater__a_as__T_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertGreater__a_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertGreater__a_as__T_wrong"
# subject = "unittest.case.TestCase.assertGreater(a: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertGreater(a: _T); call it with the wrong type.

typeshed contract: a is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertGreater(_W(), None)  # a: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertHasAttr__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertHasAttr__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertHasAttr__name_as_str_wrong"
# subject = "unittest.case.TestCase.assertHasAttr(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertHasAttr(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertHasAttr(None, 12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertIn__container_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertIn__container_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertIn__container_as_typed_wrong"
# subject = "unittest.case.TestCase.assertIn(container: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertIn(container: typed); call it with the wrong type.

typeshed contract: container is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertIn(None, _W())  # container: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertIsInstance__cls_as__ClassInfo_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertIsInstance__cls_as__ClassInfo_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertIsInstance__cls_as__ClassInfo_wrong"
# subject = "unittest.case.TestCase.assertIsInstance(cls: _ClassInfo)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertIsInstance(cls: _ClassInfo); call it with the wrong type.

typeshed contract: cls is _ClassInfo. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertIsInstance(None, _W())  # cls: _ClassInfo <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertIsSubclass__cls_as_type_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertIsSubclass__cls_as_type_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertIsSubclass__cls_as_type_wrong"
# subject = "unittest.case.TestCase.assertIsSubclass(cls: type)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertIsSubclass(cls: type); call it with the wrong type.

typeshed contract: cls is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertIsSubclass(_W(), None)  # cls: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertLessEqual__a_as_SupportsDunderLE_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertLessEqual__a_as_SupportsDunderLE_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertLessEqual__a_as_SupportsDunderLE_wrong"
# subject = "unittest.case.TestCase.assertLessEqual(a: SupportsDunderLE)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertLessEqual(a: SupportsDunderLE); call it with the wrong type.

typeshed contract: a is SupportsDunderLE. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertLessEqual(_W(), None)  # a: SupportsDunderLE <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertLessEqual__a_as__T_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertLessEqual__a_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertLessEqual__a_as__T_wrong"
# subject = "unittest.case.TestCase.assertLessEqual(a: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertLessEqual(a: _T); call it with the wrong type.

typeshed contract: a is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertLessEqual(_W(), None)  # a: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertLess__a_as_SupportsDunderLT_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertLess__a_as_SupportsDunderLT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertLess__a_as_SupportsDunderLT_wrong"
# subject = "unittest.case.TestCase.assertLess(a: SupportsDunderLT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertLess(a: SupportsDunderLT); call it with the wrong type.

typeshed contract: a is SupportsDunderLT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertLess(_W(), None)  # a: SupportsDunderLT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertLess__a_as__T_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertLess__a_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertLess__a_as__T_wrong"
# subject = "unittest.case.TestCase.assertLess(a: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertLess(a: _T); call it with the wrong type.

typeshed contract: a is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertLess(_W(), None)  # a: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertListEqual__list1_as_list_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertListEqual__list1_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertListEqual__list1_as_list_wrong"
# subject = "unittest.case.TestCase.assertListEqual(list1: list)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertListEqual(list1: list); call it with the wrong type.

typeshed contract: list1 is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertListEqual(12345, None)  # list1: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertLogs__logger_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertLogs__logger_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertLogs__logger_as_typed_wrong"
# subject = "unittest.case.TestCase.assertLogs(logger: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertLogs(logger: typed); call it with the wrong type.

typeshed contract: logger is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertLogs(_W())  # logger: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertMultiLineEqual__first_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertMultiLineEqual__first_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertMultiLineEqual__first_as_str_wrong"
# subject = "unittest.case.TestCase.assertMultiLineEqual(first: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertMultiLineEqual(first: str); call it with the wrong type.

typeshed contract: first is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertMultiLineEqual(12345, "")  # first: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertNoLogs__logger_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertNoLogs__logger_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertNoLogs__logger_as_typed_wrong"
# subject = "unittest.case.TestCase.assertNoLogs(logger: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertNoLogs(logger: typed); call it with the wrong type.

typeshed contract: logger is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertNoLogs(_W())  # logger: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertNotAlmostEqual__first_as_SupportsSub_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertNotAlmostEqual__first_as_SupportsSub_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertNotAlmostEqual__first_as_SupportsSub_wrong"
# subject = "unittest.case.TestCase.assertNotAlmostEqual(first: SupportsSub)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertNotAlmostEqual(first: SupportsSub); call it with the wrong type.

typeshed contract: first is SupportsSub. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertNotAlmostEqual(_W(), None)  # first: SupportsSub <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertNotAlmostEqual__first_as__S_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertNotAlmostEqual__first_as__S_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertNotAlmostEqual__first_as__S_wrong"
# subject = "unittest.case.TestCase.assertNotAlmostEqual(first: _S)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertNotAlmostEqual(first: _S); call it with the wrong type.

typeshed contract: first is _S. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertNotAlmostEqual(_W(), None, None, None, None)  # first: _S <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertNotAlmostEqual__first_as__T_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertNotAlmostEqual__first_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertNotAlmostEqual__first_as__T_wrong"
# subject = "unittest.case.TestCase.assertNotAlmostEqual(first: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertNotAlmostEqual(first: _T); call it with the wrong type.

typeshed contract: first is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertNotAlmostEqual(_W(), None)  # first: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertNotEndsWith__s_as__SB_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertNotEndsWith__s_as__SB_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertNotEndsWith__s_as__SB_wrong"
# subject = "unittest.case.TestCase.assertNotEndsWith(s: _SB)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertNotEndsWith(s: _SB); call it with the wrong type.

typeshed contract: s is _SB. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertNotEndsWith(_W(), None)  # s: _SB <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertNotHasAttr__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertNotHasAttr__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertNotHasAttr__name_as_str_wrong"
# subject = "unittest.case.TestCase.assertNotHasAttr(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertNotHasAttr(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertNotHasAttr(None, 12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertNotIn__container_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertNotIn__container_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertNotIn__container_as_typed_wrong"
# subject = "unittest.case.TestCase.assertNotIn(container: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertNotIn(container: typed); call it with the wrong type.

typeshed contract: container is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertNotIn(None, _W())  # container: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertNotIsInstance__cls_as__ClassInfo_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertNotIsInstance__cls_as__ClassInfo_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertNotIsInstance__cls_as__ClassInfo_wrong"
# subject = "unittest.case.TestCase.assertNotIsInstance(cls: _ClassInfo)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertNotIsInstance(cls: _ClassInfo); call it with the wrong type.

typeshed contract: cls is _ClassInfo. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertNotIsInstance(None, _W())  # cls: _ClassInfo <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertNotIsSubclass__cls_as_type_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertNotIsSubclass__cls_as_type_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertNotIsSubclass__cls_as_type_wrong"
# subject = "unittest.case.TestCase.assertNotIsSubclass(cls: type)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertNotIsSubclass(cls: type); call it with the wrong type.

typeshed contract: cls is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertNotIsSubclass(_W(), None)  # cls: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertNotRegex__text_as_AnyStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertNotRegex__text_as_AnyStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertNotRegex__text_as_AnyStr_wrong"
# subject = "unittest.case.TestCase.assertNotRegex(text: AnyStr)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertNotRegex(text: AnyStr); call it with the wrong type.

typeshed contract: text is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertNotRegex(_W(), None)  # text: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertNotStartsWith__s_as__SB_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertNotStartsWith__s_as__SB_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertNotStartsWith__s_as__SB_wrong"
# subject = "unittest.case.TestCase.assertNotStartsWith(s: _SB)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertNotStartsWith(s: _SB); call it with the wrong type.

typeshed contract: s is _SB. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertNotStartsWith(_W(), None)  # s: _SB <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertRaisesRegex__expected_exception_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertRaisesRegex__expected_exception_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertRaisesRegex__expected_exception_as_typed_wrong"
# subject = "unittest.case.TestCase.assertRaisesRegex(expected_exception: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertRaisesRegex(expected_exception: typed); call it with the wrong type.

typeshed contract: expected_exception is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertRaisesRegex(_W(), None, None)  # expected_exception: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertRaises__expected_exception_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertRaises__expected_exception_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertRaises__expected_exception_as_typed_wrong"
# subject = "unittest.case.TestCase.assertRaises(expected_exception: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertRaises(expected_exception: typed); call it with the wrong type.

typeshed contract: expected_exception is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertRaises(_W(), None)  # expected_exception: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertRegex__text_as_AnyStr_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertRegex__text_as_AnyStr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertRegex__text_as_AnyStr_wrong"
# subject = "unittest.case.TestCase.assertRegex(text: AnyStr)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertRegex(text: AnyStr); call it with the wrong type.

typeshed contract: text is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertRegex(_W(), None)  # text: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertSequenceEqual__seq1_as_Sequence_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertSequenceEqual__seq1_as_Sequence_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertSequenceEqual__seq1_as_Sequence_wrong"
# subject = "unittest.case.TestCase.assertSequenceEqual(seq1: Sequence)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertSequenceEqual(seq1: Sequence); call it with the wrong type.

typeshed contract: seq1 is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertSequenceEqual(_W(), None)  # seq1: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertSetEqual__set1_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertSetEqual__set1_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertSetEqual__set1_as_AbstractSet_wrong"
# subject = "unittest.case.TestCase.assertSetEqual(set1: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertSetEqual(set1: AbstractSet); call it with the wrong type.

typeshed contract: set1 is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertSetEqual(_W(), None)  # set1: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertStartsWith__s_as__SB_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertStartsWith__s_as__SB_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertStartsWith__s_as__SB_wrong"
# subject = "unittest.case.TestCase.assertStartsWith(s: _SB)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertStartsWith(s: _SB); call it with the wrong type.

typeshed contract: s is _SB. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertStartsWith(_W(), None)  # s: _SB <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertTupleEqual__tuple1_as_tuple_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertTupleEqual__tuple1_as_tuple_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertTupleEqual__tuple1_as_tuple_wrong"
# subject = "unittest.case.TestCase.assertTupleEqual(tuple1: tuple)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertTupleEqual(tuple1: tuple); call it with the wrong type.

typeshed contract: tuple1 is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertTupleEqual(12345, None)  # tuple1: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertWarnsRegex__expected_warning_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertWarnsRegex__expected_warning_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertWarnsRegex__expected_warning_as_typed_wrong"
# subject = "unittest.case.TestCase.assertWarnsRegex(expected_warning: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertWarnsRegex(expected_warning: typed); call it with the wrong type.

typeshed contract: expected_warning is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertWarnsRegex(_W(), None, None)  # expected_warning: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__assertWarns__expected_warning_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__assertWarns__expected_warning_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__assertWarns__expected_warning_as_typed_wrong"
# subject = "unittest.case.TestCase.assertWarns(expected_warning: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.assertWarns(expected_warning: typed); call it with the wrong type.

typeshed contract: expected_warning is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.assertWarns(_W(), None)  # expected_warning: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__enterContext__cm_as_AbstractContextManager_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__enterContext__cm_as_AbstractContextManager_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__enterContext__cm_as_AbstractContextManager_wrong"
# subject = "unittest.case.TestCase.enterContext(cm: AbstractContextManager)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.enterContext(cm: AbstractContextManager); call it with the wrong type.

typeshed contract: cm is AbstractContextManager. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.enterContext(_W())  # cm: AbstractContextManager <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__init__methodName_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__init__methodName_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__init__methodName_as_str_wrong"
# subject = "unittest.case.TestCase.__init__(methodName: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.__init__(methodName: str); call it with the wrong type.

typeshed contract: methodName is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.case import TestCase
try:
    TestCase(12345)  # methodName: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/TestCase__run__result_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_TestCase__run__result_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "TestCase__run__result_as_typed_wrong"
# subject = "unittest.case.TestCase.run(result: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.TestCase.run(result: typed); call it with the wrong type.

typeshed contract: result is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import TestCase
obj = object.__new__(TestCase)
try:
    obj.run(_W())  # result: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/addModuleCleanup__function_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_addModuleCleanup__function_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "addModuleCleanup__function_as_Callable_wrong"
# subject = "unittest.case.addModuleCleanup(function: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.addModuleCleanup(function: Callable); call it with the wrong type.

typeshed contract: function is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import addModuleCleanup
try:
    addModuleCleanup(_W())  # function: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/expectedFailure__test_item_as__FT_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_expectedFailure__test_item_as__FT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "expectedFailure__test_item_as__FT_wrong"
# subject = "unittest.case.expectedFailure(test_item: _FT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.expectedFailure(test_item: _FT); call it with the wrong type.

typeshed contract: test_item is _FT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.case import expectedFailure
try:
    expectedFailure(_W())  # test_item: _FT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/skipIf__reason_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_skipIf__reason_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "skipIf__reason_as_str_wrong"
# subject = "unittest.case.skipIf(reason: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.skipIf(reason: str); call it with the wrong type.

typeshed contract: reason is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.case import skipIf
try:
    skipIf(None, 12345)  # reason: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/skipUnless__reason_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_skipUnless__reason_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "skipUnless__reason_as_str_wrong"
# subject = "unittest.case.skipUnless(reason: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.skipUnless(reason: str); call it with the wrong type.

typeshed contract: reason is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.case import skipUnless
try:
    skipUnless(None, 12345)  # reason: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_case/skip__reason_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_case_skip__reason_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_case"
# dimension = "type"
# case = "skip__reason_as_str_wrong"
# subject = "unittest.case.skip(reason: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/case.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.case.skip(reason: str); call it with the wrong type.

typeshed contract: reason is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.case import skip
try:
    skip(12345)  # reason: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
