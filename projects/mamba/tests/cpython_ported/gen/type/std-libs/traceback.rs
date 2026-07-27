use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/traceback/FrameSummary__init__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_FrameSummary__init__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "FrameSummary__init__filename_as_str_wrong"
# subject = "traceback.FrameSummary.__init__(filename: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.FrameSummary.__init__(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from traceback import FrameSummary
try:
    FrameSummary(12345, None, "")  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/StackSummary__extract__frame_gen_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_StackSummary__extract__frame_gen_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "StackSummary__extract__frame_gen_as_Iterable_wrong"
# subject = "traceback.StackSummary.extract(frame_gen: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.StackSummary.extract(frame_gen: Iterable); call it with the wrong type.

typeshed contract: frame_gen is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import StackSummary
try:
    StackSummary.extract(_W())  # frame_gen: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/StackSummary__format_frame_summary__frame_summary_as_FrameSummary_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_StackSummary__format_frame_summary__frame_summary_as_FrameSummary_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "StackSummary__format_frame_summary__frame_summary_as_FrameSummary_wrong"
# subject = "traceback.StackSummary.format_frame_summary(frame_summary: FrameSummary)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.StackSummary.format_frame_summary(frame_summary: FrameSummary); call it with the wrong type.

typeshed contract: frame_summary is FrameSummary. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import StackSummary
obj = object.__new__(StackSummary)
try:
    obj.format_frame_summary(_W())  # frame_summary: FrameSummary <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/StackSummary__from_list__a_list_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_StackSummary__from_list__a_list_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "StackSummary__from_list__a_list_as_Iterable_wrong"
# subject = "traceback.StackSummary.from_list(a_list: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.StackSummary.from_list(a_list: Iterable); call it with the wrong type.

typeshed contract: a_list is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import StackSummary
try:
    StackSummary.from_list(_W())  # a_list: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/TracebackException__from_exception__exc_as_BaseException_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_TracebackException__from_exception__exc_as_BaseException_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "TracebackException__from_exception__exc_as_BaseException_wrong"
# subject = "traceback.TracebackException.from_exception(exc: BaseException)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.TracebackException.from_exception(exc: BaseException); call it with the wrong type.

typeshed contract: exc is BaseException. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import TracebackException
try:
    TracebackException.from_exception(_W())  # exc: BaseException <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/clear_frames__tb_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_clear_frames__tb_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "clear_frames__tb_as_typed_wrong"
# subject = "traceback.clear_frames(tb: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.clear_frames(tb: typed); call it with the wrong type.

typeshed contract: tb is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import clear_frames
try:
    clear_frames(_W())  # tb: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/extract_stack__f_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_extract_stack__f_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "extract_stack__f_as_typed_wrong"
# subject = "traceback.extract_stack(f: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.extract_stack(f: typed); call it with the wrong type.

typeshed contract: f is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import extract_stack
try:
    extract_stack(_W())  # f: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/extract_tb__tb_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_extract_tb__tb_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "extract_tb__tb_as_typed_wrong"
# subject = "traceback.extract_tb(tb: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.extract_tb(tb: typed); call it with the wrong type.

typeshed contract: tb is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import extract_tb
try:
    extract_tb(_W())  # tb: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/format_exc__limit_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_format_exc__limit_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "format_exc__limit_as_typed_wrong"
# subject = "traceback.format_exc(limit: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.format_exc(limit: typed); call it with the wrong type.

typeshed contract: limit is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import format_exc
try:
    format_exc(_W())  # limit: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/format_list__extracted_list_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_format_list__extracted_list_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "format_list__extracted_list_as_Iterable_wrong"
# subject = "traceback.format_list(extracted_list: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.format_list(extracted_list: Iterable); call it with the wrong type.

typeshed contract: extracted_list is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import format_list
try:
    format_list(_W())  # extracted_list: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/format_stack__f_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_format_stack__f_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "format_stack__f_as_typed_wrong"
# subject = "traceback.format_stack(f: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.format_stack(f: typed); call it with the wrong type.

typeshed contract: f is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import format_stack
try:
    format_stack(_W())  # f: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/format_tb__tb_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_format_tb__tb_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "format_tb__tb_as_typed_wrong"
# subject = "traceback.format_tb(tb: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.format_tb(tb: typed); call it with the wrong type.

typeshed contract: tb is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import format_tb
try:
    format_tb(_W())  # tb: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/print_exc__limit_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_print_exc__limit_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "print_exc__limit_as_typed_wrong"
# subject = "traceback.print_exc(limit: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.print_exc(limit: typed); call it with the wrong type.

typeshed contract: limit is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import print_exc
try:
    print_exc(_W())  # limit: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/print_last__limit_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_print_last__limit_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "print_last__limit_as_typed_wrong"
# subject = "traceback.print_last(limit: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.print_last(limit: typed); call it with the wrong type.

typeshed contract: limit is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import print_last
try:
    print_last(_W())  # limit: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/print_list__extracted_list_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_print_list__extracted_list_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "print_list__extracted_list_as_Iterable_wrong"
# subject = "traceback.print_list(extracted_list: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.print_list(extracted_list: Iterable); call it with the wrong type.

typeshed contract: extracted_list is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import print_list
try:
    print_list(_W())  # extracted_list: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/print_stack__f_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_print_stack__f_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "print_stack__f_as_typed_wrong"
# subject = "traceback.print_stack(f: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.print_stack(f: typed); call it with the wrong type.

typeshed contract: f is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import print_stack
try:
    print_stack(_W())  # f: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/print_tb__tb_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_print_tb__tb_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "print_tb__tb_as_typed_wrong"
# subject = "traceback.print_tb(tb: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.print_tb(tb: typed); call it with the wrong type.

typeshed contract: tb is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import print_tb
try:
    print_tb(_W())  # tb: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/walk_stack__f_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_walk_stack__f_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "walk_stack__f_as_typed_wrong"
# subject = "traceback.walk_stack(f: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.walk_stack(f: typed); call it with the wrong type.

typeshed contract: f is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import walk_stack
try:
    walk_stack(_W())  # f: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/traceback/walk_tb__tb_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_traceback_walk_tb__tb_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "type"
# case = "walk_tb__tb_as_typed_wrong"
# subject = "traceback.walk_tb(tb: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/traceback.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: traceback.walk_tb(tb: typed); call it with the wrong type.

typeshed contract: tb is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from traceback import walk_tb
try:
    walk_tb(_W())  # tb: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
