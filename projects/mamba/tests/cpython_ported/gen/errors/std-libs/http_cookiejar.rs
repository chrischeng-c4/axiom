use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/http_cookiejar/load_malformed_lwp_raises_loaderror.py`.
#[test]
fn test_gen_errors_std_libs_http_cookiejar_load_malformed_lwp_raises_loaderror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "errors"
# case = "load_malformed_lwp_raises_loaderror"
# subject = "http.cookiejar.LWPCookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.LWPCookieJar: LWPCookieJar.load of a file that is not LWP cookie format raises http.cookiejar.LoadError (staged via a tempfile)"""
import http.cookiejar
import os
import tempfile

with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as _f:
    _f.write("bad LWP content\n")
    _bad_path = _f.name
try:
    _raised = False
    try:
        http.cookiejar.LWPCookieJar(_bad_path).load()
    except http.cookiejar.LoadError:
        _raised = True
    assert _raised, "expected http.cookiejar.LoadError on malformed LWP file"
finally:
    os.unlink(_bad_path)

print("load_malformed_lwp_raises_loaderror OK")
"###);
    assert_output(&out, r###"load_malformed_lwp_raises_loaderror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/http_cookiejar/load_malformed_mozilla_raises_loaderror.py`.
#[test]
fn test_gen_errors_std_libs_http_cookiejar_load_malformed_mozilla_raises_loaderror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "errors"
# case = "load_malformed_mozilla_raises_loaderror"
# subject = "http.cookiejar.MozillaCookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.MozillaCookieJar: MozillaCookieJar.load of a file that is not a Netscape cookies.txt raises http.cookiejar.LoadError (staged via a tempfile)"""
import http.cookiejar
import os
import tempfile

with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as _f:
    _f.write("not a cookies.txt file format\n")
    _bad_path = _f.name
try:
    _raised = False
    try:
        http.cookiejar.MozillaCookieJar().load(_bad_path)
    except http.cookiejar.LoadError:
        _raised = True
    assert _raised, "expected http.cookiejar.LoadError on malformed Mozilla file"
finally:
    os.unlink(_bad_path)

print("load_malformed_mozilla_raises_loaderror OK")
"###);
    assert_output(&out, r###"load_malformed_mozilla_raises_loaderror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/http_cookiejar/load_missing_file_raises.py`.
#[test]
fn test_gen_errors_std_libs_http_cookiejar_load_missing_file_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "errors"
# case = "load_missing_file_raises"
# subject = "http.cookiejar.MozillaCookieJar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.MozillaCookieJar: load_missing_file_raises (errors)."""
import http.cookiejar

_raised = False
try:
    http.cookiejar.MozillaCookieJar().load('/no/such/cookies.txt')
except FileNotFoundError:
    _raised = True
assert _raised, "load_missing_file_raises: expected FileNotFoundError"
print("load_missing_file_raises OK")
"###);
    assert_output(&out, r###"load_missing_file_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/http_cookiejar/loaderror_is_oserror_subclass.py`.
#[test]
fn test_gen_errors_std_libs_http_cookiejar_loaderror_is_oserror_subclass() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "errors"
# case = "loaderror_is_oserror_subclass"
# subject = "http.cookiejar.LoadError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.LoadError: http.cookiejar.LoadError is a subclass of OSError"""
import http.cookiejar

assert issubclass(http.cookiejar.LoadError, OSError), "LoadError < OSError"

print("loaderror_is_oserror_subclass OK")
"###);
    assert_output(&out, r###"loaderror_is_oserror_subclass OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/http_cookiejar/lwp_load_missing_is_plain_oserror_not_loaderror.py`.
#[test]
fn test_gen_errors_std_libs_http_cookiejar_lwp_load_missing_is_plain_oserror_not_loaderror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "errors"
# case = "lwp_load_missing_is_plain_oserror_not_loaderror"
# subject = "http.cookiejar.LWPCookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.LWPCookieJar: loading a nonexistent file raises a plain OSError (FileNotFoundError), not LoadError; the LoadError branch must not fire"""
import http.cookiejar

_jar = http.cookiejar.LWPCookieJar()
_raised_oserror = False
try:
    _jar.load(filename="this-file-should-not-exist-12345.txt")
except http.cookiejar.LoadError:
    raise AssertionError("missing file must not raise LoadError")
except OSError as _exc:
    assert _exc.__class__ is not http.cookiejar.LoadError, "plain OSError expected"
    _raised_oserror = True
assert _raised_oserror, "expected a plain OSError for the missing file"

print("lwp_load_missing_is_plain_oserror_not_loaderror OK")
"###);
    assert_output(&out, r###"lwp_load_missing_is_plain_oserror_not_loaderror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/http_cookiejar/save_no_filename_raises.py`.
#[test]
fn test_gen_errors_std_libs_http_cookiejar_save_no_filename_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "errors"
# case = "save_no_filename_raises"
# subject = "http.cookiejar.MozillaCookieJar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.MozillaCookieJar: save_no_filename_raises (errors)."""
import http.cookiejar

_raised = False
try:
    http.cookiejar.MozillaCookieJar().save()
except ValueError:
    _raised = True
assert _raised, "save_no_filename_raises: expected ValueError"
print("save_no_filename_raises OK")
"###);
    assert_output(&out, r###"save_no_filename_raises OK
"###);
}
