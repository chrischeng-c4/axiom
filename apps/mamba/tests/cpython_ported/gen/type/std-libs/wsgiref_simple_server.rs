use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/wsgiref_simple_server/WSGIServer__set_app__application_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_simple_server_WSGIServer__set_app__application_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_simple_server"
# dimension = "type"
# case = "WSGIServer__set_app__application_as_typed_wrong"
# subject = "wsgiref.simple_server.WSGIServer.set_app(application: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/simple_server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.simple_server.WSGIServer.set_app(application: typed); call it with the wrong type.

typeshed contract: application is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.simple_server import WSGIServer
obj = object.__new__(WSGIServer)
try:
    obj.set_app(_W())  # application: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_simple_server/demo_app__environ_as_WSGIEnvironment_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_simple_server_demo_app__environ_as_WSGIEnvironment_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_simple_server"
# dimension = "type"
# case = "demo_app__environ_as_WSGIEnvironment_wrong"
# subject = "wsgiref.simple_server.demo_app(environ: WSGIEnvironment)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/simple_server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.simple_server.demo_app(environ: WSGIEnvironment); call it with the wrong type.

typeshed contract: environ is WSGIEnvironment. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.simple_server import demo_app
try:
    demo_app(_W(), None)  # environ: WSGIEnvironment <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/wsgiref_simple_server/make_server__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_wsgiref_simple_server_make_server__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_simple_server"
# dimension = "type"
# case = "make_server__host_as_str_wrong"
# subject = "wsgiref.simple_server.make_server(host: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/simple_server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.simple_server.make_server(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from wsgiref.simple_server import make_server
try:
    make_server(12345, 0, None)  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
