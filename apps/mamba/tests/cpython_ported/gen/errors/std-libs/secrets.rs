use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/secrets/choice_empty_sequence_raises.py`.
#[test]
fn test_gen_errors_std_libs_secrets_choice_empty_sequence_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "errors"
# case = "choice_empty_sequence_raises"
# subject = "secrets.choice"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.choice: choice_empty_sequence_raises (errors)."""
import secrets

_raised = False
try:
    secrets.choice([])
except IndexError:
    _raised = True
assert _raised, "choice_empty_sequence_raises: expected IndexError"
print("choice_empty_sequence_raises OK")
"###);
    assert_output(&out, r###"choice_empty_sequence_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/secrets/compare_digest_bytes_str_raises.py`.
#[test]
fn test_gen_errors_std_libs_secrets_compare_digest_bytes_str_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "errors"
# case = "compare_digest_bytes_str_raises"
# subject = "secrets.compare_digest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.compare_digest: compare_digest_bytes_str_raises (errors)."""
import secrets

_raised = False
try:
    secrets.compare_digest(b"abc", "abc")
except TypeError:
    _raised = True
assert _raised, "compare_digest_bytes_str_raises: expected TypeError"
print("compare_digest_bytes_str_raises OK")
"###);
    assert_output(&out, r###"compare_digest_bytes_str_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/secrets/compare_digest_str_bytes_raises.py`.
#[test]
fn test_gen_errors_std_libs_secrets_compare_digest_str_bytes_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "errors"
# case = "compare_digest_str_bytes_raises"
# subject = "secrets.compare_digest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.compare_digest: compare_digest_str_bytes_raises (errors)."""
import secrets

_raised = False
try:
    secrets.compare_digest("abc", b"abc")
except TypeError:
    _raised = True
assert _raised, "compare_digest_str_bytes_raises: expected TypeError"
print("compare_digest_str_bytes_raises OK")
"###);
    assert_output(&out, r###"compare_digest_str_bytes_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/secrets/randbelow_negative_raises.py`.
#[test]
fn test_gen_errors_std_libs_secrets_randbelow_negative_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "errors"
# case = "randbelow_negative_raises"
# subject = "secrets.randbelow"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.randbelow: randbelow_negative_raises (errors)."""
import secrets

_raised = False
try:
    secrets.randbelow(-5)
except ValueError:
    _raised = True
assert _raised, "randbelow_negative_raises: expected ValueError"
print("randbelow_negative_raises OK")
"###);
    assert_output(&out, r###"randbelow_negative_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/secrets/randbelow_zero_raises.py`.
#[test]
fn test_gen_errors_std_libs_secrets_randbelow_zero_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "errors"
# case = "randbelow_zero_raises"
# subject = "secrets.randbelow"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.randbelow: randbelow_zero_raises (errors)."""
import secrets

_raised = False
try:
    secrets.randbelow(0)
except ValueError:
    _raised = True
assert _raised, "randbelow_zero_raises: expected ValueError"
print("randbelow_zero_raises OK")
"###);
    assert_output(&out, r###"randbelow_zero_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/secrets/token_bytes_negative_raises.py`.
#[test]
fn test_gen_errors_std_libs_secrets_token_bytes_negative_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "errors"
# case = "token_bytes_negative_raises"
# subject = "secrets.token_bytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets.token_bytes: token_bytes_negative_raises (errors)."""
import secrets

_raised = False
try:
    secrets.token_bytes(-1)
except ValueError:
    _raised = True
assert _raised, "token_bytes_negative_raises: expected ValueError"
print("token_bytes_negative_raises OK")
"###);
    assert_output(&out, r###"token_bytes_negative_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/secrets/token_urlsafe_negative_raises.py`.
#[test]
fn test_gen_errors_std_libs_secrets_token_urlsafe_negative_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "errors"
# case = "token_urlsafe_negative_raises"
# subject = "secrets.token_urlsafe"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets.token_urlsafe: token_urlsafe_negative_raises (errors)."""
import secrets

_raised = False
try:
    secrets.token_urlsafe(-1)
except ValueError:
    _raised = True
assert _raised, "token_urlsafe_negative_raises: expected ValueError"
print("token_urlsafe_negative_raises OK")
"###);
    assert_output(&out, r###"token_urlsafe_negative_raises OK
"###);
}
