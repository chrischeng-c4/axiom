use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_tracemalloc/start__nframe_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__tracemalloc_start__nframe_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_tracemalloc"
# dimension = "type"
# case = "start__nframe_as_int_wrong"
# subject = "_tracemalloc.start(nframe: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _tracemalloc.start(nframe: int); call it with the wrong type.

typeshed contract: nframe is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _tracemalloc import start
try:
    start("not_an_int")  # nframe: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Frame____ge____other_as_Frame_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Frame____ge____other_as_Frame_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Frame____ge____other_as_Frame_wrong"
# subject = "tracemalloc.Frame.__ge__(other: Frame)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Frame.__ge__(other: Frame); call it with the wrong type.

typeshed contract: other is Frame. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Frame
obj = object.__new__(Frame)
try:
    obj.__ge__(_W())  # other: Frame <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Frame____gt____other_as_Frame_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Frame____gt____other_as_Frame_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Frame____gt____other_as_Frame_wrong"
# subject = "tracemalloc.Frame.__gt__(other: Frame)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Frame.__gt__(other: Frame); call it with the wrong type.

typeshed contract: other is Frame. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Frame
obj = object.__new__(Frame)
try:
    obj.__gt__(_W())  # other: Frame <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Frame____le____other_as_Frame_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Frame____le____other_as_Frame_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Frame____le____other_as_Frame_wrong"
# subject = "tracemalloc.Frame.__le__(other: Frame)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Frame.__le__(other: Frame); call it with the wrong type.

typeshed contract: other is Frame. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Frame
obj = object.__new__(Frame)
try:
    obj.__le__(_W())  # other: Frame <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Frame____lt____other_as_Frame_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Frame____lt____other_as_Frame_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Frame____lt____other_as_Frame_wrong"
# subject = "tracemalloc.Frame.__lt__(other: Frame)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Frame.__lt__(other: Frame); call it with the wrong type.

typeshed contract: other is Frame. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Frame
obj = object.__new__(Frame)
try:
    obj.__lt__(_W())  # other: Frame <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Frame__init__frame_as__FrameTuple_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Frame__init__frame_as__FrameTuple_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Frame__init__frame_as__FrameTuple_wrong"
# subject = "tracemalloc.Frame.__init__(frame: _FrameTuple)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Frame.__init__(frame: _FrameTuple); call it with the wrong type.

typeshed contract: frame is _FrameTuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Frame
try:
    Frame(_W())  # frame: _FrameTuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Snapshot__compare_to__old_snapshot_as_Snapshot_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Snapshot__compare_to__old_snapshot_as_Snapshot_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Snapshot__compare_to__old_snapshot_as_Snapshot_wrong"
# subject = "tracemalloc.Snapshot.compare_to(old_snapshot: Snapshot)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Snapshot.compare_to(old_snapshot: Snapshot); call it with the wrong type.

typeshed contract: old_snapshot is Snapshot. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Snapshot
obj = object.__new__(Snapshot)
try:
    obj.compare_to(_W(), "")  # old_snapshot: Snapshot <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Snapshot__dump__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Snapshot__dump__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Snapshot__dump__filename_as_str_wrong"
# subject = "tracemalloc.Snapshot.dump(filename: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Snapshot.dump(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tracemalloc import Snapshot
obj = object.__new__(Snapshot)
try:
    obj.dump(12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Snapshot__load__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Snapshot__load__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Snapshot__load__filename_as_str_wrong"
# subject = "tracemalloc.Snapshot.load(filename: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Snapshot.load(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tracemalloc import Snapshot
try:
    Snapshot.load(12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Snapshot__statistics__key_type_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Snapshot__statistics__key_type_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Snapshot__statistics__key_type_as_str_wrong"
# subject = "tracemalloc.Snapshot.statistics(key_type: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Snapshot.statistics(key_type: str); call it with the wrong type.

typeshed contract: key_type is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tracemalloc import Snapshot
obj = object.__new__(Snapshot)
try:
    obj.statistics(12345)  # key_type: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/StatisticDiff__init__traceback_as_Traceback_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_StatisticDiff__init__traceback_as_Traceback_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "StatisticDiff__init__traceback_as_Traceback_wrong"
# subject = "tracemalloc.StatisticDiff.__init__(traceback: Traceback)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.StatisticDiff.__init__(traceback: Traceback); call it with the wrong type.

typeshed contract: traceback is Traceback. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import StatisticDiff
try:
    StatisticDiff(_W(), 0, 0, 0, 0)  # traceback: Traceback <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Statistic__init__traceback_as_Traceback_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Statistic__init__traceback_as_Traceback_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Statistic__init__traceback_as_Traceback_wrong"
# subject = "tracemalloc.Statistic.__init__(traceback: Traceback)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Statistic.__init__(traceback: Traceback); call it with the wrong type.

typeshed contract: traceback is Traceback. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Statistic
try:
    Statistic(_W(), 0, 0)  # traceback: Traceback <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Trace__init__trace_as__TraceTuple_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Trace__init__trace_as__TraceTuple_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Trace__init__trace_as__TraceTuple_wrong"
# subject = "tracemalloc.Trace.__init__(trace: _TraceTuple)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Trace.__init__(trace: _TraceTuple); call it with the wrong type.

typeshed contract: trace is _TraceTuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Trace
try:
    Trace(_W())  # trace: _TraceTuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Traceback____contains____frame_as_Frame_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Traceback____contains____frame_as_Frame_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Traceback____contains____frame_as_Frame_wrong"
# subject = "tracemalloc.Traceback.__contains__(frame: Frame)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Traceback.__contains__(frame: Frame); call it with the wrong type.

typeshed contract: frame is Frame. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Traceback
obj = object.__new__(Traceback)
try:
    obj.__contains__(_W())  # frame: Frame <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Traceback____ge____other_as_Traceback_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Traceback____ge____other_as_Traceback_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Traceback____ge____other_as_Traceback_wrong"
# subject = "tracemalloc.Traceback.__ge__(other: Traceback)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Traceback.__ge__(other: Traceback); call it with the wrong type.

typeshed contract: other is Traceback. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Traceback
obj = object.__new__(Traceback)
try:
    obj.__ge__(_W())  # other: Traceback <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Traceback____gt____other_as_Traceback_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Traceback____gt____other_as_Traceback_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Traceback____gt____other_as_Traceback_wrong"
# subject = "tracemalloc.Traceback.__gt__(other: Traceback)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Traceback.__gt__(other: Traceback); call it with the wrong type.

typeshed contract: other is Traceback. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Traceback
obj = object.__new__(Traceback)
try:
    obj.__gt__(_W())  # other: Traceback <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Traceback____le____other_as_Traceback_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Traceback____le____other_as_Traceback_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Traceback____le____other_as_Traceback_wrong"
# subject = "tracemalloc.Traceback.__le__(other: Traceback)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Traceback.__le__(other: Traceback); call it with the wrong type.

typeshed contract: other is Traceback. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Traceback
obj = object.__new__(Traceback)
try:
    obj.__le__(_W())  # other: Traceback <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Traceback____lt____other_as_Traceback_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Traceback____lt____other_as_Traceback_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Traceback____lt____other_as_Traceback_wrong"
# subject = "tracemalloc.Traceback.__lt__(other: Traceback)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Traceback.__lt__(other: Traceback); call it with the wrong type.

typeshed contract: other is Traceback. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Traceback
obj = object.__new__(Traceback)
try:
    obj.__lt__(_W())  # other: Traceback <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tracemalloc/Traceback__format__limit_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_tracemalloc_Traceback__format__limit_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "type"
# case = "Traceback__format__limit_as_typed_wrong"
# subject = "tracemalloc.Traceback.format(limit: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tracemalloc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tracemalloc.Traceback.format(limit: typed); call it with the wrong type.

typeshed contract: limit is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tracemalloc import Traceback
obj = object.__new__(Traceback)
try:
    obj.format(_W())  # limit: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
