use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_lsprof/Profiler__enable__subcalls_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs__lsprof_Profiler__enable__subcalls_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_lsprof"
# dimension = "type"
# case = "Profiler__enable__subcalls_as_bool_wrong"
# subject = "_lsprof.Profiler.enable(subcalls: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_lsprof.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _lsprof.Profiler.enable(subcalls: bool); call it with the wrong type.

typeshed contract: subcalls is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _lsprof import Profiler
obj = object.__new__(Profiler)
try:
    obj.enable("not_a_bool")  # subcalls: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_lsprof/Profiler__init__timer_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__lsprof_Profiler__init__timer_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_lsprof"
# dimension = "type"
# case = "Profiler__init__timer_as_typed_wrong"
# subject = "_lsprof.Profiler.__init__(timer: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_lsprof.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _lsprof.Profiler.__init__(timer: typed); call it with the wrong type.

typeshed contract: timer is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _lsprof import Profiler
try:
    Profiler(_W())  # timer: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
