use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/urllib/contenttooshort_is_urlerror.py`.
#[test]
fn test_gen_errors_std_libs_urllib_contenttooshort_is_urlerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "errors"
# case = "contenttooshort_is_urlerror"
# subject = "urllib.error.ContentTooShortError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.ContentTooShortError: ContentTooShortError is a subclass of URLError"""
from urllib.error import URLError, ContentTooShortError

assert issubclass(ContentTooShortError, URLError), "ContentTooShortError < URLError"

print("contenttooshort_is_urlerror OK")
"###);
    assert_output(&out, r###"contenttooshort_is_urlerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/urllib/httperror_is_urlerror.py`.
#[test]
fn test_gen_errors_std_libs_urllib_httperror_is_urlerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "errors"
# case = "httperror_is_urlerror"
# subject = "urllib.error.HTTPError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.HTTPError: HTTPError is a subclass of URLError (and therefore of OSError)"""
from urllib.error import URLError, HTTPError

assert issubclass(HTTPError, URLError), "HTTPError < URLError"
assert issubclass(HTTPError, OSError), "HTTPError < OSError"

print("httperror_is_urlerror OK")
"###);
    assert_output(&out, r###"httperror_is_urlerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/urllib/quote_unencodable_char_unicodeerror.py`.
#[test]
fn test_gen_errors_std_libs_urllib_quote_unencodable_char_unicodeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "errors"
# case = "quote_unencodable_char_unicodeerror"
# subject = "urllib.parse.quote"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.quote: quote_unencodable_char_unicodeerror (errors)."""
from urllib.parse import quote

_raised = False
try:
    quote('\u6f22', encoding='latin-1')
except UnicodeEncodeError:
    _raised = True
assert _raised, "quote_unencodable_char_unicodeerror: expected UnicodeEncodeError"
print("quote_unencodable_char_unicodeerror OK")
"###);
    assert_output(&out, r###"quote_unencodable_char_unicodeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/urllib/urlerror_is_oserror.py`.
#[test]
fn test_gen_errors_std_libs_urllib_urlerror_is_oserror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "errors"
# case = "urlerror_is_oserror"
# subject = "urllib.error.URLError"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.error.URLError: URLError is a subclass of OSError (the documented exception hierarchy root)"""
from urllib.error import URLError

assert issubclass(URLError, OSError), "URLError < OSError"

print("urlerror_is_oserror OK")
"###);
    assert_output(&out, r###"urlerror_is_oserror OK
"###);
}
