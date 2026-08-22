use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/mimetypes/read_missing_file_raises.py`.
#[test]
fn test_gen_errors_std_libs_mimetypes_read_missing_file_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "errors"
# case = "read_missing_file_raises"
# subject = "mimetypes.MimeTypes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.MimeTypes: read_missing_file_raises (errors)."""
import mimetypes

_raised = False
try:
    mimetypes.MimeTypes().read('/no/such/mime.types')
except FileNotFoundError:
    _raised = True
assert _raised, "read_missing_file_raises: expected FileNotFoundError"
print("read_missing_file_raises OK")
"###);
    assert_output(&out, r###"read_missing_file_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/mimetypes/unknown_extension_returns_none_none.py`.
#[test]
fn test_gen_errors_std_libs_mimetypes_unknown_extension_returns_none_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "errors"
# case = "unknown_extension_returns_none_none"
# subject = "mimetypes.guess_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.guess_type: an unregistered extension is not an error: guess_type returns (None, None) rather than raising"""
import mimetypes

_t, _e = mimetypes.guess_type("file.xyz_unknown_ext_123")
assert _t is None, f"unknown type = {_t!r}"
assert _e is None, f"unknown encoding = {_e!r}"
print("unknown_extension_returns_none_none OK")
"###);
    assert_output(&out, r###"unknown_extension_returns_none_none OK
"###);
}
