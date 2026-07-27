use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/asyncio_sslproto/SSLProtocol__connection_lost__exc_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_sslproto_SSLProtocol__connection_lost__exc_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_sslproto"
# dimension = "type"
# case = "SSLProtocol__connection_lost__exc_as_typed_wrong"
# subject = "asyncio.sslproto.SSLProtocol.connection_lost(exc: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/sslproto.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.sslproto.SSLProtocol.connection_lost(exc: typed); call it with the wrong type.

typeshed contract: exc is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.sslproto import SSLProtocol
obj = object.__new__(SSLProtocol)
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

/// Ported from `tests/cpython/type/std-libs/asyncio_sslproto/SSLProtocol__get_buffer__n_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_sslproto_SSLProtocol__get_buffer__n_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_sslproto"
# dimension = "type"
# case = "SSLProtocol__get_buffer__n_as_int_wrong"
# subject = "asyncio.sslproto.SSLProtocol.get_buffer(n: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/sslproto.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.sslproto.SSLProtocol.get_buffer(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.sslproto import SSLProtocol
obj = object.__new__(SSLProtocol)
try:
    obj.get_buffer("not_an_int")  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_sslproto/SSLProtocol__init__loop_as_AbstractEventLoop_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_sslproto_SSLProtocol__init__loop_as_AbstractEventLoop_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_sslproto"
# dimension = "type"
# case = "SSLProtocol__init__loop_as_AbstractEventLoop_wrong"
# subject = "asyncio.sslproto.SSLProtocol.__init__(loop: AbstractEventLoop)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/sslproto.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.sslproto.SSLProtocol.__init__(loop: AbstractEventLoop); call it with the wrong type.

typeshed contract: loop is AbstractEventLoop. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.sslproto import SSLProtocol
try:
    SSLProtocol(_W(), None, None, None)  # loop: AbstractEventLoop <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_sslproto/add_flowcontrol_defaults__high_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_sslproto_add_flowcontrol_defaults__high_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_sslproto"
# dimension = "type"
# case = "add_flowcontrol_defaults__high_as_typed_wrong"
# subject = "asyncio.sslproto.add_flowcontrol_defaults(high: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/sslproto.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.sslproto.add_flowcontrol_defaults(high: typed); call it with the wrong type.

typeshed contract: high is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.sslproto import add_flowcontrol_defaults
try:
    add_flowcontrol_defaults(_W(), None, 0)  # high: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
