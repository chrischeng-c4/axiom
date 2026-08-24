use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/trace/CoverageResults__is_ignored_filename__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_trace_CoverageResults__is_ignored_filename__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "CoverageResults__is_ignored_filename__filename_as_str_wrong"
# subject = "trace.CoverageResults.is_ignored_filename(filename: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: trace.CoverageResults.is_ignored_filename(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from trace import CoverageResults
obj = object.__new__(CoverageResults)
try:
    obj.is_ignored_filename(12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/trace/CoverageResults__update__other_as_CoverageResults_wrong.py`.
#[test]
fn test_gen_type_std_libs_trace_CoverageResults__update__other_as_CoverageResults_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "CoverageResults__update__other_as_CoverageResults_wrong"
# subject = "trace.CoverageResults.update(other: CoverageResults)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: trace.CoverageResults.update(other: CoverageResults); call it with the wrong type.

typeshed contract: other is CoverageResults. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from trace import CoverageResults
obj = object.__new__(CoverageResults)
try:
    obj.update(_W())  # other: CoverageResults <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/trace/CoverageResults__write_results_file__path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_trace_CoverageResults__write_results_file__path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "CoverageResults__write_results_file__path_as_StrPath_wrong"
# subject = "trace.CoverageResults.write_results_file(path: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: trace.CoverageResults.write_results_file(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from trace import CoverageResults
obj = object.__new__(CoverageResults)
try:
    obj.write_results_file(_W(), None, None, None)  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/trace/Trace__file_module_function_of__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_trace_Trace__file_module_function_of__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "Trace__file_module_function_of__frame_as_FrameType_wrong"
# subject = "trace.Trace.file_module_function_of(frame: FrameType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: trace.Trace.file_module_function_of(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from trace import Trace
obj = object.__new__(Trace)
try:
    obj.file_module_function_of(_W())  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/trace/Trace__globaltrace_countfuncs__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_trace_Trace__globaltrace_countfuncs__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "Trace__globaltrace_countfuncs__frame_as_FrameType_wrong"
# subject = "trace.Trace.globaltrace_countfuncs(frame: FrameType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: trace.Trace.globaltrace_countfuncs(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from trace import Trace
obj = object.__new__(Trace)
try:
    obj.globaltrace_countfuncs(_W(), "", None)  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/trace/Trace__globaltrace_lt__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_trace_Trace__globaltrace_lt__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "Trace__globaltrace_lt__frame_as_FrameType_wrong"
# subject = "trace.Trace.globaltrace_lt(frame: FrameType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: trace.Trace.globaltrace_lt(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from trace import Trace
obj = object.__new__(Trace)
try:
    obj.globaltrace_lt(_W(), "", None)  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/trace/Trace__globaltrace_trackcallers__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_trace_Trace__globaltrace_trackcallers__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "Trace__globaltrace_trackcallers__frame_as_FrameType_wrong"
# subject = "trace.Trace.globaltrace_trackcallers(frame: FrameType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: trace.Trace.globaltrace_trackcallers(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from trace import Trace
obj = object.__new__(Trace)
try:
    obj.globaltrace_trackcallers(_W(), "", None)  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/trace/Trace__init__count_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_trace_Trace__init__count_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "Trace__init__count_as_int_wrong"
# subject = "trace.Trace.__init__(count: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: trace.Trace.__init__(count: int); call it with the wrong type.

typeshed contract: count is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from trace import Trace
try:
    Trace("not_an_int")  # count: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/trace/Trace__localtrace_count__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_trace_Trace__localtrace_count__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "Trace__localtrace_count__frame_as_FrameType_wrong"
# subject = "trace.Trace.localtrace_count(frame: FrameType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: trace.Trace.localtrace_count(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from trace import Trace
obj = object.__new__(Trace)
try:
    obj.localtrace_count(_W(), "", None)  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/trace/Trace__localtrace_trace__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_trace_Trace__localtrace_trace__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "Trace__localtrace_trace__frame_as_FrameType_wrong"
# subject = "trace.Trace.localtrace_trace(frame: FrameType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: trace.Trace.localtrace_trace(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from trace import Trace
obj = object.__new__(Trace)
try:
    obj.localtrace_trace(_W(), "", None)  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/trace/Trace__localtrace_trace_and_count__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_trace_Trace__localtrace_trace_and_count__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "Trace__localtrace_trace_and_count__frame_as_FrameType_wrong"
# subject = "trace.Trace.localtrace_trace_and_count(frame: FrameType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: trace.Trace.localtrace_trace_and_count(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from trace import Trace
obj = object.__new__(Trace)
try:
    obj.localtrace_trace_and_count(_W(), "", None)  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/trace/Trace__run__cmd_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_trace_Trace__run__cmd_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "Trace__run__cmd_as_typed_wrong"
# subject = "trace.Trace.run(cmd: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: trace.Trace.run(cmd: typed); call it with the wrong type.

typeshed contract: cmd is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from trace import Trace
obj = object.__new__(Trace)
try:
    obj.run(_W())  # cmd: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/trace/Trace__runctx__cmd_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_trace_Trace__runctx__cmd_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "type"
# case = "Trace__runctx__cmd_as_typed_wrong"
# subject = "trace.Trace.runctx(cmd: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/trace.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: trace.Trace.runctx(cmd: typed); call it with the wrong type.

typeshed contract: cmd is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from trace import Trace
obj = object.__new__(Trace)
try:
    obj.runctx(_W())  # cmd: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
