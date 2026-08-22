use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/IocpProactor__accept__listener_as_socket_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_IocpProactor__accept__listener_as_socket_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "IocpProactor__accept__listener_as_socket_wrong"
# subject = "asyncio.windows_events.IocpProactor.accept(listener: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.IocpProactor.accept(listener: socket); call it with the wrong type.

typeshed contract: listener is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_events import IocpProactor
obj = object.__new__(IocpProactor)
try:
    obj.accept(_W())  # listener: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/IocpProactor__accept_pipe__pipe_as_socket_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_IocpProactor__accept_pipe__pipe_as_socket_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "IocpProactor__accept_pipe__pipe_as_socket_wrong"
# subject = "asyncio.windows_events.IocpProactor.accept_pipe(pipe: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.IocpProactor.accept_pipe(pipe: socket); call it with the wrong type.

typeshed contract: pipe is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_events import IocpProactor
obj = object.__new__(IocpProactor)
try:
    obj.accept_pipe(_W())  # pipe: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/IocpProactor__connect__conn_as_socket_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_IocpProactor__connect__conn_as_socket_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "IocpProactor__connect__conn_as_socket_wrong"
# subject = "asyncio.windows_events.IocpProactor.connect(conn: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.IocpProactor.connect(conn: socket); call it with the wrong type.

typeshed contract: conn is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_events import IocpProactor
obj = object.__new__(IocpProactor)
try:
    obj.connect(_W(), None)  # conn: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/IocpProactor__connect_pipe__address_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_IocpProactor__connect_pipe__address_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "IocpProactor__connect_pipe__address_as_str_wrong"
# subject = "asyncio.windows_events.IocpProactor.connect_pipe(address: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.IocpProactor.connect_pipe(address: str); call it with the wrong type.

typeshed contract: address is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.windows_events import IocpProactor
obj = object.__new__(IocpProactor)
try:
    obj.connect_pipe(12345)  # address: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/IocpProactor__init__concurrency_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_IocpProactor__init__concurrency_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "IocpProactor__init__concurrency_as_int_wrong"
# subject = "asyncio.windows_events.IocpProactor.__init__(concurrency: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.IocpProactor.__init__(concurrency: int); call it with the wrong type.

typeshed contract: concurrency is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.windows_events import IocpProactor
try:
    IocpProactor("not_an_int")  # concurrency: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/IocpProactor__recv__conn_as_socket_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_IocpProactor__recv__conn_as_socket_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "IocpProactor__recv__conn_as_socket_wrong"
# subject = "asyncio.windows_events.IocpProactor.recv(conn: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.IocpProactor.recv(conn: socket); call it with the wrong type.

typeshed contract: conn is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_events import IocpProactor
obj = object.__new__(IocpProactor)
try:
    obj.recv(_W(), 0)  # conn: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/IocpProactor__recv_into__conn_as_socket_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_IocpProactor__recv_into__conn_as_socket_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "IocpProactor__recv_into__conn_as_socket_wrong"
# subject = "asyncio.windows_events.IocpProactor.recv_into(conn: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.IocpProactor.recv_into(conn: socket); call it with the wrong type.

typeshed contract: conn is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_events import IocpProactor
obj = object.__new__(IocpProactor)
try:
    obj.recv_into(_W(), None)  # conn: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/IocpProactor__recvfrom__conn_as_socket_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_IocpProactor__recvfrom__conn_as_socket_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "IocpProactor__recvfrom__conn_as_socket_wrong"
# subject = "asyncio.windows_events.IocpProactor.recvfrom(conn: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.IocpProactor.recvfrom(conn: socket); call it with the wrong type.

typeshed contract: conn is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_events import IocpProactor
obj = object.__new__(IocpProactor)
try:
    obj.recvfrom(_W(), 0)  # conn: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/IocpProactor__recvfrom_into__conn_as_socket_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_IocpProactor__recvfrom_into__conn_as_socket_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "IocpProactor__recvfrom_into__conn_as_socket_wrong"
# subject = "asyncio.windows_events.IocpProactor.recvfrom_into(conn: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.IocpProactor.recvfrom_into(conn: socket); call it with the wrong type.

typeshed contract: conn is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_events import IocpProactor
obj = object.__new__(IocpProactor)
try:
    obj.recvfrom_into(_W(), None)  # conn: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/IocpProactor__select__timeout_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_IocpProactor__select__timeout_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "IocpProactor__select__timeout_as_typed_wrong"
# subject = "asyncio.windows_events.IocpProactor.select(timeout: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.IocpProactor.select(timeout: typed); call it with the wrong type.

typeshed contract: timeout is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_events import IocpProactor
obj = object.__new__(IocpProactor)
try:
    obj.select(_W())  # timeout: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/IocpProactor__send__conn_as_socket_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_IocpProactor__send__conn_as_socket_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "IocpProactor__send__conn_as_socket_wrong"
# subject = "asyncio.windows_events.IocpProactor.send(conn: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.IocpProactor.send(conn: socket); call it with the wrong type.

typeshed contract: conn is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_events import IocpProactor
obj = object.__new__(IocpProactor)
try:
    obj.send(_W(), None)  # conn: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/IocpProactor__sendfile__sock_as_socket_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_IocpProactor__sendfile__sock_as_socket_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "IocpProactor__sendfile__sock_as_socket_wrong"
# subject = "asyncio.windows_events.IocpProactor.sendfile(sock: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.IocpProactor.sendfile(sock: socket); call it with the wrong type.

typeshed contract: sock is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_events import IocpProactor
obj = object.__new__(IocpProactor)
try:
    obj.sendfile(_W(), None, 0, 0)  # sock: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/IocpProactor__sendto__conn_as_socket_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_IocpProactor__sendto__conn_as_socket_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "IocpProactor__sendto__conn_as_socket_wrong"
# subject = "asyncio.windows_events.IocpProactor.sendto(conn: socket)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.IocpProactor.sendto(conn: socket); call it with the wrong type.

typeshed contract: conn is socket. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_events import IocpProactor
obj = object.__new__(IocpProactor)
try:
    obj.sendto(_W(), None)  # conn: socket <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/IocpProactor__set_loop__loop_as_AbstractEventLoop_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_IocpProactor__set_loop__loop_as_AbstractEventLoop_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "IocpProactor__set_loop__loop_as_AbstractEventLoop_wrong"
# subject = "asyncio.windows_events.IocpProactor.set_loop(loop: AbstractEventLoop)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.IocpProactor.set_loop(loop: AbstractEventLoop); call it with the wrong type.

typeshed contract: loop is AbstractEventLoop. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_events import IocpProactor
obj = object.__new__(IocpProactor)
try:
    obj.set_loop(_W())  # loop: AbstractEventLoop <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/IocpProactor__wait_for_handle__handle_as_PipeHandle_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_IocpProactor__wait_for_handle__handle_as_PipeHandle_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "IocpProactor__wait_for_handle__handle_as_PipeHandle_wrong"
# subject = "asyncio.windows_events.IocpProactor.wait_for_handle(handle: PipeHandle)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.IocpProactor.wait_for_handle(handle: PipeHandle); call it with the wrong type.

typeshed contract: handle is PipeHandle. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_events import IocpProactor
obj = object.__new__(IocpProactor)
try:
    obj.wait_for_handle(_W())  # handle: PipeHandle <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/PipeServer__init__address_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_PipeServer__init__address_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "PipeServer__init__address_as_str_wrong"
# subject = "asyncio.windows_events.PipeServer.__init__(address: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.PipeServer.__init__(address: str); call it with the wrong type.

typeshed contract: address is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.windows_events import PipeServer
try:
    PipeServer(12345)  # address: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_windows_events/ProactorEventLoop__init__proactor_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_windows_events_ProactorEventLoop__init__proactor_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_events"
# dimension = "type"
# case = "ProactorEventLoop__init__proactor_as_typed_wrong"
# subject = "asyncio.windows_events.ProactorEventLoop.__init__(proactor: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_events.ProactorEventLoop.__init__(proactor: typed); call it with the wrong type.

typeshed contract: proactor is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_events import ProactorEventLoop
try:
    ProactorEventLoop(_W())  # proactor: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
