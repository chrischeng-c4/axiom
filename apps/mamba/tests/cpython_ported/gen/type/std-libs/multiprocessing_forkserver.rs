use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/multiprocessing_forkserver/ForkServer__connect_to_new_process__fds_as_Sequence_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_forkserver_ForkServer__connect_to_new_process__fds_as_Sequence_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_forkserver"
# dimension = "type"
# case = "ForkServer__connect_to_new_process__fds_as_Sequence_wrong"
# subject = "multiprocessing.forkserver.ForkServer.connect_to_new_process(fds: Sequence)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/forkserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.forkserver.ForkServer.connect_to_new_process(fds: Sequence); call it with the wrong type.

typeshed contract: fds is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.forkserver import ForkServer
obj = object.__new__(ForkServer)
try:
    obj.connect_to_new_process(_W())  # fds: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_forkserver/ForkServer__set_forkserver_preload__modules_names_as_list_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_forkserver_ForkServer__set_forkserver_preload__modules_names_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_forkserver"
# dimension = "type"
# case = "ForkServer__set_forkserver_preload__modules_names_as_list_wrong"
# subject = "multiprocessing.forkserver.ForkServer.set_forkserver_preload(modules_names: list)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/forkserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.forkserver.ForkServer.set_forkserver_preload(modules_names: list); call it with the wrong type.

typeshed contract: modules_names is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.forkserver import ForkServer
obj = object.__new__(ForkServer)
try:
    obj.set_forkserver_preload(12345)  # modules_names: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_forkserver/main__listener_fd_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_forkserver_main__listener_fd_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_forkserver"
# dimension = "type"
# case = "main__listener_fd_as_typed_wrong"
# subject = "multiprocessing.forkserver.main(listener_fd: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/forkserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.forkserver.main(listener_fd: typed); call it with the wrong type.

typeshed contract: listener_fd is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.forkserver import main
try:
    main(_W(), None, None)  # listener_fd: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_forkserver/read_signed__fd_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_forkserver_read_signed__fd_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_forkserver"
# dimension = "type"
# case = "read_signed__fd_as_int_wrong"
# subject = "multiprocessing.forkserver.read_signed(fd: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/forkserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.forkserver.read_signed(fd: int); call it with the wrong type.

typeshed contract: fd is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.forkserver import read_signed
try:
    read_signed("not_an_int")  # fd: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_forkserver/write_signed__fd_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_forkserver_write_signed__fd_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_forkserver"
# dimension = "type"
# case = "write_signed__fd_as_int_wrong"
# subject = "multiprocessing.forkserver.write_signed(fd: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/forkserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.forkserver.write_signed(fd: int); call it with the wrong type.

typeshed contract: fd is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.forkserver import write_signed
try:
    write_signed("not_an_int", 0)  # fd: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
