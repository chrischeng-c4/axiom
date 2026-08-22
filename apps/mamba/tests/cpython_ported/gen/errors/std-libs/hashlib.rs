use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/hashlib/new_unknown_algorithm_raises.py`.
#[test]
fn test_gen_errors_std_libs_hashlib_new_unknown_algorithm_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "errors"
# case = "new_unknown_algorithm_raises"
# subject = "hashlib.new"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.new: new_unknown_algorithm_raises (errors)."""
import hashlib

_raised = False
try:
    hashlib.new('no_such_algorithm')
except ValueError:
    _raised = True
assert _raised, "new_unknown_algorithm_raises: expected ValueError"
print("new_unknown_algorithm_raises OK")
"###);
    assert_output(&out, r###"new_unknown_algorithm_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/hashlib/pbkdf2_unknown_hash_raises.py`.
#[test]
fn test_gen_errors_std_libs_hashlib_pbkdf2_unknown_hash_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "errors"
# case = "pbkdf2_unknown_hash_raises"
# subject = "hashlib.pbkdf2_hmac"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.pbkdf2_hmac: pbkdf2_unknown_hash_raises (errors)."""
import hashlib

_raised = False
try:
    hashlib.pbkdf2_hmac('no_such_hash', b'password', b'salt', 1)
except ValueError:
    _raised = True
assert _raised, "pbkdf2_unknown_hash_raises: expected ValueError"
print("pbkdf2_unknown_hash_raises OK")
"###);
    assert_output(&out, r###"pbkdf2_unknown_hash_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/hashlib/pbkdf2_zero_iterations_raises.py`.
#[test]
fn test_gen_errors_std_libs_hashlib_pbkdf2_zero_iterations_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "errors"
# case = "pbkdf2_zero_iterations_raises"
# subject = "hashlib.pbkdf2_hmac"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.pbkdf2_hmac: pbkdf2_zero_iterations_raises (errors)."""
import hashlib

_raised = False
try:
    hashlib.pbkdf2_hmac('sha256', b'pw', b'salt', 0)
except ValueError:
    _raised = True
assert _raised, "pbkdf2_zero_iterations_raises: expected ValueError"
print("pbkdf2_zero_iterations_raises OK")
"###);
    assert_output(&out, r###"pbkdf2_zero_iterations_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/hashlib/shake_hexdigest_without_length_raises.py`.
#[test]
fn test_gen_errors_std_libs_hashlib_shake_hexdigest_without_length_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "errors"
# case = "shake_hexdigest_without_length_raises"
# subject = "hashlib.shake_128"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.shake_128: shake_hexdigest_without_length_raises (errors)."""
import hashlib

_raised = False
try:
    hashlib.shake_128(b'data').hexdigest()
except TypeError:
    _raised = True
assert _raised, "shake_hexdigest_without_length_raises: expected TypeError"
print("shake_hexdigest_without_length_raises OK")
"###);
    assert_output(&out, r###"shake_hexdigest_without_length_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/hashlib/update_int_raises.py`.
#[test]
fn test_gen_errors_std_libs_hashlib_update_int_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "errors"
# case = "update_int_raises"
# subject = "hashlib.sha256"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.sha256: update_int_raises (errors)."""
import hashlib

_raised = False
try:
    hashlib.sha256().update(123)
except TypeError:
    _raised = True
assert _raised, "update_int_raises: expected TypeError"
print("update_int_raises OK")
"###);
    assert_output(&out, r###"update_int_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/hashlib/update_str_raises.py`.
#[test]
fn test_gen_errors_std_libs_hashlib_update_str_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "errors"
# case = "update_str_raises"
# subject = "hashlib.sha256"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.sha256: update_str_raises (errors)."""
import hashlib

_raised = False
try:
    hashlib.sha256().update('not bytes')
except TypeError:
    _raised = True
assert _raised, "update_str_raises: expected TypeError"
print("update_str_raises OK")
"###);
    assert_output(&out, r###"update_str_raises OK
"###);
}
