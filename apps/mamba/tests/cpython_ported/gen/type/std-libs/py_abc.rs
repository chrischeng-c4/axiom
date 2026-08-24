use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_py_abc/ABCMeta____new____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__py_abc_ABCMeta____new____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_py_abc"
# dimension = "type"
# case = "ABCMeta____new____name_as_str_wrong"
# subject = "_py_abc.ABCMeta.__new__(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_py_abc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _py_abc.ABCMeta.__new__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _py_abc import ABCMeta
obj = object.__new__(ABCMeta)
try:
    obj.__new__(12345, None, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_py_abc/ABCMeta__register__subclass_as_type_wrong.py`.
#[test]
fn test_gen_type_std_libs__py_abc_ABCMeta__register__subclass_as_type_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_py_abc"
# dimension = "type"
# case = "ABCMeta__register__subclass_as_type_wrong"
# subject = "_py_abc.ABCMeta.register(subclass: type)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_py_abc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _py_abc.ABCMeta.register(subclass: type); call it with the wrong type.

typeshed contract: subclass is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _py_abc import ABCMeta
obj = object.__new__(ABCMeta)
try:
    obj.register(_W())  # subclass: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
