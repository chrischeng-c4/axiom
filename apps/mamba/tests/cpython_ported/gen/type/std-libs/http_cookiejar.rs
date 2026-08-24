use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/CookieJar__add_cookie_header__request_as_Request_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_CookieJar__add_cookie_header__request_as_Request_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "CookieJar__add_cookie_header__request_as_Request_wrong"
# subject = "http.cookiejar.CookieJar.add_cookie_header(request: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.CookieJar.add_cookie_header(request: Request); call it with the wrong type.

typeshed contract: request is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import CookieJar
obj = object.__new__(CookieJar)
try:
    obj.add_cookie_header(_W())  # request: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/CookieJar__clear__domain_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_CookieJar__clear__domain_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "CookieJar__clear__domain_as_typed_wrong"
# subject = "http.cookiejar.CookieJar.clear(domain: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.CookieJar.clear(domain: typed); call it with the wrong type.

typeshed contract: domain is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import CookieJar
obj = object.__new__(CookieJar)
try:
    obj.clear(_W())  # domain: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/CookieJar__extract_cookies__response_as_HTTPResponse_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_CookieJar__extract_cookies__response_as_HTTPResponse_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "CookieJar__extract_cookies__response_as_HTTPResponse_wrong"
# subject = "http.cookiejar.CookieJar.extract_cookies(response: HTTPResponse)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.CookieJar.extract_cookies(response: HTTPResponse); call it with the wrong type.

typeshed contract: response is HTTPResponse. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import CookieJar
obj = object.__new__(CookieJar)
try:
    obj.extract_cookies(_W(), None)  # response: HTTPResponse <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/CookieJar__init__policy_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_CookieJar__init__policy_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "CookieJar__init__policy_as_typed_wrong"
# subject = "http.cookiejar.CookieJar.__init__(policy: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.CookieJar.__init__(policy: typed); call it with the wrong type.

typeshed contract: policy is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import CookieJar
try:
    CookieJar(_W())  # policy: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/CookieJar__make_cookies__response_as_HTTPResponse_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_CookieJar__make_cookies__response_as_HTTPResponse_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "CookieJar__make_cookies__response_as_HTTPResponse_wrong"
# subject = "http.cookiejar.CookieJar.make_cookies(response: HTTPResponse)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.CookieJar.make_cookies(response: HTTPResponse); call it with the wrong type.

typeshed contract: response is HTTPResponse. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import CookieJar
obj = object.__new__(CookieJar)
try:
    obj.make_cookies(_W(), None)  # response: HTTPResponse <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/CookieJar__set_cookie__cookie_as_Cookie_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_CookieJar__set_cookie__cookie_as_Cookie_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "CookieJar__set_cookie__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.CookieJar.set_cookie(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.CookieJar.set_cookie(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import CookieJar
obj = object.__new__(CookieJar)
try:
    obj.set_cookie(_W())  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/CookieJar__set_cookie_if_ok__cookie_as_Cookie_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_CookieJar__set_cookie_if_ok__cookie_as_Cookie_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "CookieJar__set_cookie_if_ok__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.CookieJar.set_cookie_if_ok(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.CookieJar.set_cookie_if_ok(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import CookieJar
obj = object.__new__(CookieJar)
try:
    obj.set_cookie_if_ok(_W(), None)  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/CookieJar__set_policy__policy_as_CookiePolicy_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_CookieJar__set_policy__policy_as_CookiePolicy_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "CookieJar__set_policy__policy_as_CookiePolicy_wrong"
# subject = "http.cookiejar.CookieJar.set_policy(policy: CookiePolicy)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.CookieJar.set_policy(policy: CookiePolicy); call it with the wrong type.

typeshed contract: policy is CookiePolicy. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import CookieJar
obj = object.__new__(CookieJar)
try:
    obj.set_policy(_W())  # policy: CookiePolicy <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/CookiePolicy__domain_return_ok__domain_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_CookiePolicy__domain_return_ok__domain_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "CookiePolicy__domain_return_ok__domain_as_str_wrong"
# subject = "http.cookiejar.CookiePolicy.domain_return_ok(domain: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.CookiePolicy.domain_return_ok(domain: str); call it with the wrong type.

typeshed contract: domain is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.cookiejar import CookiePolicy
obj = object.__new__(CookiePolicy)
try:
    obj.domain_return_ok(12345, None)  # domain: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/CookiePolicy__path_return_ok__path_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_CookiePolicy__path_return_ok__path_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "CookiePolicy__path_return_ok__path_as_str_wrong"
# subject = "http.cookiejar.CookiePolicy.path_return_ok(path: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.CookiePolicy.path_return_ok(path: str); call it with the wrong type.

typeshed contract: path is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.cookiejar import CookiePolicy
obj = object.__new__(CookiePolicy)
try:
    obj.path_return_ok(12345, None)  # path: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/CookiePolicy__return_ok__cookie_as_Cookie_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_CookiePolicy__return_ok__cookie_as_Cookie_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "CookiePolicy__return_ok__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.CookiePolicy.return_ok(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.CookiePolicy.return_ok(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import CookiePolicy
obj = object.__new__(CookiePolicy)
try:
    obj.return_ok(_W(), None)  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/CookiePolicy__set_ok__cookie_as_Cookie_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_CookiePolicy__set_ok__cookie_as_Cookie_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "CookiePolicy__set_ok__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.CookiePolicy.set_ok(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.CookiePolicy.set_ok(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import CookiePolicy
obj = object.__new__(CookiePolicy)
try:
    obj.set_ok(_W(), None)  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/Cookie__get_nonstandard_attr__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_Cookie__get_nonstandard_attr__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "Cookie__get_nonstandard_attr__name_as_str_wrong"
# subject = "http.cookiejar.Cookie.get_nonstandard_attr(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.Cookie.get_nonstandard_attr(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.cookiejar import Cookie
obj = object.__new__(Cookie)
try:
    obj.get_nonstandard_attr(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/Cookie__has_nonstandard_attr__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_Cookie__has_nonstandard_attr__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "Cookie__has_nonstandard_attr__name_as_str_wrong"
# subject = "http.cookiejar.Cookie.has_nonstandard_attr(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.Cookie.has_nonstandard_attr(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.cookiejar import Cookie
obj = object.__new__(Cookie)
try:
    obj.has_nonstandard_attr(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/Cookie__init__version_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_Cookie__init__version_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "Cookie__init__version_as_typed_wrong"
# subject = "http.cookiejar.Cookie.__init__(version: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.Cookie.__init__(version: typed); call it with the wrong type.

typeshed contract: version is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import Cookie
try:
    Cookie(_W(), "", None, None, True, "", True, True, "", True, True, None, True, None, None, None)  # version: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/Cookie__is_expired__now_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_Cookie__is_expired__now_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "Cookie__is_expired__now_as_typed_wrong"
# subject = "http.cookiejar.Cookie.is_expired(now: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.Cookie.is_expired(now: typed); call it with the wrong type.

typeshed contract: now is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import Cookie
obj = object.__new__(Cookie)
try:
    obj.is_expired(_W())  # now: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/Cookie__set_nonstandard_attr__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_Cookie__set_nonstandard_attr__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "Cookie__set_nonstandard_attr__name_as_str_wrong"
# subject = "http.cookiejar.Cookie.set_nonstandard_attr(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.Cookie.set_nonstandard_attr(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.cookiejar import Cookie
obj = object.__new__(Cookie)
try:
    obj.set_nonstandard_attr(12345, "")  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/DefaultCookiePolicy__init__blocked_domains_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_DefaultCookiePolicy__init__blocked_domains_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__init__blocked_domains_as_typed_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.__init__(blocked_domains: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.__init__(blocked_domains: typed); call it with the wrong type.

typeshed contract: blocked_domains is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
try:
    DefaultCookiePolicy(_W())  # blocked_domains: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/DefaultCookiePolicy__is_blocked__domain_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_DefaultCookiePolicy__is_blocked__domain_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__is_blocked__domain_as_str_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.is_blocked(domain: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.is_blocked(domain: str); call it with the wrong type.

typeshed contract: domain is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.is_blocked(12345)  # domain: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/DefaultCookiePolicy__is_not_allowed__domain_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_DefaultCookiePolicy__is_not_allowed__domain_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__is_not_allowed__domain_as_str_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.is_not_allowed(domain: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.is_not_allowed(domain: str); call it with the wrong type.

typeshed contract: domain is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.is_not_allowed(12345)  # domain: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/DefaultCookiePolicy__return_ok_domain__cookie_as_Cookie_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_DefaultCookiePolicy__return_ok_domain__cookie_as_Cookie_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__return_ok_domain__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.return_ok_domain(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.return_ok_domain(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.return_ok_domain(_W(), None)  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/DefaultCookiePolicy__return_ok_expires__cookie_as_Cookie_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_DefaultCookiePolicy__return_ok_expires__cookie_as_Cookie_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__return_ok_expires__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.return_ok_expires(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.return_ok_expires(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.return_ok_expires(_W(), None)  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/DefaultCookiePolicy__return_ok_port__cookie_as_Cookie_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_DefaultCookiePolicy__return_ok_port__cookie_as_Cookie_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__return_ok_port__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.return_ok_port(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.return_ok_port(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.return_ok_port(_W(), None)  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/DefaultCookiePolicy__return_ok_secure__cookie_as_Cookie_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_DefaultCookiePolicy__return_ok_secure__cookie_as_Cookie_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__return_ok_secure__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.return_ok_secure(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.return_ok_secure(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.return_ok_secure(_W(), None)  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/DefaultCookiePolicy__return_ok_verifiability__cookie_as_Cookie_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_DefaultCookiePolicy__return_ok_verifiability__cookie_as_Cookie_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__return_ok_verifiability__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.return_ok_verifiability(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.return_ok_verifiability(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.return_ok_verifiability(_W(), None)  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/DefaultCookiePolicy__return_ok_version__cookie_as_Cookie_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_DefaultCookiePolicy__return_ok_version__cookie_as_Cookie_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__return_ok_version__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.return_ok_version(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.return_ok_version(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.return_ok_version(_W(), None)  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/DefaultCookiePolicy__set_ok_domain__cookie_as_Cookie_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_DefaultCookiePolicy__set_ok_domain__cookie_as_Cookie_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__set_ok_domain__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.set_ok_domain(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.set_ok_domain(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.set_ok_domain(_W(), None)  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/DefaultCookiePolicy__set_ok_name__cookie_as_Cookie_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_DefaultCookiePolicy__set_ok_name__cookie_as_Cookie_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__set_ok_name__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.set_ok_name(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.set_ok_name(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.set_ok_name(_W(), None)  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/DefaultCookiePolicy__set_ok_path__cookie_as_Cookie_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_DefaultCookiePolicy__set_ok_path__cookie_as_Cookie_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__set_ok_path__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.set_ok_path(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.set_ok_path(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.set_ok_path(_W(), None)  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/DefaultCookiePolicy__set_ok_port__cookie_as_Cookie_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_DefaultCookiePolicy__set_ok_port__cookie_as_Cookie_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__set_ok_port__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.set_ok_port(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.set_ok_port(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.set_ok_port(_W(), None)  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/DefaultCookiePolicy__set_ok_verifiability__cookie_as_Cookie_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_DefaultCookiePolicy__set_ok_verifiability__cookie_as_Cookie_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__set_ok_verifiability__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.set_ok_verifiability(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.set_ok_verifiability(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.set_ok_verifiability(_W(), None)  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/DefaultCookiePolicy__set_ok_version__cookie_as_Cookie_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_DefaultCookiePolicy__set_ok_version__cookie_as_Cookie_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "DefaultCookiePolicy__set_ok_version__cookie_as_Cookie_wrong"
# subject = "http.cookiejar.DefaultCookiePolicy.set_ok_version(cookie: Cookie)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.DefaultCookiePolicy.set_ok_version(cookie: Cookie); call it with the wrong type.

typeshed contract: cookie is Cookie. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import DefaultCookiePolicy
obj = object.__new__(DefaultCookiePolicy)
try:
    obj.set_ok_version(_W(), None)  # cookie: Cookie <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/FileCookieJar__init__filename_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_FileCookieJar__init__filename_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "FileCookieJar__init__filename_as_typed_wrong"
# subject = "http.cookiejar.FileCookieJar.__init__(filename: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.FileCookieJar.__init__(filename: typed); call it with the wrong type.

typeshed contract: filename is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import FileCookieJar
try:
    FileCookieJar(_W())  # filename: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/FileCookieJar__load__filename_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_FileCookieJar__load__filename_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "FileCookieJar__load__filename_as_typed_wrong"
# subject = "http.cookiejar.FileCookieJar.load(filename: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.FileCookieJar.load(filename: typed); call it with the wrong type.

typeshed contract: filename is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import FileCookieJar
obj = object.__new__(FileCookieJar)
try:
    obj.load(_W())  # filename: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/FileCookieJar__revert__filename_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_FileCookieJar__revert__filename_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "FileCookieJar__revert__filename_as_typed_wrong"
# subject = "http.cookiejar.FileCookieJar.revert(filename: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.FileCookieJar.revert(filename: typed); call it with the wrong type.

typeshed contract: filename is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import FileCookieJar
obj = object.__new__(FileCookieJar)
try:
    obj.revert(_W())  # filename: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/http_cookiejar/FileCookieJar__save__filename_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_http_cookiejar_FileCookieJar__save__filename_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "type"
# case = "FileCookieJar__save__filename_as_typed_wrong"
# subject = "http.cookiejar.FileCookieJar.save(filename: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/cookiejar.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.cookiejar.FileCookieJar.save(filename: typed); call it with the wrong type.

typeshed contract: filename is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.cookiejar import FileCookieJar
obj = object.__new__(FileCookieJar)
try:
    obj.save(_W())  # filename: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
