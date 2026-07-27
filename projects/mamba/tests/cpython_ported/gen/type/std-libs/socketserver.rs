use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/socketserver/BaseRequestHandler__init__request_as__RequestType_wrong.py`.
#[test]
fn test_gen_type_std_libs_socketserver_BaseRequestHandler__init__request_as__RequestType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "BaseRequestHandler__init__request_as__RequestType_wrong"
# subject = "socketserver.BaseRequestHandler.__init__(request: _RequestType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.BaseRequestHandler.__init__(request: _RequestType); call it with the wrong type.

typeshed contract: request is _RequestType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socketserver import BaseRequestHandler
try:
    BaseRequestHandler(_W(), None, None)  # request: _RequestType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/socketserver/BaseServer__close_request__request_as__RequestType_wrong.py`.
#[test]
fn test_gen_type_std_libs_socketserver_BaseServer__close_request__request_as__RequestType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "BaseServer__close_request__request_as__RequestType_wrong"
# subject = "socketserver.BaseServer.close_request(request: _RequestType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.BaseServer.close_request(request: _RequestType); call it with the wrong type.

typeshed contract: request is _RequestType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socketserver import BaseServer
obj = object.__new__(BaseServer)
try:
    obj.close_request(_W())  # request: _RequestType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/socketserver/BaseServer__finish_request__request_as__RequestType_wrong.py`.
#[test]
fn test_gen_type_std_libs_socketserver_BaseServer__finish_request__request_as__RequestType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "BaseServer__finish_request__request_as__RequestType_wrong"
# subject = "socketserver.BaseServer.finish_request(request: _RequestType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.BaseServer.finish_request(request: _RequestType); call it with the wrong type.

typeshed contract: request is _RequestType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socketserver import BaseServer
obj = object.__new__(BaseServer)
try:
    obj.finish_request(_W(), None)  # request: _RequestType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/socketserver/BaseServer__handle_error__request_as__RequestType_wrong.py`.
#[test]
fn test_gen_type_std_libs_socketserver_BaseServer__handle_error__request_as__RequestType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "BaseServer__handle_error__request_as__RequestType_wrong"
# subject = "socketserver.BaseServer.handle_error(request: _RequestType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.BaseServer.handle_error(request: _RequestType); call it with the wrong type.

typeshed contract: request is _RequestType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socketserver import BaseServer
obj = object.__new__(BaseServer)
try:
    obj.handle_error(_W(), None)  # request: _RequestType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/socketserver/BaseServer__init__server_address_as__Address_wrong.py`.
#[test]
fn test_gen_type_std_libs_socketserver_BaseServer__init__server_address_as__Address_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "BaseServer__init__server_address_as__Address_wrong"
# subject = "socketserver.BaseServer.__init__(server_address: _Address)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.BaseServer.__init__(server_address: _Address); call it with the wrong type.

typeshed contract: server_address is _Address. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socketserver import BaseServer
try:
    BaseServer(_W(), None)  # server_address: _Address <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/socketserver/BaseServer__process_request__request_as__RequestType_wrong.py`.
#[test]
fn test_gen_type_std_libs_socketserver_BaseServer__process_request__request_as__RequestType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "BaseServer__process_request__request_as__RequestType_wrong"
# subject = "socketserver.BaseServer.process_request(request: _RequestType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.BaseServer.process_request(request: _RequestType); call it with the wrong type.

typeshed contract: request is _RequestType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socketserver import BaseServer
obj = object.__new__(BaseServer)
try:
    obj.process_request(_W(), None)  # request: _RequestType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/socketserver/BaseServer__serve_forever__poll_interval_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_socketserver_BaseServer__serve_forever__poll_interval_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "BaseServer__serve_forever__poll_interval_as_float_wrong"
# subject = "socketserver.BaseServer.serve_forever(poll_interval: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.BaseServer.serve_forever(poll_interval: float); call it with the wrong type.

typeshed contract: poll_interval is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from socketserver import BaseServer
obj = object.__new__(BaseServer)
try:
    obj.serve_forever("not_a_float")  # poll_interval: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/socketserver/BaseServer__shutdown_request__request_as__RequestType_wrong.py`.
#[test]
fn test_gen_type_std_libs_socketserver_BaseServer__shutdown_request__request_as__RequestType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "BaseServer__shutdown_request__request_as__RequestType_wrong"
# subject = "socketserver.BaseServer.shutdown_request(request: _RequestType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.BaseServer.shutdown_request(request: _RequestType); call it with the wrong type.

typeshed contract: request is _RequestType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socketserver import BaseServer
obj = object.__new__(BaseServer)
try:
    obj.shutdown_request(_W())  # request: _RequestType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/socketserver/BaseServer__verify_request__request_as__RequestType_wrong.py`.
#[test]
fn test_gen_type_std_libs_socketserver_BaseServer__verify_request__request_as__RequestType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "BaseServer__verify_request__request_as__RequestType_wrong"
# subject = "socketserver.BaseServer.verify_request(request: _RequestType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.BaseServer.verify_request(request: _RequestType); call it with the wrong type.

typeshed contract: request is _RequestType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socketserver import BaseServer
obj = object.__new__(BaseServer)
try:
    obj.verify_request(_W(), None)  # request: _RequestType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/socketserver/ForkingMixIn__process_request__request_as__RequestType_wrong.py`.
#[test]
fn test_gen_type_std_libs_socketserver_ForkingMixIn__process_request__request_as__RequestType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "ForkingMixIn__process_request__request_as__RequestType_wrong"
# subject = "socketserver.ForkingMixIn.process_request(request: _RequestType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.ForkingMixIn.process_request(request: _RequestType); call it with the wrong type.

typeshed contract: request is _RequestType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socketserver import ForkingMixIn
obj = object.__new__(ForkingMixIn)
try:
    obj.process_request(_W(), None)  # request: _RequestType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/socketserver/TCPServer__init__server_address_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_socketserver_TCPServer__init__server_address_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "TCPServer__init__server_address_as_typed_wrong"
# subject = "socketserver.TCPServer.__init__(server_address: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.TCPServer.__init__(server_address: typed); call it with the wrong type.

typeshed contract: server_address is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socketserver import TCPServer
try:
    TCPServer(_W(), None)  # server_address: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/socketserver/ThreadingMixIn__process_request__request_as__RequestType_wrong.py`.
#[test]
fn test_gen_type_std_libs_socketserver_ThreadingMixIn__process_request__request_as__RequestType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "ThreadingMixIn__process_request__request_as__RequestType_wrong"
# subject = "socketserver.ThreadingMixIn.process_request(request: _RequestType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.ThreadingMixIn.process_request(request: _RequestType); call it with the wrong type.

typeshed contract: request is _RequestType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socketserver import ThreadingMixIn
obj = object.__new__(ThreadingMixIn)
try:
    obj.process_request(_W(), None)  # request: _RequestType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/socketserver/ThreadingMixIn__process_request_thread__request_as__RequestType_wrong.py`.
#[test]
fn test_gen_type_std_libs_socketserver_ThreadingMixIn__process_request_thread__request_as__RequestType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "ThreadingMixIn__process_request_thread__request_as__RequestType_wrong"
# subject = "socketserver.ThreadingMixIn.process_request_thread(request: _RequestType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.ThreadingMixIn.process_request_thread(request: _RequestType); call it with the wrong type.

typeshed contract: request is _RequestType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socketserver import ThreadingMixIn
obj = object.__new__(ThreadingMixIn)
try:
    obj.process_request_thread(_W(), None)  # request: _RequestType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/socketserver/UnixDatagramServer__init__server_address_as__AfUnixAddress_wrong.py`.
#[test]
fn test_gen_type_std_libs_socketserver_UnixDatagramServer__init__server_address_as__AfUnixAddress_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "UnixDatagramServer__init__server_address_as__AfUnixAddress_wrong"
# subject = "socketserver.UnixDatagramServer.__init__(server_address: _AfUnixAddress)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.UnixDatagramServer.__init__(server_address: _AfUnixAddress); call it with the wrong type.

typeshed contract: server_address is _AfUnixAddress. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socketserver import UnixDatagramServer
try:
    UnixDatagramServer(_W(), None)  # server_address: _AfUnixAddress <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/socketserver/UnixStreamServer__init__server_address_as__AfUnixAddress_wrong.py`.
#[test]
fn test_gen_type_std_libs_socketserver_UnixStreamServer__init__server_address_as__AfUnixAddress_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socketserver"
# dimension = "type"
# case = "UnixStreamServer__init__server_address_as__AfUnixAddress_wrong"
# subject = "socketserver.UnixStreamServer.__init__(server_address: _AfUnixAddress)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/socketserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: socketserver.UnixStreamServer.__init__(server_address: _AfUnixAddress); call it with the wrong type.

typeshed contract: server_address is _AfUnixAddress. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from socketserver import UnixStreamServer
try:
    UnixStreamServer(_W(), None)  # server_address: _AfUnixAddress <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
