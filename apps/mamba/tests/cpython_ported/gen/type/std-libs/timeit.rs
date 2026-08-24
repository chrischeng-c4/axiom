use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/timeit/Timer__init__stmt_as__Stmt_wrong.py`.
#[test]
fn test_gen_type_std_libs_timeit_Timer__init__stmt_as__Stmt_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "type"
# case = "Timer__init__stmt_as__Stmt_wrong"
# subject = "timeit.Timer.__init__(stmt: _Stmt)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/timeit.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: timeit.Timer.__init__(stmt: _Stmt); call it with the wrong type.

typeshed contract: stmt is _Stmt. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from timeit import Timer
try:
    Timer(_W())  # stmt: _Stmt <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/timeit/Timer__print_exc__file_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_timeit_Timer__print_exc__file_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "type"
# case = "Timer__print_exc__file_as_typed_wrong"
# subject = "timeit.Timer.print_exc(file: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/timeit.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: timeit.Timer.print_exc(file: typed); call it with the wrong type.

typeshed contract: file is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from timeit import Timer
obj = object.__new__(Timer)
try:
    obj.print_exc(_W())  # file: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/timeit/Timer__repeat__repeat_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_timeit_Timer__repeat__repeat_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "type"
# case = "Timer__repeat__repeat_as_int_wrong"
# subject = "timeit.Timer.repeat(repeat: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/timeit.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: timeit.Timer.repeat(repeat: int); call it with the wrong type.

typeshed contract: repeat is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from timeit import Timer
obj = object.__new__(Timer)
try:
    obj.repeat("not_an_int")  # repeat: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/timeit/Timer__timeit__number_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_timeit_Timer__timeit__number_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "type"
# case = "Timer__timeit__number_as_int_wrong"
# subject = "timeit.Timer.timeit(number: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/timeit.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: timeit.Timer.timeit(number: int); call it with the wrong type.

typeshed contract: number is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from timeit import Timer
obj = object.__new__(Timer)
try:
    obj.timeit("not_an_int")  # number: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/timeit/repeat__stmt_as__Stmt_wrong.py`.
#[test]
fn test_gen_type_std_libs_timeit_repeat__stmt_as__Stmt_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "type"
# case = "repeat__stmt_as__Stmt_wrong"
# subject = "timeit.repeat(stmt: _Stmt)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/timeit.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: timeit.repeat(stmt: _Stmt); call it with the wrong type.

typeshed contract: stmt is _Stmt. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from timeit import repeat
try:
    repeat(_W())  # stmt: _Stmt <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/timeit/timeit__stmt_as__Stmt_wrong.py`.
#[test]
fn test_gen_type_std_libs_timeit_timeit__stmt_as__Stmt_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeit"
# dimension = "type"
# case = "timeit__stmt_as__Stmt_wrong"
# subject = "timeit.timeit(stmt: _Stmt)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/timeit.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: timeit.timeit(stmt: _Stmt); call it with the wrong type.

typeshed contract: stmt is _Stmt. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from timeit import timeit
try:
    timeit(_W())  # stmt: _Stmt <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
