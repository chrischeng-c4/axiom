use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/xmlrpc_server/CGIXMLRPCRequestHandler__handle_request__request_text_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_server_CGIXMLRPCRequestHandler__handle_request__request_text_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "CGIXMLRPCRequestHandler__handle_request__request_text_as_typed_wrong"
# subject = "xmlrpc.server.CGIXMLRPCRequestHandler.handle_request(request_text: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.CGIXMLRPCRequestHandler.handle_request(request_text: typed); call it with the wrong type.

typeshed contract: request_text is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.server import CGIXMLRPCRequestHandler
obj = object.__new__(CGIXMLRPCRequestHandler)
try:
    obj.handle_request(_W())  # request_text: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_server/CGIXMLRPCRequestHandler__handle_xmlrpc__request_text_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_server_CGIXMLRPCRequestHandler__handle_xmlrpc__request_text_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "CGIXMLRPCRequestHandler__handle_xmlrpc__request_text_as_str_wrong"
# subject = "xmlrpc.server.CGIXMLRPCRequestHandler.handle_xmlrpc(request_text: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.CGIXMLRPCRequestHandler.handle_xmlrpc(request_text: str); call it with the wrong type.

typeshed contract: request_text is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import CGIXMLRPCRequestHandler
obj = object.__new__(CGIXMLRPCRequestHandler)
try:
    obj.handle_xmlrpc(12345)  # request_text: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_server/MultiPathXMLRPCServer__add_dispatcher__path_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_server_MultiPathXMLRPCServer__add_dispatcher__path_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "MultiPathXMLRPCServer__add_dispatcher__path_as_str_wrong"
# subject = "xmlrpc.server.MultiPathXMLRPCServer.add_dispatcher(path: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.MultiPathXMLRPCServer.add_dispatcher(path: str); call it with the wrong type.

typeshed contract: path is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import MultiPathXMLRPCServer
obj = object.__new__(MultiPathXMLRPCServer)
try:
    obj.add_dispatcher(12345, None)  # path: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_server/MultiPathXMLRPCServer__get_dispatcher__path_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_server_MultiPathXMLRPCServer__get_dispatcher__path_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "MultiPathXMLRPCServer__get_dispatcher__path_as_str_wrong"
# subject = "xmlrpc.server.MultiPathXMLRPCServer.get_dispatcher(path: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.MultiPathXMLRPCServer.get_dispatcher(path: str); call it with the wrong type.

typeshed contract: path is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import MultiPathXMLRPCServer
obj = object.__new__(MultiPathXMLRPCServer)
try:
    obj.get_dispatcher(12345)  # path: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_server/ServerHTMLDoc__docroutine__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_server_ServerHTMLDoc__docroutine__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "ServerHTMLDoc__docroutine__name_as_str_wrong"
# subject = "xmlrpc.server.ServerHTMLDoc.docroutine(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.ServerHTMLDoc.docroutine(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import ServerHTMLDoc
obj = object.__new__(ServerHTMLDoc)
try:
    obj.docroutine(None, 12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_server/ServerHTMLDoc__docserver__server_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_server_ServerHTMLDoc__docserver__server_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "ServerHTMLDoc__docserver__server_name_as_str_wrong"
# subject = "xmlrpc.server.ServerHTMLDoc.docserver(server_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.ServerHTMLDoc.docserver(server_name: str); call it with the wrong type.

typeshed contract: server_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import ServerHTMLDoc
obj = object.__new__(ServerHTMLDoc)
try:
    obj.docserver(12345, "", None)  # server_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_server/SimpleXMLRPCDispatcher__register_function__function_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_server_SimpleXMLRPCDispatcher__register_function__function_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "SimpleXMLRPCDispatcher__register_function__function_as_typed_wrong"
# subject = "xmlrpc.server.SimpleXMLRPCDispatcher.register_function(function: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.SimpleXMLRPCDispatcher.register_function(function: typed); call it with the wrong type.

typeshed contract: function is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from xmlrpc.server import SimpleXMLRPCDispatcher
obj = object.__new__(SimpleXMLRPCDispatcher)
try:
    obj.register_function(_W())  # function: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_server/SimpleXMLRPCDispatcher__system_methodHelp__method_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_server_SimpleXMLRPCDispatcher__system_methodHelp__method_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "SimpleXMLRPCDispatcher__system_methodHelp__method_name_as_str_wrong"
# subject = "xmlrpc.server.SimpleXMLRPCDispatcher.system_methodHelp(method_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.SimpleXMLRPCDispatcher.system_methodHelp(method_name: str); call it with the wrong type.

typeshed contract: method_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import SimpleXMLRPCDispatcher
obj = object.__new__(SimpleXMLRPCDispatcher)
try:
    obj.system_methodHelp(12345)  # method_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_server/SimpleXMLRPCDispatcher__system_methodSignature__method_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_server_SimpleXMLRPCDispatcher__system_methodSignature__method_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "SimpleXMLRPCDispatcher__system_methodSignature__method_name_as_str_wrong"
# subject = "xmlrpc.server.SimpleXMLRPCDispatcher.system_methodSignature(method_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.SimpleXMLRPCDispatcher.system_methodSignature(method_name: str); call it with the wrong type.

typeshed contract: method_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import SimpleXMLRPCDispatcher
obj = object.__new__(SimpleXMLRPCDispatcher)
try:
    obj.system_methodSignature(12345)  # method_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_server/XMLRPCDocGenerator__set_server_documentation__server_documentation_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_server_XMLRPCDocGenerator__set_server_documentation__server_documentation_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "XMLRPCDocGenerator__set_server_documentation__server_documentation_as_str_wrong"
# subject = "xmlrpc.server.XMLRPCDocGenerator.set_server_documentation(server_documentation: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.XMLRPCDocGenerator.set_server_documentation(server_documentation: str); call it with the wrong type.

typeshed contract: server_documentation is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import XMLRPCDocGenerator
obj = object.__new__(XMLRPCDocGenerator)
try:
    obj.set_server_documentation(12345)  # server_documentation: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_server/XMLRPCDocGenerator__set_server_name__server_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_server_XMLRPCDocGenerator__set_server_name__server_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "XMLRPCDocGenerator__set_server_name__server_name_as_str_wrong"
# subject = "xmlrpc.server.XMLRPCDocGenerator.set_server_name(server_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.XMLRPCDocGenerator.set_server_name(server_name: str); call it with the wrong type.

typeshed contract: server_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import XMLRPCDocGenerator
obj = object.__new__(XMLRPCDocGenerator)
try:
    obj.set_server_name(12345)  # server_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_server/XMLRPCDocGenerator__set_server_title__server_title_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_server_XMLRPCDocGenerator__set_server_title__server_title_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "XMLRPCDocGenerator__set_server_title__server_title_as_str_wrong"
# subject = "xmlrpc.server.XMLRPCDocGenerator.set_server_title(server_title: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.XMLRPCDocGenerator.set_server_title(server_title: str); call it with the wrong type.

typeshed contract: server_title is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import XMLRPCDocGenerator
obj = object.__new__(XMLRPCDocGenerator)
try:
    obj.set_server_title(12345)  # server_title: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xmlrpc_server/resolve_dotted_attribute__attr_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xmlrpc_server_resolve_dotted_attribute__attr_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xmlrpc_server"
# dimension = "type"
# case = "resolve_dotted_attribute__attr_as_str_wrong"
# subject = "xmlrpc.server.resolve_dotted_attribute(attr: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xmlrpc/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xmlrpc.server.resolve_dotted_attribute(attr: str); call it with the wrong type.

typeshed contract: attr is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xmlrpc.server import resolve_dotted_attribute
try:
    resolve_dotted_attribute(None, 12345)  # attr: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
