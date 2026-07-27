use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/urllib_error/httperror_core_attributes.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_error_httperror_core_attributes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "behavior"
# case = "httperror_core_attributes"
# subject = "urllib.error.HTTPError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.error.HTTPError: HTTPError(url, code, msg, hdrs, fp) exposes .code, .url, .msg verbatim"""
from urllib.error import HTTPError
import io

e = HTTPError("http://x.com/api", 500, "Server Error", {}, io.BytesIO(b"err"))
assert e.code == 500, repr(e.code)
assert e.url == "http://x.com/api", repr(e.url)
assert e.msg == "Server Error", repr(e.msg)
print("httperror_core_attributes OK")
"###);
    assert_output(&out, r###"httperror_core_attributes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_error/httperror_read_returns_body.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_error_httperror_read_returns_body() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "behavior"
# case = "httperror_read_returns_body"
# subject = "urllib.error.HTTPError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.error.HTTPError: HTTPError.read() returns the full body bytes from the fp passed at construction"""
from urllib.error import HTTPError
import io

body = b"Internal Server Error body"
e = HTTPError("http://x.com/", 500, "Error", {}, io.BytesIO(body))
assert e.read() == body, "HTTPError.read returns the fp body"
print("httperror_read_returns_body OK")
"###);
    assert_output(&out, r###"httperror_read_returns_body OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_error/httperror_reason_aliases_msg.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_error_httperror_reason_aliases_msg() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "behavior"
# case = "httperror_reason_aliases_msg"
# subject = "urllib.error.HTTPError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.error.HTTPError: HTTPError.reason aliases .msg and HTTPError.filename aliases .url per CPython 3.12"""
from urllib.error import HTTPError

e = HTTPError("http://example.com/missing", 404, "Not Found", None, None)
assert e.reason == e.msg == "Not Found", (repr(e.reason), repr(e.msg))
assert e.filename == e.url == "http://example.com/missing", (repr(e.filename), repr(e.url))
print("httperror_reason_aliases_msg OK")
"###);
    assert_output(&out, r###"httperror_reason_aliases_msg OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_error/urlerror_filename_second_arg.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_error_urlerror_filename_second_arg() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "behavior"
# case = "urlerror_filename_second_arg"
# subject = "urllib.error.URLError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.error.URLError: URLError(reason, filename) records the optional second positional as .filename"""
from urllib.error import URLError

e = URLError("not found", "http://x/y")
assert e.reason == "not found", repr(e.reason)
assert e.filename == "http://x/y", repr(e.filename)
print("urlerror_filename_second_arg OK")
"###);
    assert_output(&out, r###"urlerror_filename_second_arg OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_error/urlerror_reason_is_exception_object.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_error_urlerror_reason_is_exception_object() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "behavior"
# case = "urlerror_reason_is_exception_object"
# subject = "urllib.error.URLError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.error.URLError: URLError(exc) stores the exception object identically as .reason (an OSError instance is kept by identity)"""
from urllib.error import URLError

inner = ConnectionRefusedError(111, "Connection refused")
e = URLError(inner)
assert e.reason is inner, "reason is the exact exception object"
assert isinstance(e.reason, OSError), "reason is an OSError"
print("urlerror_reason_is_exception_object OK")
"###);
    assert_output(&out, r###"urlerror_reason_is_exception_object OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_error/urlerror_reason_preserved.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_error_urlerror_reason_preserved() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "behavior"
# case = "urlerror_reason_preserved"
# subject = "urllib.error.URLError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.error.URLError: URLError(reason) preserves the reason string verbatim on the .reason attribute"""
from urllib.error import URLError

e = URLError("DNS resolution failed")
assert e.reason == "DNS resolution failed", repr(e.reason)
print("urlerror_reason_preserved OK")
"###);
    assert_output(&out, r###"urlerror_reason_preserved OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/urllib_error/urlerror_str_includes_reason.py`.
#[test]
fn test_gen_behavior_std_libs_urllib_error_urlerror_str_includes_reason() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "behavior"
# case = "urlerror_str_includes_reason"
# subject = "urllib.error.URLError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.error.URLError: str(URLError(reason)) renders an <urlopen error ...> message that contains the reason text"""
from urllib.error import URLError

s = str(URLError("timeout"))
assert "timeout" in s, repr(s)
print("urlerror_str_includes_reason OK")
"###);
    assert_output(&out, r###"urlerror_str_includes_reason OK
"###);
}
