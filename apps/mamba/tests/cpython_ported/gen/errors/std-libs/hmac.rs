use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/hmac/empty_digestmod_raises.py`.
#[test]
fn test_gen_errors_std_libs_hmac_empty_digestmod_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "errors"
# case = "empty_digestmod_raises"
# subject = "hmac.HMAC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.HMAC: empty_digestmod_raises (errors)."""
import hmac

_raised = False
try:
    hmac.HMAC(b"key", msg=b"msg", digestmod="")
except TypeError:
    _raised = True
assert _raised, "empty_digestmod_raises: expected TypeError"
print("empty_digestmod_raises OK")
"###);
    assert_output(&out, r###"empty_digestmod_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/hmac/mixed_type_compare_raises.py`.
#[test]
fn test_gen_errors_std_libs_hmac_mixed_type_compare_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "errors"
# case = "mixed_type_compare_raises"
# subject = "hmac.compare_digest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.compare_digest: mixed_type_compare_raises (errors)."""
import hmac

_raised = False
try:
    hmac.compare_digest("string", b"bytes")
except TypeError:
    _raised = True
assert _raised, "mixed_type_compare_raises: expected TypeError"
print("mixed_type_compare_raises OK")
"###);
    assert_output(&out, r###"mixed_type_compare_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/hmac/none_digestmod_raises.py`.
#[test]
fn test_gen_errors_std_libs_hmac_none_digestmod_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "errors"
# case = "none_digestmod_raises"
# subject = "hmac.new"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.new: none_digestmod_raises (errors)."""
import hmac

_raised = False
try:
    hmac.new(b"key", b"msg", None)
except TypeError:
    _raised = True
assert _raised, "none_digestmod_raises: expected TypeError"
print("none_digestmod_raises OK")
"###);
    assert_output(&out, r###"none_digestmod_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/hmac/str_update_raises.py`.
#[test]
fn test_gen_errors_std_libs_hmac_str_update_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "errors"
# case = "str_update_raises"
# subject = "hmac.HMAC.update"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.HMAC.update: str_update_raises (errors)."""
import hmac

_raised = False
try:
    hmac.new(b"key", digestmod="sha256").update("not bytes")
except TypeError:
    _raised = True
assert _raised, "str_update_raises: expected TypeError"
print("str_update_raises OK")
"###);
    assert_output(&out, r###"str_update_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/hmac/unknown_digestmod_raises.py`.
#[test]
fn test_gen_errors_std_libs_hmac_unknown_digestmod_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "errors"
# case = "unknown_digestmod_raises"
# subject = "hmac.new"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.new: unknown_digestmod_raises (errors)."""
import hmac

_raised = False
try:
    hmac.new(b"key", b"msg", "no_such_hash")
except ValueError:
    _raised = True
assert _raised, "unknown_digestmod_raises: expected ValueError"
print("unknown_digestmod_raises OK")
"###);
    assert_output(&out, r###"unknown_digestmod_raises OK
"###);
}
