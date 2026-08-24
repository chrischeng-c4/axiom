use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/http_server/BaseHTTPRequestHandler__date_time_string__timestamp_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_BaseHTTPRequestHandler__date_time_string__timestamp_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "BaseHTTPRequestHandler__date_time_string__timestamp_as_typed_wrong"
# subject = "http.server.BaseHTTPRequestHandler.date_time_string(timestamp: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.BaseHTTPRequestHandler.date_time_string(timestamp: typed); call it with the wrong type.

typeshed contract: timestamp is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.server import BaseHTTPRequestHandler
obj = object.__new__(BaseHTTPRequestHandler)
try:
    obj.date_time_string(_W())  # timestamp: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_server/BaseHTTPRequestHandler__log_error__format_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_BaseHTTPRequestHandler__log_error__format_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "BaseHTTPRequestHandler__log_error__format_as_str_wrong"
# subject = "http.server.BaseHTTPRequestHandler.log_error(format: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.BaseHTTPRequestHandler.log_error(format: str); call it with the wrong type.

typeshed contract: format is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.server import BaseHTTPRequestHandler
obj = object.__new__(BaseHTTPRequestHandler)
try:
    obj.log_error(12345)  # format: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_server/BaseHTTPRequestHandler__log_message__format_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_BaseHTTPRequestHandler__log_message__format_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "BaseHTTPRequestHandler__log_message__format_as_str_wrong"
# subject = "http.server.BaseHTTPRequestHandler.log_message(format: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.BaseHTTPRequestHandler.log_message(format: str); call it with the wrong type.

typeshed contract: format is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.server import BaseHTTPRequestHandler
obj = object.__new__(BaseHTTPRequestHandler)
try:
    obj.log_message(12345)  # format: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_server/BaseHTTPRequestHandler__log_request__code_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_BaseHTTPRequestHandler__log_request__code_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "BaseHTTPRequestHandler__log_request__code_as_typed_wrong"
# subject = "http.server.BaseHTTPRequestHandler.log_request(code: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.BaseHTTPRequestHandler.log_request(code: typed); call it with the wrong type.

typeshed contract: code is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.server import BaseHTTPRequestHandler
obj = object.__new__(BaseHTTPRequestHandler)
try:
    obj.log_request(_W())  # code: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_server/BaseHTTPRequestHandler__send_error__code_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_BaseHTTPRequestHandler__send_error__code_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "BaseHTTPRequestHandler__send_error__code_as_int_wrong"
# subject = "http.server.BaseHTTPRequestHandler.send_error(code: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.BaseHTTPRequestHandler.send_error(code: int); call it with the wrong type.

typeshed contract: code is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.server import BaseHTTPRequestHandler
obj = object.__new__(BaseHTTPRequestHandler)
try:
    obj.send_error("not_an_int")  # code: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_server/BaseHTTPRequestHandler__send_header__keyword_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_BaseHTTPRequestHandler__send_header__keyword_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "BaseHTTPRequestHandler__send_header__keyword_as_str_wrong"
# subject = "http.server.BaseHTTPRequestHandler.send_header(keyword: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.BaseHTTPRequestHandler.send_header(keyword: str); call it with the wrong type.

typeshed contract: keyword is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.server import BaseHTTPRequestHandler
obj = object.__new__(BaseHTTPRequestHandler)
try:
    obj.send_header(12345, "")  # keyword: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_server/BaseHTTPRequestHandler__send_response__code_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_BaseHTTPRequestHandler__send_response__code_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "BaseHTTPRequestHandler__send_response__code_as_int_wrong"
# subject = "http.server.BaseHTTPRequestHandler.send_response(code: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.BaseHTTPRequestHandler.send_response(code: int); call it with the wrong type.

typeshed contract: code is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.server import BaseHTTPRequestHandler
obj = object.__new__(BaseHTTPRequestHandler)
try:
    obj.send_response("not_an_int")  # code: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_server/BaseHTTPRequestHandler__send_response_only__code_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_BaseHTTPRequestHandler__send_response_only__code_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "BaseHTTPRequestHandler__send_response_only__code_as_int_wrong"
# subject = "http.server.BaseHTTPRequestHandler.send_response_only(code: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.BaseHTTPRequestHandler.send_response_only(code: int); call it with the wrong type.

typeshed contract: code is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.server import BaseHTTPRequestHandler
obj = object.__new__(BaseHTTPRequestHandler)
try:
    obj.send_response_only("not_an_int")  # code: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_server/CGIHTTPRequestHandler__is_executable__path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_CGIHTTPRequestHandler__is_executable__path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "CGIHTTPRequestHandler__is_executable__path_as_StrPath_wrong"
# subject = "http.server.CGIHTTPRequestHandler.is_executable(path: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.CGIHTTPRequestHandler.is_executable(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.server import CGIHTTPRequestHandler
obj = object.__new__(CGIHTTPRequestHandler)
try:
    obj.is_executable(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_server/CGIHTTPRequestHandler__is_python__path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_CGIHTTPRequestHandler__is_python__path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "CGIHTTPRequestHandler__is_python__path_as_StrPath_wrong"
# subject = "http.server.CGIHTTPRequestHandler.is_python(path: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.CGIHTTPRequestHandler.is_python(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.server import CGIHTTPRequestHandler
obj = object.__new__(CGIHTTPRequestHandler)
try:
    obj.is_python(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_server/HTTPSServer__init__server_address_as__AfInetAddress_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_HTTPSServer__init__server_address_as__AfInetAddress_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "HTTPSServer__init__server_address_as__AfInetAddress_wrong"
# subject = "http.server.HTTPSServer.__init__(server_address: _AfInetAddress)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.HTTPSServer.__init__(server_address: _AfInetAddress); call it with the wrong type.

typeshed contract: server_address is _AfInetAddress. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.server import HTTPSServer
try:
    HTTPSServer(_W(), None)  # server_address: _AfInetAddress <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_server/SimpleHTTPRequestHandler__copyfile__source_as_SupportsRead_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_SimpleHTTPRequestHandler__copyfile__source_as_SupportsRead_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "SimpleHTTPRequestHandler__copyfile__source_as_SupportsRead_wrong"
# subject = "http.server.SimpleHTTPRequestHandler.copyfile(source: SupportsRead)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.SimpleHTTPRequestHandler.copyfile(source: SupportsRead); call it with the wrong type.

typeshed contract: source is SupportsRead. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.server import SimpleHTTPRequestHandler
obj = object.__new__(SimpleHTTPRequestHandler)
try:
    obj.copyfile(_W(), None)  # source: SupportsRead <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_server/SimpleHTTPRequestHandler__guess_type__path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_SimpleHTTPRequestHandler__guess_type__path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "SimpleHTTPRequestHandler__guess_type__path_as_StrPath_wrong"
# subject = "http.server.SimpleHTTPRequestHandler.guess_type(path: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.SimpleHTTPRequestHandler.guess_type(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.server import SimpleHTTPRequestHandler
obj = object.__new__(SimpleHTTPRequestHandler)
try:
    obj.guess_type(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_server/SimpleHTTPRequestHandler__init__request_as__RequestType_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_SimpleHTTPRequestHandler__init__request_as__RequestType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "SimpleHTTPRequestHandler__init__request_as__RequestType_wrong"
# subject = "http.server.SimpleHTTPRequestHandler.__init__(request: _RequestType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.SimpleHTTPRequestHandler.__init__(request: _RequestType); call it with the wrong type.

typeshed contract: request is _RequestType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.server import SimpleHTTPRequestHandler
try:
    SimpleHTTPRequestHandler(_W(), None, None)  # request: _RequestType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_server/SimpleHTTPRequestHandler__list_directory__path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_SimpleHTTPRequestHandler__list_directory__path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "SimpleHTTPRequestHandler__list_directory__path_as_StrPath_wrong"
# subject = "http.server.SimpleHTTPRequestHandler.list_directory(path: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.SimpleHTTPRequestHandler.list_directory(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.server import SimpleHTTPRequestHandler
obj = object.__new__(SimpleHTTPRequestHandler)
try:
    obj.list_directory(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_server/SimpleHTTPRequestHandler__translate_path__path_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_SimpleHTTPRequestHandler__translate_path__path_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "SimpleHTTPRequestHandler__translate_path__path_as_str_wrong"
# subject = "http.server.SimpleHTTPRequestHandler.translate_path(path: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.SimpleHTTPRequestHandler.translate_path(path: str); call it with the wrong type.

typeshed contract: path is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.server import SimpleHTTPRequestHandler
obj = object.__new__(SimpleHTTPRequestHandler)
try:
    obj.translate_path(12345)  # path: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_server/executable__path_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_server_executable__path_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "executable__path_as_StrPath_wrong"
# subject = "http.server.executable(path: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.executable(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.server import executable
try:
    executable(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
