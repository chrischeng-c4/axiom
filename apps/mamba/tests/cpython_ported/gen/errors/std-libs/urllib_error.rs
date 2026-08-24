use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/urllib_error/httperror_is_oserror.py`.
#[test]
fn test_gen_errors_std_libs_urllib_error_httperror_is_oserror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "errors"
# case = "httperror_is_oserror"
# subject = "urllib.error.HTTPError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.HTTPError: HTTPError is a subclass of OSError and is catchable as OSError when raised"""
from urllib.error import HTTPError

assert issubclass(HTTPError, OSError), "HTTPError < OSError"

caught = False
try:
    raise HTTPError("http://x.com/", 404, "Not Found", {}, None)
except OSError as e:
    caught = True
    assert isinstance(e, HTTPError), type(e)
assert caught, "HTTPError raised and caught as OSError"
print("httperror_is_oserror OK")
"###);
    assert_output(&out, r###"httperror_is_oserror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/urllib_error/httperror_is_urlerror.py`.
#[test]
fn test_gen_errors_std_libs_urllib_error_httperror_is_urlerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "errors"
# case = "httperror_is_urlerror"
# subject = "urllib.error.HTTPError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.HTTPError: HTTPError is a subclass of URLError and is catchable as URLError when raised"""
from urllib.error import URLError, HTTPError

assert issubclass(HTTPError, URLError), "HTTPError < URLError"

caught = False
try:
    raise HTTPError("http://x.com/", 403, "Forbidden", {}, None)
except URLError as e:
    caught = True
    assert isinstance(e, HTTPError), type(e)
assert caught, "HTTPError raised and caught as URLError"
print("httperror_is_urlerror OK")
"###);
    assert_output(&out, r###"httperror_is_urlerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/urllib_error/urlerror_is_oserror.py`.
#[test]
fn test_gen_errors_std_libs_urllib_error_urlerror_is_oserror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_error"
# dimension = "errors"
# case = "urlerror_is_oserror"
# subject = "urllib.error.URLError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.URLError: URLError is a subclass of OSError (documented exception hierarchy root) and is raiseable/catchable as OSError"""
from urllib.error import URLError

assert issubclass(URLError, OSError), "URLError < OSError"

caught = False
try:
    raise URLError("connection refused")
except OSError as e:
    caught = True
    assert isinstance(e, URLError), type(e)
assert caught, "URLError raised and caught as OSError"
print("urlerror_is_oserror OK")
"###);
    assert_output(&out, r###"urlerror_is_oserror OK
"###);
}
