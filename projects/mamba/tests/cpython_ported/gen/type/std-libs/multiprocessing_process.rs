use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/multiprocessing_process/BaseProcess__init__group_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_process_BaseProcess__init__group_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_process"
# dimension = "type"
# case = "BaseProcess__init__group_as_typed_wrong"
# subject = "multiprocessing.process.BaseProcess.__init__(group: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/process.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.process.BaseProcess.__init__(group: typed); call it with the wrong type.

typeshed contract: group is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.process import BaseProcess
try:
    BaseProcess(_W())  # group: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_process/BaseProcess__join__timeout_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_process_BaseProcess__join__timeout_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_process"
# dimension = "type"
# case = "BaseProcess__join__timeout_as_typed_wrong"
# subject = "multiprocessing.process.BaseProcess.join(timeout: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/process.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.process.BaseProcess.join(timeout: typed); call it with the wrong type.

typeshed contract: timeout is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.process import BaseProcess
obj = object.__new__(BaseProcess)
try:
    obj.join(_W())  # timeout: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
