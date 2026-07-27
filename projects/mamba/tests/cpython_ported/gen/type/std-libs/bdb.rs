use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__break_anywhere__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__break_anywhere__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__break_anywhere__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.break_anywhere(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.break_anywhere(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.break_anywhere(_W())  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__break_here__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__break_here__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__break_here__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.break_here(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.break_here(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.break_here(_W())  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__canonic__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__canonic__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__canonic__filename_as_str_wrong"
# subject = "bdb.Bdb.canonic(filename: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.canonic(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.canonic(12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__clear_all_file_breaks__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__clear_all_file_breaks__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__clear_all_file_breaks__filename_as_str_wrong"
# subject = "bdb.Bdb.clear_all_file_breaks(filename: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.clear_all_file_breaks(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.clear_all_file_breaks(12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__clear_bpbynumber__arg_as_SupportsInt_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__clear_bpbynumber__arg_as_SupportsInt_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__clear_bpbynumber__arg_as_SupportsInt_wrong"
# subject = "bdb.Bdb.clear_bpbynumber(arg: SupportsInt)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.clear_bpbynumber(arg: SupportsInt); call it with the wrong type.

typeshed contract: arg is SupportsInt. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.clear_bpbynumber(_W())  # arg: SupportsInt <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__clear_break__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__clear_break__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__clear_break__filename_as_str_wrong"
# subject = "bdb.Bdb.clear_break(filename: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.clear_break(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.clear_break(12345, 0)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__dispatch_call__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__dispatch_call__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__dispatch_call__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.dispatch_call(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.dispatch_call(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.dispatch_call(_W(), None)  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__dispatch_exception__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__dispatch_exception__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__dispatch_exception__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.dispatch_exception(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.dispatch_exception(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.dispatch_exception(_W(), None)  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__dispatch_line__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__dispatch_line__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__dispatch_line__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.dispatch_line(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.dispatch_line(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.dispatch_line(_W())  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__dispatch_opcode__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__dispatch_opcode__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__dispatch_opcode__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.dispatch_opcode(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.dispatch_opcode(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.dispatch_opcode(_W(), None)  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__dispatch_return__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__dispatch_return__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__dispatch_return__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.dispatch_return(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.dispatch_return(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.dispatch_return(_W(), None)  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__format_stack_entry__frame_lineno_as_tuple_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__format_stack_entry__frame_lineno_as_tuple_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__format_stack_entry__frame_lineno_as_tuple_wrong"
# subject = "bdb.Bdb.format_stack_entry(frame_lineno: tuple)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.format_stack_entry(frame_lineno: tuple); call it with the wrong type.

typeshed contract: frame_lineno is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.format_stack_entry(12345)  # frame_lineno: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__get_bpbynumber__arg_as_SupportsInt_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__get_bpbynumber__arg_as_SupportsInt_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__get_bpbynumber__arg_as_SupportsInt_wrong"
# subject = "bdb.Bdb.get_bpbynumber(arg: SupportsInt)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.get_bpbynumber(arg: SupportsInt); call it with the wrong type.

typeshed contract: arg is SupportsInt. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.get_bpbynumber(_W())  # arg: SupportsInt <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__get_break__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__get_break__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__get_break__filename_as_str_wrong"
# subject = "bdb.Bdb.get_break(filename: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.get_break(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.get_break(12345, 0)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__get_breaks__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__get_breaks__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__get_breaks__filename_as_str_wrong"
# subject = "bdb.Bdb.get_breaks(filename: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.get_breaks(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.get_breaks(12345, 0)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__get_file_breaks__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__get_file_breaks__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__get_file_breaks__filename_as_str_wrong"
# subject = "bdb.Bdb.get_file_breaks(filename: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.get_file_breaks(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.get_file_breaks(12345)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__get_stack__f_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__get_stack__f_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__get_stack__f_as_typed_wrong"
# subject = "bdb.Bdb.get_stack(f: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.get_stack(f: typed); call it with the wrong type.

typeshed contract: f is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.get_stack(_W(), None)  # f: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__init__skip_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__init__skip_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__init__skip_as_typed_wrong"
# subject = "bdb.Bdb.__init__(skip: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.__init__(skip: typed); call it with the wrong type.

typeshed contract: skip is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
try:
    Bdb(_W())  # skip: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__is_skipped_module__module_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__is_skipped_module__module_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__is_skipped_module__module_name_as_str_wrong"
# subject = "bdb.Bdb.is_skipped_module(module_name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.is_skipped_module(module_name: str); call it with the wrong type.

typeshed contract: module_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.is_skipped_module(12345)  # module_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__run__cmd_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__run__cmd_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__run__cmd_as_typed_wrong"
# subject = "bdb.Bdb.run(cmd: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.run(cmd: typed); call it with the wrong type.

typeshed contract: cmd is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
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

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__runcall__func_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__runcall__func_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__runcall__func_as_Callable_wrong"
# subject = "bdb.Bdb.runcall(func: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.runcall(func: Callable); call it with the wrong type.

typeshed contract: func is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.runcall(_W())  # func: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__runctx__cmd_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__runctx__cmd_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__runctx__cmd_as_typed_wrong"
# subject = "bdb.Bdb.runctx(cmd: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.runctx(cmd: typed); call it with the wrong type.

typeshed contract: cmd is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.runctx(_W(), None, None)  # cmd: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__runeval__expr_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__runeval__expr_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__runeval__expr_as_typed_wrong"
# subject = "bdb.Bdb.runeval(expr: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.runeval(expr: typed); call it with the wrong type.

typeshed contract: expr is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.runeval(_W())  # expr: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__set_break__filename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__set_break__filename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__set_break__filename_as_str_wrong"
# subject = "bdb.Bdb.set_break(filename: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.set_break(filename: str); call it with the wrong type.

typeshed contract: filename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.set_break(12345, 0)  # filename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__set_enterframe__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__set_enterframe__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__set_enterframe__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.set_enterframe(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.set_enterframe(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.set_enterframe(_W())  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__set_next__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__set_next__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__set_next__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.set_next(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.set_next(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.set_next(_W())  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__set_return__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__set_return__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__set_return__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.set_return(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.set_return(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.set_return(_W())  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__set_trace__frame_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__set_trace__frame_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__set_trace__frame_as_typed_wrong"
# subject = "bdb.Bdb.set_trace(frame: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.set_trace(frame: typed); call it with the wrong type.

typeshed contract: frame is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.set_trace(_W())  # frame: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__set_until__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__set_until__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__set_until__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.set_until(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.set_until(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.set_until(_W())  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__stop_here__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__stop_here__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__stop_here__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.stop_here(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.stop_here(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.stop_here(_W())  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__trace_dispatch__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__trace_dispatch__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__trace_dispatch__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.trace_dispatch(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.trace_dispatch(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.trace_dispatch(_W(), "", None)  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__user_call__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__user_call__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__user_call__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.user_call(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.user_call(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.user_call(_W(), None)  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__user_exception__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__user_exception__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__user_exception__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.user_exception(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.user_exception(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.user_exception(_W(), None)  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__user_line__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__user_line__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__user_line__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.user_line(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.user_line(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.user_line(_W())  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__user_opcode__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__user_opcode__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__user_opcode__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.user_opcode(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.user_opcode(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.user_opcode(_W())  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Bdb__user_return__frame_as_FrameType_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Bdb__user_return__frame_as_FrameType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Bdb__user_return__frame_as_FrameType_wrong"
# subject = "bdb.Bdb.user_return(frame: FrameType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Bdb.user_return(frame: FrameType); call it with the wrong type.

typeshed contract: frame is FrameType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Bdb
obj = object.__new__(Bdb)
try:
    obj.user_return(_W(), None)  # frame: FrameType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Breakpoint__bpprint__out_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Breakpoint__bpprint__out_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Breakpoint__bpprint__out_as_typed_wrong"
# subject = "bdb.Breakpoint.bpprint(out: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Breakpoint.bpprint(out: typed); call it with the wrong type.

typeshed contract: out is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import Breakpoint
obj = object.__new__(Breakpoint)
try:
    obj.bpprint(_W())  # out: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/Breakpoint__init__file_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_Breakpoint__init__file_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "Breakpoint__init__file_as_str_wrong"
# subject = "bdb.Breakpoint.__init__(file: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.Breakpoint.__init__(file: str); call it with the wrong type.

typeshed contract: file is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from bdb import Breakpoint
try:
    Breakpoint(12345, 0)  # file: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/checkfuncname__b_as_Breakpoint_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_checkfuncname__b_as_Breakpoint_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "checkfuncname__b_as_Breakpoint_wrong"
# subject = "bdb.checkfuncname(b: Breakpoint)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.checkfuncname(b: Breakpoint); call it with the wrong type.

typeshed contract: b is Breakpoint. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bdb import checkfuncname
try:
    checkfuncname(_W(), None)  # b: Breakpoint <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/bdb/effective__file_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_bdb_effective__file_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "type"
# case = "effective__file_as_str_wrong"
# subject = "bdb.effective(file: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bdb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bdb.effective(file: str); call it with the wrong type.

typeshed contract: file is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from bdb import effective
try:
    effective(12345, 0, None)  # file: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
