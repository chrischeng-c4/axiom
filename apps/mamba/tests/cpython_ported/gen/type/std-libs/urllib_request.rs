use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractBasicAuthHandler__http_error_auth_reqed__authreq_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractBasicAuthHandler__http_error_auth_reqed__authreq_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractBasicAuthHandler__http_error_auth_reqed__authreq_as_str_wrong"
# subject = "urllib.request.AbstractBasicAuthHandler.http_error_auth_reqed(authreq: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractBasicAuthHandler.http_error_auth_reqed(authreq: str); call it with the wrong type.

typeshed contract: authreq is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import AbstractBasicAuthHandler
obj = object.__new__(AbstractBasicAuthHandler)
try:
    obj.http_error_auth_reqed(12345, "", None, None)  # authreq: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractBasicAuthHandler__http_request__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractBasicAuthHandler__http_request__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractBasicAuthHandler__http_request__req_as_Request_wrong"
# subject = "urllib.request.AbstractBasicAuthHandler.http_request(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractBasicAuthHandler.http_request(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import AbstractBasicAuthHandler
obj = object.__new__(AbstractBasicAuthHandler)
try:
    obj.http_request(_W())  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractBasicAuthHandler__http_response__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractBasicAuthHandler__http_response__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractBasicAuthHandler__http_response__req_as_Request_wrong"
# subject = "urllib.request.AbstractBasicAuthHandler.http_response(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractBasicAuthHandler.http_response(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import AbstractBasicAuthHandler
obj = object.__new__(AbstractBasicAuthHandler)
try:
    obj.http_response(_W(), None)  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractBasicAuthHandler__https_request__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractBasicAuthHandler__https_request__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractBasicAuthHandler__https_request__req_as_Request_wrong"
# subject = "urllib.request.AbstractBasicAuthHandler.https_request(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractBasicAuthHandler.https_request(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import AbstractBasicAuthHandler
obj = object.__new__(AbstractBasicAuthHandler)
try:
    obj.https_request(_W())  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractBasicAuthHandler__https_response__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractBasicAuthHandler__https_response__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractBasicAuthHandler__https_response__req_as_Request_wrong"
# subject = "urllib.request.AbstractBasicAuthHandler.https_response(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractBasicAuthHandler.https_response(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import AbstractBasicAuthHandler
obj = object.__new__(AbstractBasicAuthHandler)
try:
    obj.https_response(_W(), None)  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractBasicAuthHandler__init__password_mgr_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractBasicAuthHandler__init__password_mgr_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractBasicAuthHandler__init__password_mgr_as_typed_wrong"
# subject = "urllib.request.AbstractBasicAuthHandler.__init__(password_mgr: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractBasicAuthHandler.__init__(password_mgr: typed); call it with the wrong type.

typeshed contract: password_mgr is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import AbstractBasicAuthHandler
try:
    AbstractBasicAuthHandler(_W())  # password_mgr: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractBasicAuthHandler__retry_http_basic_auth__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractBasicAuthHandler__retry_http_basic_auth__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractBasicAuthHandler__retry_http_basic_auth__host_as_str_wrong"
# subject = "urllib.request.AbstractBasicAuthHandler.retry_http_basic_auth(host: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractBasicAuthHandler.retry_http_basic_auth(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import AbstractBasicAuthHandler
obj = object.__new__(AbstractBasicAuthHandler)
try:
    obj.retry_http_basic_auth(12345, None, "")  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractDigestAuthHandler__get_algorithm_impls__algorithm_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractDigestAuthHandler__get_algorithm_impls__algorithm_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractDigestAuthHandler__get_algorithm_impls__algorithm_as_str_wrong"
# subject = "urllib.request.AbstractDigestAuthHandler.get_algorithm_impls(algorithm: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractDigestAuthHandler.get_algorithm_impls(algorithm: str); call it with the wrong type.

typeshed contract: algorithm is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import AbstractDigestAuthHandler
obj = object.__new__(AbstractDigestAuthHandler)
try:
    obj.get_algorithm_impls(12345)  # algorithm: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractDigestAuthHandler__get_authorization__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractDigestAuthHandler__get_authorization__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractDigestAuthHandler__get_authorization__req_as_Request_wrong"
# subject = "urllib.request.AbstractDigestAuthHandler.get_authorization(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractDigestAuthHandler.get_authorization(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import AbstractDigestAuthHandler
obj = object.__new__(AbstractDigestAuthHandler)
try:
    obj.get_authorization(_W(), None)  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractDigestAuthHandler__get_cnonce__nonce_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractDigestAuthHandler__get_cnonce__nonce_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractDigestAuthHandler__get_cnonce__nonce_as_str_wrong"
# subject = "urllib.request.AbstractDigestAuthHandler.get_cnonce(nonce: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractDigestAuthHandler.get_cnonce(nonce: str); call it with the wrong type.

typeshed contract: nonce is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import AbstractDigestAuthHandler
obj = object.__new__(AbstractDigestAuthHandler)
try:
    obj.get_cnonce(12345)  # nonce: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractDigestAuthHandler__get_entity_digest__data_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractDigestAuthHandler__get_entity_digest__data_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractDigestAuthHandler__get_entity_digest__data_as_typed_wrong"
# subject = "urllib.request.AbstractDigestAuthHandler.get_entity_digest(data: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractDigestAuthHandler.get_entity_digest(data: typed); call it with the wrong type.

typeshed contract: data is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import AbstractDigestAuthHandler
obj = object.__new__(AbstractDigestAuthHandler)
try:
    obj.get_entity_digest(_W(), None)  # data: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractDigestAuthHandler__http_error_auth_reqed__auth_header_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractDigestAuthHandler__http_error_auth_reqed__auth_header_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractDigestAuthHandler__http_error_auth_reqed__auth_header_as_str_wrong"
# subject = "urllib.request.AbstractDigestAuthHandler.http_error_auth_reqed(auth_header: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractDigestAuthHandler.http_error_auth_reqed(auth_header: str); call it with the wrong type.

typeshed contract: auth_header is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import AbstractDigestAuthHandler
obj = object.__new__(AbstractDigestAuthHandler)
try:
    obj.http_error_auth_reqed(12345, "", None, None)  # auth_header: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractDigestAuthHandler__init__passwd_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractDigestAuthHandler__init__passwd_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractDigestAuthHandler__init__passwd_as_typed_wrong"
# subject = "urllib.request.AbstractDigestAuthHandler.__init__(passwd: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractDigestAuthHandler.__init__(passwd: typed); call it with the wrong type.

typeshed contract: passwd is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import AbstractDigestAuthHandler
try:
    AbstractDigestAuthHandler(_W())  # passwd: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractDigestAuthHandler__retry_http_digest_auth__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractDigestAuthHandler__retry_http_digest_auth__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractDigestAuthHandler__retry_http_digest_auth__req_as_Request_wrong"
# subject = "urllib.request.AbstractDigestAuthHandler.retry_http_digest_auth(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractDigestAuthHandler.retry_http_digest_auth(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import AbstractDigestAuthHandler
obj = object.__new__(AbstractDigestAuthHandler)
try:
    obj.retry_http_digest_auth(_W(), "")  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractHTTPHandler__do_open__http_class_as__HTTPConnectionProtocol_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractHTTPHandler__do_open__http_class_as__HTTPConnectionProtocol_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractHTTPHandler__do_open__http_class_as__HTTPConnectionProtocol_wrong"
# subject = "urllib.request.AbstractHTTPHandler.do_open(http_class: _HTTPConnectionProtocol)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractHTTPHandler.do_open(http_class: _HTTPConnectionProtocol); call it with the wrong type.

typeshed contract: http_class is _HTTPConnectionProtocol. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import AbstractHTTPHandler
obj = object.__new__(AbstractHTTPHandler)
try:
    obj.do_open(_W(), None)  # http_class: _HTTPConnectionProtocol <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractHTTPHandler__do_request___request_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractHTTPHandler__do_request___request_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractHTTPHandler__do_request___request_as_Request_wrong"
# subject = "urllib.request.AbstractHTTPHandler.do_request_(request: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractHTTPHandler.do_request_(request: Request); call it with the wrong type.

typeshed contract: request is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import AbstractHTTPHandler
obj = object.__new__(AbstractHTTPHandler)
try:
    obj.do_request_(_W())  # request: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractHTTPHandler__init__debuglevel_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractHTTPHandler__init__debuglevel_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractHTTPHandler__init__debuglevel_as_typed_wrong"
# subject = "urllib.request.AbstractHTTPHandler.__init__(debuglevel: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractHTTPHandler.__init__(debuglevel: typed); call it with the wrong type.

typeshed contract: debuglevel is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import AbstractHTTPHandler
try:
    AbstractHTTPHandler(_W())  # debuglevel: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/AbstractHTTPHandler__set_http_debuglevel__level_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_AbstractHTTPHandler__set_http_debuglevel__level_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractHTTPHandler__set_http_debuglevel__level_as_int_wrong"
# subject = "urllib.request.AbstractHTTPHandler.set_http_debuglevel(level: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractHTTPHandler.set_http_debuglevel(level: int); call it with the wrong type.

typeshed contract: level is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import AbstractHTTPHandler
obj = object.__new__(AbstractHTTPHandler)
try:
    obj.set_http_debuglevel("not_an_int")  # level: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/BaseHandler__add_parent__parent_as_OpenerDirector_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_BaseHandler__add_parent__parent_as_OpenerDirector_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "BaseHandler__add_parent__parent_as_OpenerDirector_wrong"
# subject = "urllib.request.BaseHandler.add_parent(parent: OpenerDirector)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.BaseHandler.add_parent(parent: OpenerDirector); call it with the wrong type.

typeshed contract: parent is OpenerDirector. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import BaseHandler
obj = object.__new__(BaseHandler)
try:
    obj.add_parent(_W())  # parent: OpenerDirector <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/CacheFTPHandler__setMaxConns__m_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_CacheFTPHandler__setMaxConns__m_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "CacheFTPHandler__setMaxConns__m_as_int_wrong"
# subject = "urllib.request.CacheFTPHandler.setMaxConns(m: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.CacheFTPHandler.setMaxConns(m: int); call it with the wrong type.

typeshed contract: m is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import CacheFTPHandler
obj = object.__new__(CacheFTPHandler)
try:
    obj.setMaxConns("not_an_int")  # m: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/CacheFTPHandler__setTimeout__t_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_CacheFTPHandler__setTimeout__t_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "CacheFTPHandler__setTimeout__t_as_float_wrong"
# subject = "urllib.request.CacheFTPHandler.setTimeout(t: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.CacheFTPHandler.setTimeout(t: float); call it with the wrong type.

typeshed contract: t is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import CacheFTPHandler
obj = object.__new__(CacheFTPHandler)
try:
    obj.setTimeout("not_a_float")  # t: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/DataHandler__data_open__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_DataHandler__data_open__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "DataHandler__data_open__req_as_Request_wrong"
# subject = "urllib.request.DataHandler.data_open(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.DataHandler.data_open(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import DataHandler
obj = object.__new__(DataHandler)
try:
    obj.data_open(_W())  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FTPHandler__connect_ftp__user_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FTPHandler__connect_ftp__user_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FTPHandler__connect_ftp__user_as_str_wrong"
# subject = "urllib.request.FTPHandler.connect_ftp(user: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FTPHandler.connect_ftp(user: str); call it with the wrong type.

typeshed contract: user is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import FTPHandler
obj = object.__new__(FTPHandler)
try:
    obj.connect_ftp(12345, "", "", 0, "", 0.0)  # user: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FTPHandler__ftp_open__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FTPHandler__ftp_open__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FTPHandler__ftp_open__req_as_Request_wrong"
# subject = "urllib.request.FTPHandler.ftp_open(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FTPHandler.ftp_open(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import FTPHandler
obj = object.__new__(FTPHandler)
try:
    obj.ftp_open(_W())  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FancyURLopener__get_user_passwd__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FancyURLopener__get_user_passwd__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FancyURLopener__get_user_passwd__host_as_str_wrong"
# subject = "urllib.request.FancyURLopener.get_user_passwd(host: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FancyURLopener.get_user_passwd(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import FancyURLopener
obj = object.__new__(FancyURLopener)
try:
    obj.get_user_passwd(12345, "")  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FancyURLopener__http_error_301__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FancyURLopener__http_error_301__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FancyURLopener__http_error_301__url_as_str_wrong"
# subject = "urllib.request.FancyURLopener.http_error_301(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FancyURLopener.http_error_301(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import FancyURLopener
obj = object.__new__(FancyURLopener)
try:
    obj.http_error_301(12345, None, 0, "", None)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FancyURLopener__http_error_302__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FancyURLopener__http_error_302__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FancyURLopener__http_error_302__url_as_str_wrong"
# subject = "urllib.request.FancyURLopener.http_error_302(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FancyURLopener.http_error_302(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import FancyURLopener
obj = object.__new__(FancyURLopener)
try:
    obj.http_error_302(12345, None, 0, "", None)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FancyURLopener__http_error_303__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FancyURLopener__http_error_303__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FancyURLopener__http_error_303__url_as_str_wrong"
# subject = "urllib.request.FancyURLopener.http_error_303(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FancyURLopener.http_error_303(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import FancyURLopener
obj = object.__new__(FancyURLopener)
try:
    obj.http_error_303(12345, None, 0, "", None)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FancyURLopener__http_error_307__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FancyURLopener__http_error_307__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FancyURLopener__http_error_307__url_as_str_wrong"
# subject = "urllib.request.FancyURLopener.http_error_307(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FancyURLopener.http_error_307(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import FancyURLopener
obj = object.__new__(FancyURLopener)
try:
    obj.http_error_307(12345, None, 0, "", None)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FancyURLopener__http_error_308__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FancyURLopener__http_error_308__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FancyURLopener__http_error_308__url_as_str_wrong"
# subject = "urllib.request.FancyURLopener.http_error_308(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FancyURLopener.http_error_308(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import FancyURLopener
obj = object.__new__(FancyURLopener)
try:
    obj.http_error_308(12345, None, 0, "", None)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FancyURLopener__http_error_401__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FancyURLopener__http_error_401__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FancyURLopener__http_error_401__url_as_str_wrong"
# subject = "urllib.request.FancyURLopener.http_error_401(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FancyURLopener.http_error_401(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import FancyURLopener
obj = object.__new__(FancyURLopener)
try:
    obj.http_error_401(12345, None, 0, "", None)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FancyURLopener__http_error_407__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FancyURLopener__http_error_407__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FancyURLopener__http_error_407__url_as_str_wrong"
# subject = "urllib.request.FancyURLopener.http_error_407(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FancyURLopener.http_error_407(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import FancyURLopener
obj = object.__new__(FancyURLopener)
try:
    obj.http_error_407(12345, None, 0, "", None)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FancyURLopener__http_error_default__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FancyURLopener__http_error_default__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FancyURLopener__http_error_default__url_as_str_wrong"
# subject = "urllib.request.FancyURLopener.http_error_default(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FancyURLopener.http_error_default(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import FancyURLopener
obj = object.__new__(FancyURLopener)
try:
    obj.http_error_default(12345, None, 0, "", None)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FancyURLopener__prompt_user_passwd__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FancyURLopener__prompt_user_passwd__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FancyURLopener__prompt_user_passwd__host_as_str_wrong"
# subject = "urllib.request.FancyURLopener.prompt_user_passwd(host: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FancyURLopener.prompt_user_passwd(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import FancyURLopener
obj = object.__new__(FancyURLopener)
try:
    obj.prompt_user_passwd(12345, "")  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FancyURLopener__redirect_internal__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FancyURLopener__redirect_internal__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FancyURLopener__redirect_internal__url_as_str_wrong"
# subject = "urllib.request.FancyURLopener.redirect_internal(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FancyURLopener.redirect_internal(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import FancyURLopener
obj = object.__new__(FancyURLopener)
try:
    obj.redirect_internal(12345, None, 0, "", None, None)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FancyURLopener__retry_http_basic_auth__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FancyURLopener__retry_http_basic_auth__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FancyURLopener__retry_http_basic_auth__url_as_str_wrong"
# subject = "urllib.request.FancyURLopener.retry_http_basic_auth(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FancyURLopener.retry_http_basic_auth(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import FancyURLopener
obj = object.__new__(FancyURLopener)
try:
    obj.retry_http_basic_auth(12345, "")  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FancyURLopener__retry_https_basic_auth__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FancyURLopener__retry_https_basic_auth__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FancyURLopener__retry_https_basic_auth__url_as_str_wrong"
# subject = "urllib.request.FancyURLopener.retry_https_basic_auth(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FancyURLopener.retry_https_basic_auth(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import FancyURLopener
obj = object.__new__(FancyURLopener)
try:
    obj.retry_https_basic_auth(12345, "")  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FancyURLopener__retry_proxy_http_basic_auth__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FancyURLopener__retry_proxy_http_basic_auth__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FancyURLopener__retry_proxy_http_basic_auth__url_as_str_wrong"
# subject = "urllib.request.FancyURLopener.retry_proxy_http_basic_auth(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FancyURLopener.retry_proxy_http_basic_auth(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import FancyURLopener
obj = object.__new__(FancyURLopener)
try:
    obj.retry_proxy_http_basic_auth(12345, "")  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FancyURLopener__retry_proxy_https_basic_auth__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FancyURLopener__retry_proxy_https_basic_auth__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FancyURLopener__retry_proxy_https_basic_auth__url_as_str_wrong"
# subject = "urllib.request.FancyURLopener.retry_proxy_https_basic_auth(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FancyURLopener.retry_proxy_https_basic_auth(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import FancyURLopener
obj = object.__new__(FancyURLopener)
try:
    obj.retry_proxy_https_basic_auth(12345, "")  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FileHandler__file_open__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FileHandler__file_open__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FileHandler__file_open__req_as_Request_wrong"
# subject = "urllib.request.FileHandler.file_open(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FileHandler.file_open(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import FileHandler
obj = object.__new__(FileHandler)
try:
    obj.file_open(_W())  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/FileHandler__open_local_file__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_FileHandler__open_local_file__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FileHandler__open_local_file__req_as_Request_wrong"
# subject = "urllib.request.FileHandler.open_local_file(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FileHandler.open_local_file(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import FileHandler
obj = object.__new__(FileHandler)
try:
    obj.open_local_file(_W())  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPBasicAuthHandler__http_error_401__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPBasicAuthHandler__http_error_401__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPBasicAuthHandler__http_error_401__req_as_Request_wrong"
# subject = "urllib.request.HTTPBasicAuthHandler.http_error_401(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPBasicAuthHandler.http_error_401(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPBasicAuthHandler
obj = object.__new__(HTTPBasicAuthHandler)
try:
    obj.http_error_401(_W(), None, 0, "", None)  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPCookieProcessor__http_request__request_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPCookieProcessor__http_request__request_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPCookieProcessor__http_request__request_as_Request_wrong"
# subject = "urllib.request.HTTPCookieProcessor.http_request(request: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPCookieProcessor.http_request(request: Request); call it with the wrong type.

typeshed contract: request is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPCookieProcessor
obj = object.__new__(HTTPCookieProcessor)
try:
    obj.http_request(_W())  # request: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPCookieProcessor__http_response__request_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPCookieProcessor__http_response__request_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPCookieProcessor__http_response__request_as_Request_wrong"
# subject = "urllib.request.HTTPCookieProcessor.http_response(request: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPCookieProcessor.http_response(request: Request); call it with the wrong type.

typeshed contract: request is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPCookieProcessor
obj = object.__new__(HTTPCookieProcessor)
try:
    obj.http_response(_W(), None)  # request: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPCookieProcessor__https_request__request_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPCookieProcessor__https_request__request_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPCookieProcessor__https_request__request_as_Request_wrong"
# subject = "urllib.request.HTTPCookieProcessor.https_request(request: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPCookieProcessor.https_request(request: Request); call it with the wrong type.

typeshed contract: request is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPCookieProcessor
obj = object.__new__(HTTPCookieProcessor)
try:
    obj.https_request(_W())  # request: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPCookieProcessor__https_response__request_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPCookieProcessor__https_response__request_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPCookieProcessor__https_response__request_as_Request_wrong"
# subject = "urllib.request.HTTPCookieProcessor.https_response(request: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPCookieProcessor.https_response(request: Request); call it with the wrong type.

typeshed contract: request is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPCookieProcessor
obj = object.__new__(HTTPCookieProcessor)
try:
    obj.https_response(_W(), None)  # request: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPCookieProcessor__init__cookiejar_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPCookieProcessor__init__cookiejar_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPCookieProcessor__init__cookiejar_as_typed_wrong"
# subject = "urllib.request.HTTPCookieProcessor.__init__(cookiejar: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPCookieProcessor.__init__(cookiejar: typed); call it with the wrong type.

typeshed contract: cookiejar is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPCookieProcessor
try:
    HTTPCookieProcessor(_W())  # cookiejar: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPDefaultErrorHandler__http_error_default__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPDefaultErrorHandler__http_error_default__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPDefaultErrorHandler__http_error_default__req_as_Request_wrong"
# subject = "urllib.request.HTTPDefaultErrorHandler.http_error_default(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPDefaultErrorHandler.http_error_default(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPDefaultErrorHandler
obj = object.__new__(HTTPDefaultErrorHandler)
try:
    obj.http_error_default(_W(), None, 0, "", None)  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPDigestAuthHandler__http_error_401__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPDigestAuthHandler__http_error_401__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPDigestAuthHandler__http_error_401__req_as_Request_wrong"
# subject = "urllib.request.HTTPDigestAuthHandler.http_error_401(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPDigestAuthHandler.http_error_401(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPDigestAuthHandler
obj = object.__new__(HTTPDigestAuthHandler)
try:
    obj.http_error_401(_W(), None, 0, "", None)  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPErrorProcessor__http_response__request_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPErrorProcessor__http_response__request_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPErrorProcessor__http_response__request_as_Request_wrong"
# subject = "urllib.request.HTTPErrorProcessor.http_response(request: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPErrorProcessor.http_response(request: Request); call it with the wrong type.

typeshed contract: request is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPErrorProcessor
obj = object.__new__(HTTPErrorProcessor)
try:
    obj.http_response(_W(), None)  # request: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPErrorProcessor__https_response__request_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPErrorProcessor__https_response__request_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPErrorProcessor__https_response__request_as_Request_wrong"
# subject = "urllib.request.HTTPErrorProcessor.https_response(request: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPErrorProcessor.https_response(request: Request); call it with the wrong type.

typeshed contract: request is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPErrorProcessor
obj = object.__new__(HTTPErrorProcessor)
try:
    obj.https_response(_W(), None)  # request: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPHandler__http_open__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPHandler__http_open__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPHandler__http_open__req_as_Request_wrong"
# subject = "urllib.request.HTTPHandler.http_open(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPHandler.http_open(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPHandler
obj = object.__new__(HTTPHandler)
try:
    obj.http_open(_W())  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPHandler__http_request__request_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPHandler__http_request__request_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPHandler__http_request__request_as_Request_wrong"
# subject = "urllib.request.HTTPHandler.http_request(request: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPHandler.http_request(request: Request); call it with the wrong type.

typeshed contract: request is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPHandler
obj = object.__new__(HTTPHandler)
try:
    obj.http_request(_W())  # request: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPPasswordMgrWithDefaultRealm__add_password__realm_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPPasswordMgrWithDefaultRealm__add_password__realm_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPPasswordMgrWithDefaultRealm__add_password__realm_as_typed_wrong"
# subject = "urllib.request.HTTPPasswordMgrWithDefaultRealm.add_password(realm: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPPasswordMgrWithDefaultRealm.add_password(realm: typed); call it with the wrong type.

typeshed contract: realm is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPPasswordMgrWithDefaultRealm
obj = object.__new__(HTTPPasswordMgrWithDefaultRealm)
try:
    obj.add_password(_W(), None, "", "")  # realm: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPPasswordMgrWithDefaultRealm__find_user_password__realm_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPPasswordMgrWithDefaultRealm__find_user_password__realm_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPPasswordMgrWithDefaultRealm__find_user_password__realm_as_typed_wrong"
# subject = "urllib.request.HTTPPasswordMgrWithDefaultRealm.find_user_password(realm: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPPasswordMgrWithDefaultRealm.find_user_password(realm: typed); call it with the wrong type.

typeshed contract: realm is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPPasswordMgrWithDefaultRealm
obj = object.__new__(HTTPPasswordMgrWithDefaultRealm)
try:
    obj.find_user_password(_W(), "")  # realm: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPPasswordMgrWithPriorAuth__add_password__realm_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPPasswordMgrWithPriorAuth__add_password__realm_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPPasswordMgrWithPriorAuth__add_password__realm_as_typed_wrong"
# subject = "urllib.request.HTTPPasswordMgrWithPriorAuth.add_password(realm: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPPasswordMgrWithPriorAuth.add_password(realm: typed); call it with the wrong type.

typeshed contract: realm is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPPasswordMgrWithPriorAuth
obj = object.__new__(HTTPPasswordMgrWithPriorAuth)
try:
    obj.add_password(_W(), None, "", "")  # realm: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPPasswordMgrWithPriorAuth__is_authenticated__authuri_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPPasswordMgrWithPriorAuth__is_authenticated__authuri_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPPasswordMgrWithPriorAuth__is_authenticated__authuri_as_str_wrong"
# subject = "urllib.request.HTTPPasswordMgrWithPriorAuth.is_authenticated(authuri: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPPasswordMgrWithPriorAuth.is_authenticated(authuri: str); call it with the wrong type.

typeshed contract: authuri is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import HTTPPasswordMgrWithPriorAuth
obj = object.__new__(HTTPPasswordMgrWithPriorAuth)
try:
    obj.is_authenticated(12345)  # authuri: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPPasswordMgr__add_password__realm_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPPasswordMgr__add_password__realm_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPPasswordMgr__add_password__realm_as_str_wrong"
# subject = "urllib.request.HTTPPasswordMgr.add_password(realm: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPPasswordMgr.add_password(realm: str); call it with the wrong type.

typeshed contract: realm is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import HTTPPasswordMgr
obj = object.__new__(HTTPPasswordMgr)
try:
    obj.add_password(12345, None, "", "")  # realm: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPPasswordMgr__find_user_password__realm_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPPasswordMgr__find_user_password__realm_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPPasswordMgr__find_user_password__realm_as_str_wrong"
# subject = "urllib.request.HTTPPasswordMgr.find_user_password(realm: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPPasswordMgr.find_user_password(realm: str); call it with the wrong type.

typeshed contract: realm is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import HTTPPasswordMgr
obj = object.__new__(HTTPPasswordMgr)
try:
    obj.find_user_password(12345, "")  # realm: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPPasswordMgr__is_suburi__base_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPPasswordMgr__is_suburi__base_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPPasswordMgr__is_suburi__base_as_str_wrong"
# subject = "urllib.request.HTTPPasswordMgr.is_suburi(base: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPPasswordMgr.is_suburi(base: str); call it with the wrong type.

typeshed contract: base is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import HTTPPasswordMgr
obj = object.__new__(HTTPPasswordMgr)
try:
    obj.is_suburi(12345, "")  # base: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPPasswordMgr__reduce_uri__uri_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPPasswordMgr__reduce_uri__uri_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPPasswordMgr__reduce_uri__uri_as_str_wrong"
# subject = "urllib.request.HTTPPasswordMgr.reduce_uri(uri: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPPasswordMgr.reduce_uri(uri: str); call it with the wrong type.

typeshed contract: uri is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import HTTPPasswordMgr
obj = object.__new__(HTTPPasswordMgr)
try:
    obj.reduce_uri(12345)  # uri: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPRedirectHandler__http_error_301__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPRedirectHandler__http_error_301__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPRedirectHandler__http_error_301__req_as_Request_wrong"
# subject = "urllib.request.HTTPRedirectHandler.http_error_301(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPRedirectHandler.http_error_301(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPRedirectHandler
obj = object.__new__(HTTPRedirectHandler)
try:
    obj.http_error_301(_W(), None, 0, "", None)  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPRedirectHandler__http_error_302__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPRedirectHandler__http_error_302__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPRedirectHandler__http_error_302__req_as_Request_wrong"
# subject = "urllib.request.HTTPRedirectHandler.http_error_302(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPRedirectHandler.http_error_302(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPRedirectHandler
obj = object.__new__(HTTPRedirectHandler)
try:
    obj.http_error_302(_W(), None, 0, "", None)  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPRedirectHandler__http_error_303__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPRedirectHandler__http_error_303__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPRedirectHandler__http_error_303__req_as_Request_wrong"
# subject = "urllib.request.HTTPRedirectHandler.http_error_303(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPRedirectHandler.http_error_303(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPRedirectHandler
obj = object.__new__(HTTPRedirectHandler)
try:
    obj.http_error_303(_W(), None, 0, "", None)  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPRedirectHandler__http_error_307__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPRedirectHandler__http_error_307__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPRedirectHandler__http_error_307__req_as_Request_wrong"
# subject = "urllib.request.HTTPRedirectHandler.http_error_307(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPRedirectHandler.http_error_307(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPRedirectHandler
obj = object.__new__(HTTPRedirectHandler)
try:
    obj.http_error_307(_W(), None, 0, "", None)  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPRedirectHandler__http_error_308__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPRedirectHandler__http_error_308__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPRedirectHandler__http_error_308__req_as_Request_wrong"
# subject = "urllib.request.HTTPRedirectHandler.http_error_308(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPRedirectHandler.http_error_308(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPRedirectHandler
obj = object.__new__(HTTPRedirectHandler)
try:
    obj.http_error_308(_W(), None, 0, "", None)  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPRedirectHandler__redirect_request__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPRedirectHandler__redirect_request__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPRedirectHandler__redirect_request__req_as_Request_wrong"
# subject = "urllib.request.HTTPRedirectHandler.redirect_request(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPRedirectHandler.redirect_request(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPRedirectHandler
obj = object.__new__(HTTPRedirectHandler)
try:
    obj.redirect_request(_W(), None, 0, "", None, "")  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPSHandler__https_open__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPSHandler__https_open__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPSHandler__https_open__req_as_Request_wrong"
# subject = "urllib.request.HTTPSHandler.https_open(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPSHandler.https_open(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPSHandler
obj = object.__new__(HTTPSHandler)
try:
    obj.https_open(_W())  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPSHandler__https_request__request_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPSHandler__https_request__request_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPSHandler__https_request__request_as_Request_wrong"
# subject = "urllib.request.HTTPSHandler.https_request(request: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPSHandler.https_request(request: Request); call it with the wrong type.

typeshed contract: request is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPSHandler
obj = object.__new__(HTTPSHandler)
try:
    obj.https_request(_W())  # request: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/HTTPSHandler__init__debuglevel_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_HTTPSHandler__init__debuglevel_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPSHandler__init__debuglevel_as_typed_wrong"
# subject = "urllib.request.HTTPSHandler.__init__(debuglevel: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPSHandler.__init__(debuglevel: typed); call it with the wrong type.

typeshed contract: debuglevel is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPSHandler
try:
    HTTPSHandler(_W())  # debuglevel: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/OpenerDirector__add_handler__handler_as_BaseHandler_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_OpenerDirector__add_handler__handler_as_BaseHandler_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "OpenerDirector__add_handler__handler_as_BaseHandler_wrong"
# subject = "urllib.request.OpenerDirector.add_handler(handler: BaseHandler)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.OpenerDirector.add_handler(handler: BaseHandler); call it with the wrong type.

typeshed contract: handler is BaseHandler. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import OpenerDirector
obj = object.__new__(OpenerDirector)
try:
    obj.add_handler(_W())  # handler: BaseHandler <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/OpenerDirector__open__fullurl_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_OpenerDirector__open__fullurl_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "OpenerDirector__open__fullurl_as_typed_wrong"
# subject = "urllib.request.OpenerDirector.open(fullurl: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.OpenerDirector.open(fullurl: typed); call it with the wrong type.

typeshed contract: fullurl is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import OpenerDirector
obj = object.__new__(OpenerDirector)
try:
    obj.open(_W())  # fullurl: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/ProxyBasicAuthHandler__http_error_407__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_ProxyBasicAuthHandler__http_error_407__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "ProxyBasicAuthHandler__http_error_407__req_as_Request_wrong"
# subject = "urllib.request.ProxyBasicAuthHandler.http_error_407(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.ProxyBasicAuthHandler.http_error_407(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import ProxyBasicAuthHandler
obj = object.__new__(ProxyBasicAuthHandler)
try:
    obj.http_error_407(_W(), None, 0, "", None)  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/ProxyDigestAuthHandler__http_error_407__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_ProxyDigestAuthHandler__http_error_407__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "ProxyDigestAuthHandler__http_error_407__req_as_Request_wrong"
# subject = "urllib.request.ProxyDigestAuthHandler.http_error_407(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.ProxyDigestAuthHandler.http_error_407(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import ProxyDigestAuthHandler
obj = object.__new__(ProxyDigestAuthHandler)
try:
    obj.http_error_407(_W(), None, 0, "", None)  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/ProxyHandler__proxy_open__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_ProxyHandler__proxy_open__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "ProxyHandler__proxy_open__req_as_Request_wrong"
# subject = "urllib.request.ProxyHandler.proxy_open(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.ProxyHandler.proxy_open(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import ProxyHandler
obj = object.__new__(ProxyHandler)
try:
    obj.proxy_open(_W(), "", "")  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/Request__add_header__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_Request__add_header__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "Request__add_header__key_as_str_wrong"
# subject = "urllib.request.Request.add_header(key: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.Request.add_header(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import Request
obj = object.__new__(Request)
try:
    obj.add_header(12345, "")  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/Request__add_unredirected_header__key_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_Request__add_unredirected_header__key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "Request__add_unredirected_header__key_as_str_wrong"
# subject = "urllib.request.Request.add_unredirected_header(key: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.Request.add_unredirected_header(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import Request
obj = object.__new__(Request)
try:
    obj.add_unredirected_header(12345, "")  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/Request__get_header__header_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_Request__get_header__header_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "Request__get_header__header_name_as_str_wrong"
# subject = "urllib.request.Request.get_header(header_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.Request.get_header(header_name: str); call it with the wrong type.

typeshed contract: header_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import Request
obj = object.__new__(Request)
try:
    obj.get_header(12345)  # header_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/Request__has_header__header_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_Request__has_header__header_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "Request__has_header__header_name_as_str_wrong"
# subject = "urllib.request.Request.has_header(header_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.Request.has_header(header_name: str); call it with the wrong type.

typeshed contract: header_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import Request
obj = object.__new__(Request)
try:
    obj.has_header(12345)  # header_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/Request__init__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_Request__init__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "Request__init__url_as_str_wrong"
# subject = "urllib.request.Request.__init__(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.Request.__init__(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import Request
try:
    Request(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/Request__remove_header__header_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_Request__remove_header__header_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "Request__remove_header__header_name_as_str_wrong"
# subject = "urllib.request.Request.remove_header(header_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.Request.remove_header(header_name: str); call it with the wrong type.

typeshed contract: header_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import Request
obj = object.__new__(Request)
try:
    obj.remove_header(12345)  # header_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/Request__set_proxy__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_Request__set_proxy__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "Request__set_proxy__host_as_str_wrong"
# subject = "urllib.request.Request.set_proxy(host: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.Request.set_proxy(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import Request
obj = object.__new__(Request)
try:
    obj.set_proxy(12345, "")  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/URLopener__http_error__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_URLopener__http_error__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "URLopener__http_error__url_as_str_wrong"
# subject = "urllib.request.URLopener.http_error(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.URLopener.http_error(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import URLopener
obj = object.__new__(URLopener)
try:
    obj.http_error(12345, None, 0, "", None)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/URLopener__http_error_default__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_URLopener__http_error_default__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "URLopener__http_error_default__url_as_str_wrong"
# subject = "urllib.request.URLopener.http_error_default(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.URLopener.http_error_default(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import URLopener
obj = object.__new__(URLopener)
try:
    obj.http_error_default(12345, None, 0, "", None)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/URLopener__open__fullurl_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_URLopener__open__fullurl_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "URLopener__open__fullurl_as_str_wrong"
# subject = "urllib.request.URLopener.open(fullurl: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.URLopener.open(fullurl: str); call it with the wrong type.

typeshed contract: fullurl is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import URLopener
obj = object.__new__(URLopener)
try:
    obj.open(12345)  # fullurl: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/URLopener__open_data__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_URLopener__open_data__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "URLopener__open_data__url_as_str_wrong"
# subject = "urllib.request.URLopener.open_data(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.URLopener.open_data(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import URLopener
obj = object.__new__(URLopener)
try:
    obj.open_data(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/URLopener__open_file__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_URLopener__open_file__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "URLopener__open_file__url_as_str_wrong"
# subject = "urllib.request.URLopener.open_file(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.URLopener.open_file(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import URLopener
obj = object.__new__(URLopener)
try:
    obj.open_file(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/URLopener__open_ftp__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_URLopener__open_ftp__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "URLopener__open_ftp__url_as_str_wrong"
# subject = "urllib.request.URLopener.open_ftp(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.URLopener.open_ftp(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import URLopener
obj = object.__new__(URLopener)
try:
    obj.open_ftp(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/URLopener__open_http__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_URLopener__open_http__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "URLopener__open_http__url_as_str_wrong"
# subject = "urllib.request.URLopener.open_http(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.URLopener.open_http(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import URLopener
obj = object.__new__(URLopener)
try:
    obj.open_http(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/URLopener__open_https__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_URLopener__open_https__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "URLopener__open_https__url_as_str_wrong"
# subject = "urllib.request.URLopener.open_https(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.URLopener.open_https(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import URLopener
obj = object.__new__(URLopener)
try:
    obj.open_https(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/URLopener__open_local_file__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_URLopener__open_local_file__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "URLopener__open_local_file__url_as_str_wrong"
# subject = "urllib.request.URLopener.open_local_file(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.URLopener.open_local_file(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import URLopener
obj = object.__new__(URLopener)
try:
    obj.open_local_file(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/URLopener__open_unknown__fullurl_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_URLopener__open_unknown__fullurl_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "URLopener__open_unknown__fullurl_as_str_wrong"
# subject = "urllib.request.URLopener.open_unknown(fullurl: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.URLopener.open_unknown(fullurl: str); call it with the wrong type.

typeshed contract: fullurl is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import URLopener
obj = object.__new__(URLopener)
try:
    obj.open_unknown(12345)  # fullurl: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/URLopener__open_unknown_proxy__proxy_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_URLopener__open_unknown_proxy__proxy_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "URLopener__open_unknown_proxy__proxy_as_str_wrong"
# subject = "urllib.request.URLopener.open_unknown_proxy(proxy: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.URLopener.open_unknown_proxy(proxy: str); call it with the wrong type.

typeshed contract: proxy is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import URLopener
obj = object.__new__(URLopener)
try:
    obj.open_unknown_proxy(12345, "")  # proxy: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/URLopener__retrieve__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_URLopener__retrieve__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "URLopener__retrieve__url_as_str_wrong"
# subject = "urllib.request.URLopener.retrieve(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.URLopener.retrieve(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import URLopener
obj = object.__new__(URLopener)
try:
    obj.retrieve(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/UnknownHandler__unknown_open__req_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_UnknownHandler__unknown_open__req_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "UnknownHandler__unknown_open__req_as_Request_wrong"
# subject = "urllib.request.UnknownHandler.unknown_open(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.UnknownHandler.unknown_open(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import UnknownHandler
obj = object.__new__(UnknownHandler)
try:
    obj.unknown_open(_W())  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/ftpwrapper__init__user_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_ftpwrapper__init__user_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "ftpwrapper__init__user_as_str_wrong"
# subject = "urllib.request.ftpwrapper.__init__(user: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.ftpwrapper.__init__(user: str); call it with the wrong type.

typeshed contract: user is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import ftpwrapper
try:
    ftpwrapper(12345, "", "", 0, "")  # user: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/ftpwrapper__retrfile__file_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_ftpwrapper__retrfile__file_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "ftpwrapper__retrfile__file_as_str_wrong"
# subject = "urllib.request.ftpwrapper.retrfile(file: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.ftpwrapper.retrfile(file: str); call it with the wrong type.

typeshed contract: file is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import ftpwrapper
obj = object.__new__(ftpwrapper)
try:
    obj.retrfile(12345, "")  # file: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/install_opener__opener_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_install_opener__opener_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "install_opener__opener_as_typed_wrong"
# subject = "urllib.request.install_opener(opener: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.install_opener(opener: typed); call it with the wrong type.

typeshed contract: opener is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import install_opener
try:
    install_opener(_W())  # opener: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/parse_http_list__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_parse_http_list__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "parse_http_list__s_as_str_wrong"
# subject = "urllib.request.parse_http_list(s: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.parse_http_list(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import parse_http_list
try:
    parse_http_list(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/pathname2url__pathname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_pathname2url__pathname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "pathname2url__pathname_as_str_wrong"
# subject = "urllib.request.pathname2url(pathname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.pathname2url(pathname: str); call it with the wrong type.

typeshed contract: pathname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import pathname2url
try:
    pathname2url(12345)  # pathname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/proxy_bypass__host_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_proxy_bypass__host_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "proxy_bypass__host_as_str_wrong"
# subject = "urllib.request.proxy_bypass(host: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.proxy_bypass(host: str); call it with the wrong type.

typeshed contract: host is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import proxy_bypass
try:
    proxy_bypass(12345)  # host: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/url2pathname__pathname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_url2pathname__pathname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "url2pathname__pathname_as_str_wrong"
# subject = "urllib.request.url2pathname(pathname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.url2pathname(pathname: str); call it with the wrong type.

typeshed contract: pathname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import url2pathname
try:
    url2pathname(12345)  # pathname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_request/urlretrieve__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_request_urlretrieve__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "urlretrieve__url_as_str_wrong"
# subject = "urllib.request.urlretrieve(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.urlretrieve(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.request import urlretrieve
try:
    urlretrieve(12345)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
