use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/hmac/class_constructor_digestmod_spellings.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_class_constructor_digestmod_spellings() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "class_constructor_digestmod_spellings"
# subject = "hmac.HMAC"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.HMAC: hmac.HMAC accepts a string name, a hashlib-module constructor, or a keyword-only digestmod, all equivalent to hmac.new; the object reports digest_size=32, block_size=64, name='hmac-sha256'"""
import hmac
import hashlib

key = b"my secret key"
msg = b"compute the hash of this text!"

# Positional hashlib-module digestmod.
h_mod = hmac.HMAC(key, msg, hashlib.sha256)
assert isinstance(h_mod, hmac.HMAC), f"type = {type(h_mod)!r}"
assert len(h_mod.digest()) == 32, "module digestmod -> sha256 digest"

# A string digest name resolves to the same algorithm.
h_str = hmac.HMAC(key, msg, digestmod="sha256")
assert h_str.digest() == h_mod.digest(), "string name == module digestmod"

# hmac.new is equivalent to the class constructor.
h_new = hmac.new(key, msg, digestmod=hashlib.sha256)
assert h_new.digest() == h_mod.digest(), "hmac.new == HMAC(...)"

# Reported metadata for an HMAC object.
h = hmac.new(key, digestmod="sha256")
assert h.digest_size == 32, f"digest_size = {h.digest_size!r}"
assert h.block_size == 64, f"block_size = {h.block_size!r}"
assert h.name == "hmac-sha256", f"name = {h.name!r}"

# msg may be omitted at construction and supplied via update() later.
deferred = hmac.HMAC(key, digestmod="sha256")
deferred.update(msg)
assert deferred.digest() == h_mod.digest(), "deferred update == construct-time msg"

print("class_constructor_digestmod_spellings OK")
"###);
    assert_output(&out, r###"class_constructor_digestmod_spellings OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hmac/compare_digest_equality.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_compare_digest_equality() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "compare_digest_equality"
# subject = "hmac.compare_digest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.compare_digest: compare_digest is True for equal bytes/str, False for differing or different-length inputs, and never raises for same-type args"""
import hmac

# bytes vs bytes.
assert hmac.compare_digest(b"same", b"same"), "bytes same"
assert not hmac.compare_digest(b"abc", b"xyz"), "bytes different"

# str vs str.
assert hmac.compare_digest("same", "same"), "str same"
assert not hmac.compare_digest("abc", "xyz"), "str different"
assert hmac.compare_digest("", ""), "empty str equal"

# Different lengths return False (not raise).
assert not hmac.compare_digest(b"short", b"much_longer_string"), "diff lengths"
assert not hmac.compare_digest("ab", "abc"), "diff str lengths"

print("compare_digest_equality OK")
"###);
    assert_output(&out, r###"compare_digest_equality OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hmac/constructor_test_case__test_normal.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_constructor_test_case__test_normal() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "constructor_test_case__test_normal"
# subject = "cpython.test_hmac.ConstructorTestCase.test_normal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_hmac.py::ConstructorTestCase::test_normal
"""Auto-ported test: ConstructorTestCase::test_normal (CPython 3.12 oracle)."""


import binascii
import functools
import hmac
import hashlib
import unittest
import unittest.mock
import warnings
from test.support import hashlib_helper, check_disallow_instantiation
from _operator import _compare_digest as operator_compare_digest


try:
    import _hashlib as _hashopenssl
    from _hashlib import HMAC as C_HMAC
    from _hashlib import hmac_new as c_hmac_new
    from _hashlib import compare_digest as openssl_compare_digest
except ImportError:
    _hashopenssl = None
    C_HMAC = None
    c_hmac_new = None
    openssl_compare_digest = None

try:
    import _sha256 as sha256_module
except ImportError:
    sha256_module = None

def ignore_warning(func):

    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        with warnings.catch_warnings():
            warnings.filterwarnings('ignore', category=DeprecationWarning)
            return func(*args, **kwargs)
    return wrapper


# --- test body ---
expected = '6c845b47f52b3b47f6590c502db7825aad757bf4fadc8fa972f7cd2e76a5bdeb'
try:
    hmac.HMAC(b'key', digestmod='sha256')
except Exception:

    raise AssertionError('Standard constructor call raised exception.')
print("ConstructorTestCase::test_normal: ok")
"###);
    assert_output(&out, r###"ConstructorTestCase::test_normal: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hmac/constructor_test_case__test_with_bytearray.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_constructor_test_case__test_with_bytearray() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "constructor_test_case__test_with_bytearray"
# subject = "cpython.test_hmac.ConstructorTestCase.test_with_bytearray"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_hmac.py::ConstructorTestCase::test_with_bytearray
"""Auto-ported test: ConstructorTestCase::test_with_bytearray (CPython 3.12 oracle)."""


import binascii
import functools
import hmac
import hashlib
import unittest
import unittest.mock
import warnings
from test.support import hashlib_helper, check_disallow_instantiation
from _operator import _compare_digest as operator_compare_digest


try:
    import _hashlib as _hashopenssl
    from _hashlib import HMAC as C_HMAC
    from _hashlib import hmac_new as c_hmac_new
    from _hashlib import compare_digest as openssl_compare_digest
except ImportError:
    _hashopenssl = None
    C_HMAC = None
    c_hmac_new = None
    openssl_compare_digest = None

try:
    import _sha256 as sha256_module
except ImportError:
    sha256_module = None

def ignore_warning(func):

    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        with warnings.catch_warnings():
            warnings.filterwarnings('ignore', category=DeprecationWarning)
            return func(*args, **kwargs)
    return wrapper


# --- test body ---
expected = '6c845b47f52b3b47f6590c502db7825aad757bf4fadc8fa972f7cd2e76a5bdeb'
try:
    h = hmac.HMAC(bytearray(b'key'), bytearray(b'hash this!'), digestmod='sha256')
except Exception:

    raise AssertionError('Constructor call with bytearray arguments raised exception.')

assert h.hexdigest() == expected
print("ConstructorTestCase::test_with_bytearray: ok")
"###);
    assert_output(&out, r###"ConstructorTestCase::test_with_bytearray: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hmac/constructor_test_case__test_with_memoryview_msg.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_constructor_test_case__test_with_memoryview_msg() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "constructor_test_case__test_with_memoryview_msg"
# subject = "cpython.test_hmac.ConstructorTestCase.test_with_memoryview_msg"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_hmac.py::ConstructorTestCase::test_with_memoryview_msg
"""Auto-ported test: ConstructorTestCase::test_with_memoryview_msg (CPython 3.12 oracle)."""


import binascii
import functools
import hmac
import hashlib
import unittest
import unittest.mock
import warnings
from test.support import hashlib_helper, check_disallow_instantiation
from _operator import _compare_digest as operator_compare_digest


try:
    import _hashlib as _hashopenssl
    from _hashlib import HMAC as C_HMAC
    from _hashlib import hmac_new as c_hmac_new
    from _hashlib import compare_digest as openssl_compare_digest
except ImportError:
    _hashopenssl = None
    C_HMAC = None
    c_hmac_new = None
    openssl_compare_digest = None

try:
    import _sha256 as sha256_module
except ImportError:
    sha256_module = None

def ignore_warning(func):

    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        with warnings.catch_warnings():
            warnings.filterwarnings('ignore', category=DeprecationWarning)
            return func(*args, **kwargs)
    return wrapper


# --- test body ---
expected = '6c845b47f52b3b47f6590c502db7825aad757bf4fadc8fa972f7cd2e76a5bdeb'
try:
    h = hmac.HMAC(b'key', memoryview(b'hash this!'), digestmod='sha256')
except Exception:

    raise AssertionError('Constructor call with memoryview msg raised exception.')

assert h.hexdigest() == expected
print("ConstructorTestCase::test_with_memoryview_msg: ok")
"###);
    assert_output(&out, r###"ConstructorTestCase::test_with_memoryview_msg: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hmac/constructor_test_case__test_withmodule.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_constructor_test_case__test_withmodule() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "constructor_test_case__test_withmodule"
# subject = "cpython.test_hmac.ConstructorTestCase.test_withmodule"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_hmac.py::ConstructorTestCase::test_withmodule
"""Auto-ported test: ConstructorTestCase::test_withmodule (CPython 3.12 oracle)."""


import binascii
import functools
import hmac
import hashlib
import unittest
import unittest.mock
import warnings
from test.support import hashlib_helper, check_disallow_instantiation
from _operator import _compare_digest as operator_compare_digest


try:
    import _hashlib as _hashopenssl
    from _hashlib import HMAC as C_HMAC
    from _hashlib import hmac_new as c_hmac_new
    from _hashlib import compare_digest as openssl_compare_digest
except ImportError:
    _hashopenssl = None
    C_HMAC = None
    c_hmac_new = None
    openssl_compare_digest = None

try:
    import _sha256 as sha256_module
except ImportError:
    sha256_module = None

def ignore_warning(func):

    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        with warnings.catch_warnings():
            warnings.filterwarnings('ignore', category=DeprecationWarning)
            return func(*args, **kwargs)
    return wrapper


# --- test body ---
expected = '6c845b47f52b3b47f6590c502db7825aad757bf4fadc8fa972f7cd2e76a5bdeb'
try:
    h = hmac.HMAC(b'key', b'', hashlib.sha256)
except Exception:

    raise AssertionError('Constructor call with hashlib.sha256 raised exception.')
print("ConstructorTestCase::test_withmodule: ok")
"###);
    assert_output(&out, r###"ConstructorTestCase::test_withmodule: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hmac/constructor_test_case__test_withtext.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_constructor_test_case__test_withtext() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "constructor_test_case__test_withtext"
# subject = "cpython.test_hmac.ConstructorTestCase.test_withtext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_hmac.py::ConstructorTestCase::test_withtext
"""Auto-ported test: ConstructorTestCase::test_withtext (CPython 3.12 oracle)."""


import binascii
import functools
import hmac
import hashlib
import unittest
import unittest.mock
import warnings
from test.support import hashlib_helper, check_disallow_instantiation
from _operator import _compare_digest as operator_compare_digest


try:
    import _hashlib as _hashopenssl
    from _hashlib import HMAC as C_HMAC
    from _hashlib import hmac_new as c_hmac_new
    from _hashlib import compare_digest as openssl_compare_digest
except ImportError:
    _hashopenssl = None
    C_HMAC = None
    c_hmac_new = None
    openssl_compare_digest = None

try:
    import _sha256 as sha256_module
except ImportError:
    sha256_module = None

def ignore_warning(func):

    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        with warnings.catch_warnings():
            warnings.filterwarnings('ignore', category=DeprecationWarning)
            return func(*args, **kwargs)
    return wrapper


# --- test body ---
expected = '6c845b47f52b3b47f6590c502db7825aad757bf4fadc8fa972f7cd2e76a5bdeb'
try:
    h = hmac.HMAC(b'key', b'hash this!', digestmod='sha256')
except Exception:

    raise AssertionError('Constructor call with text argument raised exception.')

assert h.hexdigest() == expected
print("ConstructorTestCase::test_withtext: ok")
"###);
    assert_output(&out, r###"ConstructorTestCase::test_withtext: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hmac/copy_is_independent_snapshot.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_copy_is_independent_snapshot() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "copy_is_independent_snapshot"
# subject = "hmac.HMAC.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.HMAC.copy: copy() returns a distinct object that freezes the original's state at copy time; the original keeps accumulating after digest(), and sibling copies diverge independently"""
import hmac

key = b"key"

# A fresh copy is a distinct object with identical accumulated state.
h1 = hmac.new(key, digestmod="sha256")
h1.update(b"some random text")
h2 = h1.copy()
assert h1 is not h2, "copy is a distinct object"
assert h1.digest() == h2.digest(), "copy digest matches original"
assert h1.hexdigest() == h2.hexdigest(), "copy hexdigest matches original"

# Computing digest() does not finalize state: more updates still apply.
baseline = hmac.new(key, b"some random texttail", digestmod="sha256").digest()
h1.update(b"tail")
assert h1.digest() == baseline, "update after digest() keeps accumulating"

# The copy taken earlier is unaffected by mutations to the original.
assert h2.digest() != h1.digest(), "copy stayed independent of original"
assert h2.digest() == hmac.new(key, b"some random text", digestmod="sha256").digest(), \
    "copy froze the original's state at copy time"

# Copies can themselves diverge from each other.
h3 = h2.copy()
h2.update(b"AAA")
h3.update(b"BBB")
assert h2.digest() != h3.digest(), "sibling copies diverge independently"

print("copy_is_independent_snapshot OK")
"###);
    assert_output(&out, r###"copy_is_independent_snapshot OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hmac/copy_test_case__test_equality.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_copy_test_case__test_equality() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "copy_test_case__test_equality"
# subject = "cpython.test_hmac.CopyTestCase.test_equality"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_hmac.py::CopyTestCase::test_equality
"""Auto-ported test: CopyTestCase::test_equality (CPython 3.12 oracle)."""


import binascii
import functools
import hmac
import hashlib
import unittest
import unittest.mock
import warnings
from test.support import hashlib_helper, check_disallow_instantiation
from _operator import _compare_digest as operator_compare_digest


try:
    import _hashlib as _hashopenssl
    from _hashlib import HMAC as C_HMAC
    from _hashlib import hmac_new as c_hmac_new
    from _hashlib import compare_digest as openssl_compare_digest
except ImportError:
    _hashopenssl = None
    C_HMAC = None
    c_hmac_new = None
    openssl_compare_digest = None

try:
    import _sha256 as sha256_module
except ImportError:
    sha256_module = None

def ignore_warning(func):

    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        with warnings.catch_warnings():
            warnings.filterwarnings('ignore', category=DeprecationWarning)
            return func(*args, **kwargs)
    return wrapper


# --- test body ---
h1 = hmac.HMAC(b'key', digestmod='sha256')
h1.update(b'some random text')
h2 = h1.copy()

assert h1.digest() == h2.digest()

assert h1.hexdigest() == h2.hexdigest()
print("CopyTestCase::test_equality: ok")
"###);
    assert_output(&out, r###"CopyTestCase::test_equality: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hmac/digest_sizes_per_algorithm.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_digest_sizes_per_algorithm() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "digest_sizes_per_algorithm"
# subject = "hmac.new"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.new: HMAC digest length tracks the underlying hash: md5 -> 16 bytes, sha1 -> 20 bytes, sha256 -> 32 bytes, and differing algorithms over the same key/msg yield differing MACs"""
import hmac
import hashlib

key = b"key"
msg = b"message"

# Digest length tracks the underlying hash function.
md5_mac = hmac.new(b"key", b"data", digestmod=hashlib.md5).digest()
assert len(md5_mac) == 16, f"HMAC-MD5 len = {len(md5_mac)!r}"

sha1_mac = hmac.new(key, msg, digestmod=hashlib.sha1).digest()
assert len(sha1_mac) == 20, f"HMAC-SHA1 len = {len(sha1_mac)!r}"

sha256_mac = hmac.new(key, msg, digestmod=hashlib.sha256).digest()
assert len(sha256_mac) == 32, f"HMAC-SHA256 len = {len(sha256_mac)!r}"

# Different digestmod over the same key/msg yields different MACs.
assert sha256_mac != sha1_mac, "different digestmod = different MAC"

print("digest_sizes_per_algorithm OK")
"###);
    assert_output(&out, r###"digest_sizes_per_algorithm OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hmac/incremental_equals_single.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_incremental_equals_single() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "incremental_equals_single"
# subject = "hmac.HMAC.update"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.HMAC.update: feeding the message in chunks via update() yields the same digest as supplying it whole at construction"""
import hmac
import hashlib

key = b"secret_key"

# Whole message at construction.
single = hmac.new(key, b"hello world", digestmod=hashlib.sha256).digest()

# Same message fed in chunks via update().
inc = hmac.new(key, digestmod=hashlib.sha256)
inc.update(b"hello ")
inc.update(b"world")
assert inc.digest() == single, "incremental update == single-shot digest"

print("incremental_equals_single OK")
"###);
    assert_output(&out, r###"incremental_equals_single OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hmac/key_and_message_sensitivity.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_key_and_message_sensitivity() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "key_and_message_sensitivity"
# subject = "hmac.new"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.new: changing the key, or the message, changes the MAC; an empty message and an empty key are both valid and produce a full-length digest"""
import hmac
import hashlib

key = b"secret_key"
msg = b"hello world"
base = hmac.new(key, msg, digestmod=hashlib.sha256).digest()

# A different key changes the MAC.
assert hmac.new(b"other_key", msg, digestmod=hashlib.sha256).digest() != base, \
    "different key = different MAC"

# A different message changes the MAC.
assert hmac.new(key, b"other message", digestmod=hashlib.sha256).digest() != base, \
    "different msg = different MAC"

# An empty message is valid and produces a full-length digest.
empty_msg = hmac.new(b"key", b"", digestmod=hashlib.sha256).hexdigest()
assert len(empty_msg) == 64, f"empty msg HMAC len = {len(empty_msg)!r}"
assert empty_msg != hmac.new(b"key", b"x", digestmod=hashlib.sha256).hexdigest(), \
    "empty vs non-empty differ"

# An empty key is valid and produces a full-length digest.
empty_key = hmac.new(b"", b"message", digestmod=hashlib.sha256).digest()
assert isinstance(empty_key, bytes), f"empty key HMAC = {type(empty_key)!r}"
assert len(empty_key) == 32, "empty key mac len"

print("key_and_message_sensitivity OK")
"###);
    assert_output(&out, r###"key_and_message_sensitivity OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hmac/one_shot_equals_object_path.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_one_shot_equals_object_path() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "one_shot_equals_object_path"
# subject = "hmac.digest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.digest: the hmac.digest(...) one-shot fast path returns the same bytes as hmac.new(...).digest(), for both a string and a hashlib-constructor digestmod"""
import hmac
import hashlib

key = b"test_key_123"
msg = b"test message"

# String digest name.
obj_path = hmac.new(key, msg, digestmod="sha256").digest()
one_shot = hmac.digest(key, msg, digest="sha256")
assert one_shot == obj_path, "hmac.digest(str) == hmac.new().digest()"

# hashlib-constructor digestmod.
obj_path2 = hmac.new(key, msg, digestmod=hashlib.sha256).digest()
one_shot2 = hmac.digest(key, msg, digest=hashlib.sha256)
assert one_shot2 == obj_path2, "hmac.digest(ctor) == hmac.new().digest()"

print("one_shot_equals_object_path OK")
"###);
    assert_output(&out, r###"one_shot_equals_object_path OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hmac/sanity_test_case__test_exercise_all_methods.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_sanity_test_case__test_exercise_all_methods() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "sanity_test_case__test_exercise_all_methods"
# subject = "cpython.test_hmac.SanityTestCase.test_exercise_all_methods"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_hmac.py::SanityTestCase::test_exercise_all_methods
"""Auto-ported test: SanityTestCase::test_exercise_all_methods (CPython 3.12 oracle)."""


import binascii
import functools
import hmac
import hashlib
import unittest
import unittest.mock
import warnings
from test.support import hashlib_helper, check_disallow_instantiation
from _operator import _compare_digest as operator_compare_digest


try:
    import _hashlib as _hashopenssl
    from _hashlib import HMAC as C_HMAC
    from _hashlib import hmac_new as c_hmac_new
    from _hashlib import compare_digest as openssl_compare_digest
except ImportError:
    _hashopenssl = None
    C_HMAC = None
    c_hmac_new = None
    openssl_compare_digest = None

try:
    import _sha256 as sha256_module
except ImportError:
    sha256_module = None

def ignore_warning(func):

    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        with warnings.catch_warnings():
            warnings.filterwarnings('ignore', category=DeprecationWarning)
            return func(*args, **kwargs)
    return wrapper


# --- test body ---
try:
    h = hmac.HMAC(b'my secret key', digestmod='sha256')
    h.update(b'compute the hash of this text!')
    h.digest()
    h.hexdigest()
    h.copy()
except Exception:

    raise AssertionError('Exception raised during normal usage of HMAC class.')
print("SanityTestCase::test_exercise_all_methods: ok")
"###);
    assert_output(&out, r###"SanityTestCase::test_exercise_all_methods: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hmac/sha256_known_vector.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_sha256_known_vector() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "sha256_known_vector"
# subject = "hmac.new"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.new: HMAC-SHA256 of key='key' over the pangram matches the published RFC/NIST hexdigest f7bc83f4...2d1a3cd8"""
import hmac
import hashlib

# Published HMAC-SHA256 test vector: key="key", msg=the pangram.
key = b"key"
msg = b"The quick brown fox jumps over the lazy dog"
expected = "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
got = hmac.new(key, msg, digestmod=hashlib.sha256).hexdigest()
assert got == expected, f"HMAC-SHA256 vector = {got!r}"

print("sha256_known_vector OK")
"###);
    assert_output(&out, r###"sha256_known_vector OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hmac/small_block_size_warns.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_small_block_size_warns() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "small_block_size_warns"
# subject = "hmac.HMAC"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
"""hmac.HMAC: a caller-supplied digestmod that lacks block_size, or reports a suspiciously small one, triggers a RuntimeWarning mentioning block_size"""
import hmac
import hashlib
import warnings


class CrazyHash:
    """A digest object that (initially) has no block_size attribute."""

    def __init__(self, *args):
        self._x = hashlib.sha256(*args)
        self.digest_size = self._x.digest_size

    def update(self, v):
        self._x.update(v)

    def digest(self):
        return self._x.digest()


with warnings.catch_warnings():
    warnings.simplefilter("error", RuntimeWarning)

    # Missing block_size attribute -> RuntimeWarning.
    missing_warned = False
    try:
        hmac.HMAC(b"a", b"b", digestmod=CrazyHash)
    except RuntimeWarning as w:
        assert "block_size" in str(w), f"warn text = {str(w)!r}"
        missing_warned = True
    assert missing_warned, "missing block_size should warn"

    # A block_size that is too small -> RuntimeWarning.
    CrazyHash.block_size = 1
    small_warned = False
    try:
        hmac.HMAC(b"a", b"b", digestmod=CrazyHash)
    except RuntimeWarning as w:
        assert "block_size" in str(w), f"warn text = {str(w)!r}"
        small_warned = True
    assert small_warned, "small block_size should warn"

print("small_block_size_warns OK")
"###);
    assert_output(&out, r###"small_block_size_warns OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hmac/update_test_case__test_with_str_update.py`.
#[test]
fn test_gen_behavior_std_libs_hmac_update_test_case__test_with_str_update() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "update_test_case__test_with_str_update"
# subject = "cpython.test_hmac.UpdateTestCase.test_with_str_update"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_hmac.py::UpdateTestCase::test_with_str_update
"""Auto-ported test: UpdateTestCase::test_with_str_update (CPython 3.12 oracle)."""


import binascii
import functools
import hmac
import hashlib
import unittest
import unittest.mock
import warnings
from test.support import hashlib_helper, check_disallow_instantiation
from _operator import _compare_digest as operator_compare_digest


try:
    import _hashlib as _hashopenssl
    from _hashlib import HMAC as C_HMAC
    from _hashlib import hmac_new as c_hmac_new
    from _hashlib import compare_digest as openssl_compare_digest
except ImportError:
    _hashopenssl = None
    C_HMAC = None
    c_hmac_new = None
    openssl_compare_digest = None

try:
    import _sha256 as sha256_module
except ImportError:
    sha256_module = None

def ignore_warning(func):

    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        with warnings.catch_warnings():
            warnings.filterwarnings('ignore', category=DeprecationWarning)
            return func(*args, **kwargs)
    return wrapper


# --- test body ---
try:
    h = hmac.new(b'key', digestmod='sha256')
    h.update('invalid update')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("UpdateTestCase::test_with_str_update: ok")
"###);
    assert_output(&out, r###"UpdateTestCase::test_with_str_update: ok
"###);
}
