use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/unittest_signals/registerResult__result_as_TestResult_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_signals_registerResult__result_as_TestResult_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_signals"
# dimension = "type"
# case = "registerResult__result_as_TestResult_wrong"
# subject = "unittest.signals.registerResult(result: TestResult)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/signals.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.signals.registerResult(result: TestResult); call it with the wrong type.

typeshed contract: result is TestResult. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.signals import registerResult
try:
    registerResult(_W())  # result: TestResult <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_signals/removeResult__result_as_TestResult_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_signals_removeResult__result_as_TestResult_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_signals"
# dimension = "type"
# case = "removeResult__result_as_TestResult_wrong"
# subject = "unittest.signals.removeResult(result: TestResult)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/signals.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.signals.removeResult(result: TestResult); call it with the wrong type.

typeshed contract: result is TestResult. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.signals import removeResult
try:
    removeResult(_W())  # result: TestResult <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
