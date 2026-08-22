use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/urllib_error/ContentTooShortError__init__message_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_error_ContentTooShortError__init__message_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "type"
# case = "ContentTooShortError__init__message_as_str_wrong"
# subject = "urllib.error.ContentTooShortError.__init__(message: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/error.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.error.ContentTooShortError.__init__(message: str); call it with the wrong type.

typeshed contract: message is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.error import ContentTooShortError
try:
    ContentTooShortError(12345, None)  # message: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_error/HTTPError__init__url_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_error_HTTPError__init__url_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "type"
# case = "HTTPError__init__url_as_str_wrong"
# subject = "urllib.error.HTTPError.__init__(url: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/error.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.error.HTTPError.__init__(url: str); call it with the wrong type.

typeshed contract: url is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from urllib.error import HTTPError
try:
    HTTPError(12345, 0, "", None, None)  # url: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/urllib_error/URLError__init__reason_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_urllib_error_URLError__init__reason_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "type"
# case = "URLError__init__reason_as_typed_wrong"
# subject = "urllib.error.URLError.__init__(reason: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/error.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.error.URLError.__init__(reason: typed); call it with the wrong type.

typeshed contract: reason is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.error import URLError
try:
    URLError(_W())  # reason: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
