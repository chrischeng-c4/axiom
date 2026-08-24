use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_remote_debugging/BinaryReader__init__filename_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs__remote_debugging_BinaryReader__init__filename_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_remote_debugging"
# dimension = "type"
# case = "BinaryReader__init__filename_as_StrOrBytesPath_wrong"
# subject = "_remote_debugging.BinaryReader.__init__(filename: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_remote_debugging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _remote_debugging.BinaryReader.__init__(filename: StrOrBytesPath); call it with the wrong type.

typeshed contract: filename is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _remote_debugging import BinaryReader
try:
    BinaryReader(_W())  # filename: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_remote_debugging/BinaryReader__replay__progress_callback_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__remote_debugging_BinaryReader__replay__progress_callback_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_remote_debugging"
# dimension = "type"
# case = "BinaryReader__replay__progress_callback_as_typed_wrong"
# subject = "_remote_debugging.BinaryReader.replay(progress_callback: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_remote_debugging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _remote_debugging.BinaryReader.replay(progress_callback: typed); call it with the wrong type.

typeshed contract: progress_callback is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _remote_debugging import BinaryReader
obj = object.__new__(BinaryReader)
try:
    obj.replay(None, _W())  # progress_callback: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_remote_debugging/BinaryWriter__init__filename_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs__remote_debugging_BinaryWriter__init__filename_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_remote_debugging"
# dimension = "type"
# case = "BinaryWriter__init__filename_as_StrOrBytesPath_wrong"
# subject = "_remote_debugging.BinaryWriter.__init__(filename: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_remote_debugging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _remote_debugging.BinaryWriter.__init__(filename: StrOrBytesPath); call it with the wrong type.

typeshed contract: filename is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _remote_debugging import BinaryWriter
try:
    BinaryWriter(_W(), 0, 0)  # filename: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_remote_debugging/BinaryWriter__write_sample__stack_frames_as_list_wrong.py`.
#[test]
fn test_gen_type_std_libs__remote_debugging_BinaryWriter__write_sample__stack_frames_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_remote_debugging"
# dimension = "type"
# case = "BinaryWriter__write_sample__stack_frames_as_list_wrong"
# subject = "_remote_debugging.BinaryWriter.write_sample(stack_frames: list)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_remote_debugging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _remote_debugging.BinaryWriter.write_sample(stack_frames: list); call it with the wrong type.

typeshed contract: stack_frames is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _remote_debugging import BinaryWriter
obj = object.__new__(BinaryWriter)
try:
    obj.write_sample(12345, 0)  # stack_frames: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_remote_debugging/GCMonitor__get_gc_stats__all_interpreters_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs__remote_debugging_GCMonitor__get_gc_stats__all_interpreters_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_remote_debugging"
# dimension = "type"
# case = "GCMonitor__get_gc_stats__all_interpreters_as_bool_wrong"
# subject = "_remote_debugging.GCMonitor.get_gc_stats(all_interpreters: bool)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_remote_debugging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _remote_debugging.GCMonitor.get_gc_stats(all_interpreters: bool); call it with the wrong type.

typeshed contract: all_interpreters is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _remote_debugging import GCMonitor
obj = object.__new__(GCMonitor)
try:
    obj.get_gc_stats("not_a_bool")  # all_interpreters: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_remote_debugging/GCMonitor__init__pid_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__remote_debugging_GCMonitor__init__pid_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_remote_debugging"
# dimension = "type"
# case = "GCMonitor__init__pid_as_int_wrong"
# subject = "_remote_debugging.GCMonitor.__init__(pid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_remote_debugging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _remote_debugging.GCMonitor.__init__(pid: int); call it with the wrong type.

typeshed contract: pid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _remote_debugging import GCMonitor
try:
    GCMonitor("not_an_int")  # pid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_remote_debugging/RemoteUnwinder__init__pid_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__remote_debugging_RemoteUnwinder__init__pid_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_remote_debugging"
# dimension = "type"
# case = "RemoteUnwinder__init__pid_as_int_wrong"
# subject = "_remote_debugging.RemoteUnwinder.__init__(pid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_remote_debugging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _remote_debugging.RemoteUnwinder.__init__(pid: int); call it with the wrong type.

typeshed contract: pid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _remote_debugging import RemoteUnwinder
try:
    RemoteUnwinder("not_an_int")  # pid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_remote_debugging/get_child_pids__pid_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__remote_debugging_get_child_pids__pid_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_remote_debugging"
# dimension = "type"
# case = "get_child_pids__pid_as_int_wrong"
# subject = "_remote_debugging.get_child_pids(pid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_remote_debugging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _remote_debugging.get_child_pids(pid: int); call it with the wrong type.

typeshed contract: pid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _remote_debugging import get_child_pids
try:
    get_child_pids("not_an_int")  # pid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_remote_debugging/get_gc_stats__pid_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__remote_debugging_get_gc_stats__pid_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_remote_debugging"
# dimension = "type"
# case = "get_gc_stats__pid_as_int_wrong"
# subject = "_remote_debugging.get_gc_stats(pid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_remote_debugging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _remote_debugging.get_gc_stats(pid: int); call it with the wrong type.

typeshed contract: pid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _remote_debugging import get_gc_stats
try:
    get_gc_stats("not_an_int")  # pid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_remote_debugging/is_python_process__pid_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__remote_debugging_is_python_process__pid_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_remote_debugging"
# dimension = "type"
# case = "is_python_process__pid_as_int_wrong"
# subject = "_remote_debugging.is_python_process(pid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_remote_debugging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _remote_debugging.is_python_process(pid: int); call it with the wrong type.

typeshed contract: pid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _remote_debugging import is_python_process
try:
    is_python_process("not_an_int")  # pid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
