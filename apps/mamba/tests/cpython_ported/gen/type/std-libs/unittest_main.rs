use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/unittest_main/TestProgram__init__module_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_main_TestProgram__init__module_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_main"
# dimension = "type"
# case = "TestProgram__init__module_as_typed_wrong"
# subject = "unittest.main.TestProgram.__init__(module: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/main.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.main.TestProgram.__init__(module: typed); call it with the wrong type.

typeshed contract: module is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.main import TestProgram
try:
    TestProgram(_W())  # module: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
