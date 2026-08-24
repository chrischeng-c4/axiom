use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/multiprocessing_popen_fork/Popen__duplicate_for_child__fd_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_popen_fork_Popen__duplicate_for_child__fd_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_popen_fork"
# dimension = "type"
# case = "Popen__duplicate_for_child__fd_as_int_wrong"
# subject = "multiprocessing.popen_fork.Popen.duplicate_for_child(fd: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/popen_fork.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.popen_fork.Popen.duplicate_for_child(fd: int); call it with the wrong type.

typeshed contract: fd is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.popen_fork import Popen
obj = object.__new__(Popen)
try:
    obj.duplicate_for_child("not_an_int")  # fd: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_popen_fork/Popen__init__process_obj_as_BaseProcess_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_popen_fork_Popen__init__process_obj_as_BaseProcess_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_popen_fork"
# dimension = "type"
# case = "Popen__init__process_obj_as_BaseProcess_wrong"
# subject = "multiprocessing.popen_fork.Popen.__init__(process_obj: BaseProcess)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/popen_fork.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.popen_fork.Popen.__init__(process_obj: BaseProcess); call it with the wrong type.

typeshed contract: process_obj is BaseProcess. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.popen_fork import Popen
try:
    Popen(_W())  # process_obj: BaseProcess <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_popen_fork/Popen__poll__flag_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_popen_fork_Popen__poll__flag_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_popen_fork"
# dimension = "type"
# case = "Popen__poll__flag_as_int_wrong"
# subject = "multiprocessing.popen_fork.Popen.poll(flag: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/popen_fork.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.popen_fork.Popen.poll(flag: int); call it with the wrong type.

typeshed contract: flag is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.popen_fork import Popen
obj = object.__new__(Popen)
try:
    obj.poll("not_an_int")  # flag: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_popen_fork/Popen__wait__timeout_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_popen_fork_Popen__wait__timeout_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_popen_fork"
# dimension = "type"
# case = "Popen__wait__timeout_as_typed_wrong"
# subject = "multiprocessing.popen_fork.Popen.wait(timeout: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/popen_fork.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.popen_fork.Popen.wait(timeout: typed); call it with the wrong type.

typeshed contract: timeout is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.popen_fork import Popen
obj = object.__new__(Popen)
try:
    obj.wait(_W())  # timeout: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
