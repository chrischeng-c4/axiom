use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/multiprocessing_resource_sharer/DupFd__init__fd_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_resource_sharer_DupFd__init__fd_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_resource_sharer"
# dimension = "type"
# case = "DupFd__init__fd_as_int_wrong"
# subject = "multiprocessing.resource_sharer.DupFd.__init__(fd: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/resource_sharer.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.resource_sharer.DupFd.__init__(fd: int); call it with the wrong type.

typeshed contract: fd is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.resource_sharer import DupFd
try:
    DupFd("not_an_int")  # fd: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_resource_sharer/DupSocket__init__sock_as_socket_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_resource_sharer_DupSocket__init__sock_as_socket_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_resource_sharer"
# dimension = "type"
# case = "DupSocket__init__sock_as_socket_wrong"
# subject = "multiprocessing.resource_sharer.DupSocket.__init__(sock: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/resource_sharer.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.resource_sharer.DupSocket.__init__(sock: socket); call it with the wrong type.

typeshed contract: sock is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.resource_sharer import DupSocket
try:
    DupSocket(_W())  # sock: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/multiprocessing_resource_sharer/stop__timeout_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_multiprocessing_resource_sharer_stop__timeout_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_resource_sharer"
# dimension = "type"
# case = "stop__timeout_as_typed_wrong"
# subject = "multiprocessing.resource_sharer.stop(timeout: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/resource_sharer.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.resource_sharer.stop(timeout: typed); call it with the wrong type.

typeshed contract: timeout is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.resource_sharer import stop
try:
    stop(_W())  # timeout: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
