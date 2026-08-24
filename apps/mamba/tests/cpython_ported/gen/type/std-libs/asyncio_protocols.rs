use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/asyncio_protocols/BaseProtocol__connection_lost__exc_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_protocols_BaseProtocol__connection_lost__exc_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_protocols"
# dimension = "type"
# case = "BaseProtocol__connection_lost__exc_as_typed_wrong"
# subject = "asyncio.protocols.BaseProtocol.connection_lost(exc: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/protocols.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.protocols.BaseProtocol.connection_lost(exc: typed); call it with the wrong type.

typeshed contract: exc is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.protocols import BaseProtocol
obj = object.__new__(BaseProtocol)
try:
    obj.connection_lost(_W())  # exc: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_protocols/BaseProtocol__connection_made__transport_as_BaseTransport_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_protocols_BaseProtocol__connection_made__transport_as_BaseTransport_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_protocols"
# dimension = "type"
# case = "BaseProtocol__connection_made__transport_as_BaseTransport_wrong"
# subject = "asyncio.protocols.BaseProtocol.connection_made(transport: BaseTransport)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/protocols.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.protocols.BaseProtocol.connection_made(transport: BaseTransport); call it with the wrong type.

typeshed contract: transport is BaseTransport. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.protocols import BaseProtocol
obj = object.__new__(BaseProtocol)
try:
    obj.connection_made(_W())  # transport: BaseTransport <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_protocols/BufferedProtocol__buffer_updated__nbytes_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_protocols_BufferedProtocol__buffer_updated__nbytes_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_protocols"
# dimension = "type"
# case = "BufferedProtocol__buffer_updated__nbytes_as_int_wrong"
# subject = "asyncio.protocols.BufferedProtocol.buffer_updated(nbytes: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/protocols.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.protocols.BufferedProtocol.buffer_updated(nbytes: int); call it with the wrong type.

typeshed contract: nbytes is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.protocols import BufferedProtocol
obj = object.__new__(BufferedProtocol)
try:
    obj.buffer_updated("not_an_int")  # nbytes: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_protocols/BufferedProtocol__get_buffer__sizehint_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_protocols_BufferedProtocol__get_buffer__sizehint_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_protocols"
# dimension = "type"
# case = "BufferedProtocol__get_buffer__sizehint_as_int_wrong"
# subject = "asyncio.protocols.BufferedProtocol.get_buffer(sizehint: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/protocols.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.protocols.BufferedProtocol.get_buffer(sizehint: int); call it with the wrong type.

typeshed contract: sizehint is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.protocols import BufferedProtocol
obj = object.__new__(BufferedProtocol)
try:
    obj.get_buffer("not_an_int")  # sizehint: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_protocols/DatagramProtocol__connection_made__transport_as_DatagramTransport_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_protocols_DatagramProtocol__connection_made__transport_as_DatagramTransport_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_protocols"
# dimension = "type"
# case = "DatagramProtocol__connection_made__transport_as_DatagramTransport_wrong"
# subject = "asyncio.protocols.DatagramProtocol.connection_made(transport: DatagramTransport)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/protocols.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.protocols.DatagramProtocol.connection_made(transport: DatagramTransport); call it with the wrong type.

typeshed contract: transport is DatagramTransport. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.protocols import DatagramProtocol
obj = object.__new__(DatagramProtocol)
try:
    obj.connection_made(_W())  # transport: DatagramTransport <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_protocols/DatagramProtocol__error_received__exc_as_Exception_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_protocols_DatagramProtocol__error_received__exc_as_Exception_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_protocols"
# dimension = "type"
# case = "DatagramProtocol__error_received__exc_as_Exception_wrong"
# subject = "asyncio.protocols.DatagramProtocol.error_received(exc: Exception)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/protocols.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.protocols.DatagramProtocol.error_received(exc: Exception); call it with the wrong type.

typeshed contract: exc is Exception. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.protocols import DatagramProtocol
obj = object.__new__(DatagramProtocol)
try:
    obj.error_received(_W())  # exc: Exception <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_protocols/SubprocessProtocol__pipe_connection_lost__fd_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_protocols_SubprocessProtocol__pipe_connection_lost__fd_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_protocols"
# dimension = "type"
# case = "SubprocessProtocol__pipe_connection_lost__fd_as_int_wrong"
# subject = "asyncio.protocols.SubprocessProtocol.pipe_connection_lost(fd: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/protocols.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.protocols.SubprocessProtocol.pipe_connection_lost(fd: int); call it with the wrong type.

typeshed contract: fd is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.protocols import SubprocessProtocol
obj = object.__new__(SubprocessProtocol)
try:
    obj.pipe_connection_lost("not_an_int", None)  # fd: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_protocols/SubprocessProtocol__pipe_data_received__fd_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_protocols_SubprocessProtocol__pipe_data_received__fd_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_protocols"
# dimension = "type"
# case = "SubprocessProtocol__pipe_data_received__fd_as_int_wrong"
# subject = "asyncio.protocols.SubprocessProtocol.pipe_data_received(fd: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/protocols.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.protocols.SubprocessProtocol.pipe_data_received(fd: int); call it with the wrong type.

typeshed contract: fd is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.protocols import SubprocessProtocol
obj = object.__new__(SubprocessProtocol)
try:
    obj.pipe_data_received("not_an_int", b"")  # fd: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
