use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/secrets/choice_returns_member_without_mutation.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_choice_returns_member_without_mutation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "choice_returns_member_without_mutation"
# subject = "secrets.choice"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.choice: choice(seq) returns an element of seq across repeated draws and never mutates the input sequence"""
import secrets

_items = list(range(10))
_orig = list(_items)
for _draw in range(20):
    _c = secrets.choice(_items)
    assert _c in _items, f"choice not in sequence: {_c}"
assert _items == _orig, "choice must not modify the input sequence"

print("choice_returns_member_without_mutation OK")
"###);
    assert_output(&out, r###"choice_returns_member_without_mutation OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/compare_digest_distinguishes_length.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_compare_digest_distinguishes_length() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "compare_digest_distinguishes_length"
# subject = "secrets.compare_digest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.compare_digest: compare_digest reports unequal for same-prefix different-length operands and for equal-length last-char-differing operands, str and bytes alike"""
import secrets

# Long equal operands compare equal, str and bytes alike.
for _s in ("a", "bcd", "xyz123"):
    _a = _s * 100
    assert secrets.compare_digest(_a, _a), f"equal str x100: {_s!r}"
    _ab = _a.encode("utf-8")
    assert secrets.compare_digest(_ab, _ab), f"equal bytes x100: {_s!r}"

# Equal-length operands differing only in the last char compare unequal.
for _s in ("x", "mn", "a1b2c3"):
    _base = _s * 100
    assert not secrets.compare_digest(_base + "q", _base + "k"), f"last-char diff str: {_s!r}"
    assert not secrets.compare_digest(
        (_base + "q").encode("utf-8"), (_base + "k").encode("utf-8")
    ), f"last-char diff bytes: {_s!r}"

# Same-prefix different-length operands compare unequal (str and bytes).
assert not secrets.compare_digest("abc", "abcd"), "shorter vs longer str"
assert not secrets.compare_digest(b"abc", b"abcd"), "shorter vs longer bytes"

print("compare_digest_distinguishes_length OK")
"###);
    assert_output(&out, r###"compare_digest_distinguishes_length OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/compare_digest_returns_real_bool.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_compare_digest_returns_real_bool() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "compare_digest_returns_real_bool"
# subject = "secrets.compare_digest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.compare_digest: compare_digest returns a genuine bool (True for equal, False for unequal) for both str and bytes operands"""
import secrets

# Return value is a real bool (not just truthy/falsy), both directions.
_eq = secrets.compare_digest("abc", "abc")
_neq = secrets.compare_digest("abc", "xyz")
assert type(_eq) is bool, f"equal result type = {type(_eq)!r}"
assert type(_neq) is bool, f"unequal result type = {type(_neq)!r}"
assert _eq == True, f"equal result = {_eq!r}"
assert _neq == False, f"unequal result = {_neq!r}"

# Same for bytes operands.
_eq_b = secrets.compare_digest(b"abc", b"abc")
_neq_b = secrets.compare_digest(b"abc", b"xyz")
assert type(_eq_b) is bool, f"equal bytes result type = {type(_eq_b)!r}"
assert type(_neq_b) is bool, f"unequal bytes result type = {type(_neq_b)!r}"
assert _eq_b == True, f"equal bytes result = {_eq_b!r}"
assert _neq_b == False, f"unequal bytes result = {_neq_b!r}"

print("compare_digest_returns_real_bool OK")
"###);
    assert_output(&out, r###"compare_digest_returns_real_bool OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/compare_digest_tests__test_bad_types.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_compare_digest_tests__test_bad_types() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "compare_digest_tests__test_bad_types"
# subject = "cpython.test_secrets.Compare_Digest_Tests.test_bad_types"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_secrets.py::Compare_Digest_Tests::test_bad_types
"""Auto-ported test: Compare_Digest_Tests::test_bad_types (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
a = 'abcde'
b = a.encode('utf-8')
assert isinstance(a, str)
assert isinstance(b, bytes)

try:
    secrets.compare_digest(a, b)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    secrets.compare_digest(b, a)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("Compare_Digest_Tests::test_bad_types: ok")
"###);
    assert_output(&out, r###"Compare_Digest_Tests::test_bad_types: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/compare_digest_tests__test_bool.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_compare_digest_tests__test_bool() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "compare_digest_tests__test_bool"
# subject = "cpython.test_secrets.Compare_Digest_Tests.test_bool"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_secrets.py::Compare_Digest_Tests::test_bool
"""Auto-ported test: Compare_Digest_Tests::test_bool (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---

assert isinstance(secrets.compare_digest('abc', 'abc'), bool)

assert isinstance(secrets.compare_digest('abc', 'xyz'), bool)
print("Compare_Digest_Tests::test_bool: ok")
"###);
    assert_output(&out, r###"Compare_Digest_Tests::test_bool: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/compare_digest_tests__test_equal.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_compare_digest_tests__test_equal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "compare_digest_tests__test_equal"
# subject = "cpython.test_secrets.Compare_Digest_Tests.test_equal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_secrets.py::Compare_Digest_Tests::test_equal
"""Auto-ported test: Compare_Digest_Tests::test_equal (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
for s in ('a', 'bcd', 'xyz123'):
    a = s * 100
    b = s * 100

    assert secrets.compare_digest(a, b)

    assert secrets.compare_digest(a.encode('utf-8'), b.encode('utf-8'))
print("Compare_Digest_Tests::test_equal: ok")
"###);
    assert_output(&out, r###"Compare_Digest_Tests::test_equal: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/compare_digest_tests__test_unequal.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_compare_digest_tests__test_unequal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "compare_digest_tests__test_unequal"
# subject = "cpython.test_secrets.Compare_Digest_Tests.test_unequal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_secrets.py::Compare_Digest_Tests::test_unequal
"""Auto-ported test: Compare_Digest_Tests::test_unequal (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---

assert not secrets.compare_digest('abc', 'abcd')

assert not secrets.compare_digest(b'abc', b'abcd')
for s in ('x', 'mn', 'a1b2c3'):
    a = s * 100 + 'q'
    b = s * 100 + 'k'

    assert not secrets.compare_digest(a, b)

    assert not secrets.compare_digest(a.encode('utf-8'), b.encode('utf-8'))
print("Compare_Digest_Tests::test_unequal: ok")
"###);
    assert_output(&out, r###"Compare_Digest_Tests::test_unequal: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/randbelow_in_range.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_randbelow_in_range() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "randbelow_in_range"
# subject = "secrets.randbelow"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.randbelow: randbelow(n) returns an int in range(n) for n>=2 across repeated draws; randbelow(1) is always 0"""
import secrets

for _hi in range(2, 10):
    for _draw in range(6):
        _rb = secrets.randbelow(_hi)
        assert isinstance(_rb, int), f"randbelow type = {type(_rb)!r}"
        assert _rb in range(_hi), f"randbelow({_hi}) out of range: {_rb}"

# randbelow(1) has exactly one valid value below 1.
for _draw in range(5):
    assert secrets.randbelow(1) == 0, "randbelow(1) must be 0"

print("randbelow_in_range OK")
"###);
    assert_output(&out, r###"randbelow_in_range OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/randbits_in_range.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_randbits_in_range() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "randbits_in_range"
# subject = "secrets.randbits"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.randbits: randbits(k) returns an int in [0, 2**k) for k in 1,8,16,32,64; randbits(0) is 0"""
import secrets

for _bits in [1, 8, 16, 32, 64]:
    _max = 1 << _bits
    for _draw in range(5):
        _v = secrets.randbits(_bits)
        assert isinstance(_v, int), f"randbits({_bits}) type = {type(_v)!r}"
        assert 0 <= _v < _max, f"randbits({_bits}) out of range: {_v}"

# randbits(0) yields the only value below 2**0 == 1.
assert secrets.randbits(0) == 0, "randbits(0) must be 0"

print("randbits_in_range OK")
"###);
    assert_output(&out, r###"randbits_in_range OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/random_tests__test_choice.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_random_tests__test_choice() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "random_tests__test_choice"
# subject = "cpython.test_secrets.Random_Tests.test_choice"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_secrets.py::Random_Tests::test_choice
"""Auto-ported test: Random_Tests::test_choice (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
items = [1, 2, 4, 8, 16, 32, 64]
for i in range(10):

    assert secrets.choice(items) in items
print("Random_Tests::test_choice: ok")
"###);
    assert_output(&out, r###"Random_Tests::test_choice: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/random_tests__test_randbelow.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_random_tests__test_randbelow() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "random_tests__test_randbelow"
# subject = "cpython.test_secrets.Random_Tests.test_randbelow"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_secrets.py::Random_Tests::test_randbelow
"""Auto-ported test: Random_Tests::test_randbelow (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
for i in range(2, 10):

    assert secrets.randbelow(i) in range(i)

try:
    secrets.randbelow(0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    secrets.randbelow(-1)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("Random_Tests::test_randbelow: ok")
"###);
    assert_output(&out, r###"Random_Tests::test_randbelow: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/random_tests__test_randbits.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_random_tests__test_randbits() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "random_tests__test_randbits"
# subject = "cpython.test_secrets.Random_Tests.test_randbits"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_secrets.py::Random_Tests::test_randbits
"""Auto-ported test: Random_Tests::test_randbits (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
errmsg = 'randbits(%d) returned %d'
for numbits in (3, 12, 30):
    for i in range(6):
        n = secrets.randbits(numbits)

        assert 0 <= n < 2 ** numbits
print("Random_Tests::test_randbits: ok")
"###);
    assert_output(&out, r###"Random_Tests::test_randbits: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/token_bytes_length_matches_request.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_token_bytes_length_matches_request() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_bytes_length_matches_request"
# subject = "secrets.token_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.token_bytes: token_bytes(n) returns exactly n bytes for n in 0,1,8,16,32,64 (default is DEFAULT_ENTROPY=32)"""
import secrets

for _n in [0, 1, 8, 16, 32, 64]:
    _tb = secrets.token_bytes(_n)
    assert isinstance(_tb, bytes), f"token_bytes({_n}) type = {type(_tb)!r}"
    assert len(_tb) == _n, f"token_bytes({_n}) len = {len(_tb)!r}"

# Default size is DEFAULT_ENTROPY (32 bytes in CPython 3.12).
assert len(secrets.token_bytes()) == secrets.DEFAULT_ENTROPY, "token_bytes() default len"
assert secrets.DEFAULT_ENTROPY == 32, "DEFAULT_ENTROPY value"

print("token_bytes_length_matches_request OK")
"###);
    assert_output(&out, r###"token_bytes_length_matches_request OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/token_bytes_not_all_zero.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_token_bytes_not_all_zero() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_bytes_not_all_zero"
# subject = "secrets.token_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets.token_bytes: a 32-byte token_bytes draw is not all-zero (cryptographically random, not a stub)"""
import secrets

# A 32-byte cryptographic draw being all-zero has probability 2**-256;
# any all-zero result signals a broken/stubbed RNG rather than a flake.
_b = secrets.token_bytes(32)
assert any(_byte != 0 for _byte in _b), f"token_bytes(32) is all zero: {_b!r}"

print("token_bytes_not_all_zero OK")
"###);
    assert_output(&out, r###"token_bytes_not_all_zero OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/token_hex_length_is_double.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_token_hex_length_is_double() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_hex_length_is_double"
# subject = "secrets.token_hex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.token_hex: token_hex(n) returns a lowercase-hex str of length 2*n for n in 4,8,16,32; default is 64 chars (2*DEFAULT_ENTROPY)"""
import secrets

for _n in [4, 8, 16, 32]:
    _th = secrets.token_hex(_n)
    assert isinstance(_th, str), f"token_hex({_n}) type = {type(_th)!r}"
    assert len(_th) == 2 * _n, f"token_hex({_n}) len = {len(_th)!r}"
    assert all(c in "0123456789abcdef" for c in _th), f"token_hex({_n}) charset"

# Default is 2 * DEFAULT_ENTROPY = 64 hex chars.
assert len(secrets.token_hex()) == 64, "token_hex() default len"

print("token_hex_length_is_double OK")
"###);
    assert_output(&out, r###"token_hex_length_is_double OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/token_hex_outputs_are_unique.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_token_hex_outputs_are_unique() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_hex_outputs_are_unique"
# subject = "secrets.token_hex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""secrets.token_hex: 100 consecutive token_hex(16) draws are all distinct (no collision / no constant stub)"""
import secrets

# 16-byte tokens collide with probability ~2**-128; a duplicate signals a
# constant stub or a broken RNG, not a flake.
_seen = set()
for _draw in range(100):
    _seen.add(secrets.token_hex(16))
_collisions = 100 - len(_seen)
assert _collisions == 0, f"token_hex(16) collisions: {_collisions}"

print("token_hex_outputs_are_unique OK")
"###);
    assert_output(&out, r###"token_hex_outputs_are_unique OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/token_tests__test_token_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_token_tests__test_token_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_tests__test_token_bytes"
# subject = "cpython.test_secrets.Token_Tests.test_token_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_secrets.py::Token_Tests::test_token_bytes
"""Auto-ported test: Token_Tests::test_token_bytes (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
for n in (1, 8, 17, 100):

    assert isinstance(secrets.token_bytes(n), bytes)

    assert len(secrets.token_bytes(n)) == n
print("Token_Tests::test_token_bytes: ok")
"###);
    assert_output(&out, r###"Token_Tests::test_token_bytes: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/token_tests__test_token_defaults.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_token_tests__test_token_defaults() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_tests__test_token_defaults"
# subject = "cpython.test_secrets.Token_Tests.test_token_defaults"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_secrets.py::Token_Tests::test_token_defaults
"""Auto-ported test: Token_Tests::test_token_defaults (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
for func in (secrets.token_bytes, secrets.token_hex, secrets.token_urlsafe):
    name = func.__name__
    try:
        func()
    except TypeError:

        raise AssertionError('%s cannot be called with no argument' % name)
    try:
        func(None)
    except TypeError:

        raise AssertionError('%s cannot be called with None' % name)
size = secrets.DEFAULT_ENTROPY

assert len(secrets.token_bytes(None)) == size

assert len(secrets.token_hex(None)) == 2 * size
print("Token_Tests::test_token_defaults: ok")
"###);
    assert_output(&out, r###"Token_Tests::test_token_defaults: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/token_tests__test_token_hex.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_token_tests__test_token_hex() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_tests__test_token_hex"
# subject = "cpython.test_secrets.Token_Tests.test_token_hex"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_secrets.py::Token_Tests::test_token_hex
"""Auto-ported test: Token_Tests::test_token_hex (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
for n in (1, 12, 25, 90):
    s = secrets.token_hex(n)

    assert isinstance(s, str)

    assert len(s) == 2 * n

    assert all((c in string.hexdigits for c in s))
print("Token_Tests::test_token_hex: ok")
"###);
    assert_output(&out, r###"Token_Tests::test_token_hex: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/token_tests__test_token_urlsafe.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_token_tests__test_token_urlsafe() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_tests__test_token_urlsafe"
# subject = "cpython.test_secrets.Token_Tests.test_token_urlsafe"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_secrets.py::Token_Tests::test_token_urlsafe
"""Auto-ported test: Token_Tests::test_token_urlsafe (CPython 3.12 oracle)."""


import secrets
import unittest
import string


"Test the secrets module.\n\nAs most of the functions in secrets are thin wrappers around functions\ndefined elsewhere, we don't need to test them exhaustively.\n"


# --- test body ---
legal = string.ascii_letters + string.digits + '-_'
for n in (1, 11, 28, 76):
    s = secrets.token_urlsafe(n)

    assert isinstance(s, str)

    assert all((c in legal for c in s))
print("Token_Tests::test_token_urlsafe: ok")
"###);
    assert_output(&out, r###"Token_Tests::test_token_urlsafe: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/secrets/token_urlsafe_charset_and_length.py`.
#[test]
fn test_gen_behavior_std_libs_secrets_token_urlsafe_charset_and_length() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "behavior"
# case = "token_urlsafe_charset_and_length"
# subject = "secrets.token_urlsafe"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_secrets.py"
# status = "filled"
# ///
"""secrets.token_urlsafe: token_urlsafe(n) returns an unpadded URL-safe base64 str (chars in A-Za-z0-9-_) of length >= n; token_urlsafe(3) is exactly 4 chars"""
import secrets

# URL-safe base64 alphabet: letters, digits, '-' and '_'; never padding ('=')
# nor the non-URL-safe '+' / '/'.
_legal = set("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_")
for _n in [8, 16, 32]:
    _tu = secrets.token_urlsafe(_n)
    assert isinstance(_tu, str), f"token_urlsafe({_n}) type = {type(_tu)!r}"
    assert len(_tu) >= _n, f"token_urlsafe({_n}) len = {len(_tu)!r}"
    assert all(c in _legal for c in _tu), f"token_urlsafe({_n}) charset = {_tu!r}"
    assert "=" not in _tu, f"token_urlsafe({_n}) must not pad"

# n=3 -> ceil(4*3/3) = 4 chars, no padding stripped.
assert len(secrets.token_urlsafe(3)) == 4, "token_urlsafe(3) len"

print("token_urlsafe_charset_and_length OK")
"###);
    assert_output(&out, r###"token_urlsafe_charset_and_length OK
"###);
}
