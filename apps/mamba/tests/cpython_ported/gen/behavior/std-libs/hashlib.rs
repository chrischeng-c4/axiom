use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/hashlib/blake2_default_digest_sizes.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_blake2_default_digest_sizes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "blake2_default_digest_sizes"
# subject = "hashlib.blake2b"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.blake2b: blake2b default digest_size is 64, blake2s default digest_size is 32"""
import hashlib

assert hashlib.blake2b().digest_size == 64, "blake2b default digest_size"
assert hashlib.blake2s().digest_size == 32, "blake2s default digest_size"

print("blake2_default_digest_sizes OK")
"###);
    assert_output(&out, r###"blake2_default_digest_sizes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/blake2_digest_size_param.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_blake2_digest_size_param() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "blake2_digest_size_param"
# subject = "hashlib.blake2b"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.blake2b: the digest_size= parameter shrinks output exactly: blake2b(digest_size=16) is 16 bytes, blake2s(digest_size=8) is 8 bytes"""
import hashlib

assert len(hashlib.blake2b(b"x", digest_size=16).digest()) == 16, "blake2b digest_size=16"
assert len(hashlib.blake2s(b"x", digest_size=8).digest()) == 8, "blake2s digest_size=8"

print("blake2_digest_size_param OK")
"###);
    assert_output(&out, r###"blake2_digest_size_param OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/blake2b_empty_known_vector.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_blake2b_empty_known_vector() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "blake2b_empty_known_vector"
# subject = "hashlib.blake2b"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.blake2b: blake2b(b'') hexdigest begins with the known prefix 786a02f742015903c6c6fd852552d272"""
import hashlib

assert hashlib.blake2b(b"").hexdigest()[:32] == \
    "786a02f742015903c6c6fd852552d272", "blake2b('') prefix"

print("blake2b_empty_known_vector OK")
"###);
    assert_output(&out, r###"blake2b_empty_known_vector OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/blake2b_incremental_equals_oneshot.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_blake2b_incremental_equals_oneshot() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "blake2b_incremental_equals_oneshot"
# subject = "hashlib.blake2b"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.blake2b: blake2b incremental update('hello')+update(' world') equals the one-shot digest, and .name reports 'blake2b'/'blake2s'"""
import hashlib

_one = hashlib.blake2b(b"hello world")
_inc = hashlib.blake2b()
_inc.update(b"hello")
_inc.update(b" world")
assert _one.digest() == _inc.digest(), "blake2b incremental == one-shot"
assert hashlib.blake2b().name == "blake2b", "blake2b .name"
assert hashlib.blake2s().name == "blake2s", "blake2s .name"

print("blake2b_incremental_equals_oneshot OK")
"###);
    assert_output(&out, r###"blake2b_incremental_equals_oneshot OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/blake2b_keyed_mac.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_blake2b_keyed_mac() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "blake2b_keyed_mac"
# subject = "hashlib.blake2b"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.blake2b: a keyed blake2b (MAC use) differs from the unkeyed hash of the same data and is deterministic given the same key"""
import hashlib

_unkeyed = hashlib.blake2b(b"message").hexdigest()
_keyed = hashlib.blake2b(b"message", key=b"secret").hexdigest()
assert _unkeyed != _keyed, "keyed blake2b differs from unkeyed"
assert hashlib.blake2b(b"message", key=b"secret").hexdigest() == _keyed, "keyed blake2b deterministic"

print("blake2b_keyed_mac OK")
"###);
    assert_output(&out, r###"blake2b_keyed_mac OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/copy_is_independent.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_copy_is_independent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "copy_is_independent"
# subject = "hashlib.sha256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha256: copy() snapshots state: updating the original after copy() leaves the copy's digest equal to the un-updated base"""
import hashlib

_h = hashlib.sha256(b"base")
_c = _h.copy()
_h.update(b"_extra")
_before = _c.hexdigest()
assert _before == hashlib.sha256(b"base").hexdigest(), "copy unaffected by original update"

print("copy_is_independent OK")
"###);
    assert_output(&out, r###"copy_is_independent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/different_inputs_differ.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_different_inputs_differ() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "different_inputs_differ"
# subject = "hashlib.sha256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha256: different inputs produce different digests: sha256(b'a') != sha256(b'b')"""
import hashlib

assert hashlib.sha256(b"a").digest() != hashlib.sha256(b"b").digest(), "collision-free"

print("different_inputs_differ OK")
"###);
    assert_output(&out, r###"different_inputs_differ OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/digest_is_deterministic.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_digest_is_deterministic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "digest_is_deterministic"
# subject = "hashlib.sha256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha256: the same input always produces the same digest: sha256(b'test') equals itself across two constructions"""
import hashlib

assert hashlib.sha256(b"test").digest() == hashlib.sha256(b"test").digest(), "deterministic"

print("digest_is_deterministic OK")
"###);
    assert_output(&out, r###"digest_is_deterministic OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/file_digest_streams_file.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_file_digest_streams_file() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "file_digest_streams_file"
# subject = "hashlib.file_digest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.file_digest: file_digest streams a temp file's bytes and matches the in-memory digest: by algorithm name, by constructor callable, for md5, and for an empty file"""
import hashlib

import os
import tempfile

_payload = b"hello world\n" * 100
_expected = hashlib.sha256(_payload).hexdigest()

with tempfile.TemporaryDirectory() as _d:
    _path = os.path.join(_d, "data.bin")
    with open(_path, "wb") as _w:
        _w.write(_payload)

    with open(_path, "rb") as _r:
        _by_name = hashlib.file_digest(_r, "sha256").hexdigest()
    assert _by_name == _expected, f"file_digest by name = {_by_name!r}"

    with open(_path, "rb") as _r:
        _by_ctor = hashlib.file_digest(_r, lambda: hashlib.sha256()).hexdigest()
    assert _by_ctor == _expected, "file_digest by callable matches"

    with open(_path, "rb") as _r:
        _md5 = hashlib.file_digest(_r, "md5").hexdigest()
    assert _md5 == hashlib.md5(_payload).hexdigest(), "file_digest md5 matches"

    _empty_path = os.path.join(_d, "empty.bin")
    with open(_empty_path, "wb"):
        pass
    with open(_empty_path, "rb") as _r:
        _empty = hashlib.file_digest(_r, "sha256").hexdigest()
    assert _empty == hashlib.sha256(b"").hexdigest(), "file_digest of empty file"

assert not os.path.exists(_d), "tempdir auto-cleaned"

print("file_digest_streams_file OK")
"###);
    assert_output(&out, r###"file_digest_streams_file OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/hexdigest_repeatable.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_hexdigest_repeatable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "hexdigest_repeatable"
# subject = "hashlib.sha256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha256: hexdigest() does not consume state (repeatable) and digest().hex() equals hexdigest()"""
import hashlib

_rep = hashlib.sha256(b"abc")
assert _rep.hexdigest() == _rep.hexdigest(), "hexdigest repeatable"
assert _rep.digest().hex() == _rep.hexdigest(), "digest().hex() == hexdigest()"

print("hexdigest_repeatable OK")
"###);
    assert_output(&out, r###"hexdigest_repeatable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/md5_hello_known_vector.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_md5_hello_known_vector() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "md5_hello_known_vector"
# subject = "hashlib.md5"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.md5: md5(b'hello') hexdigest is the reference 5d41402abc4b2a76b9719d911017c592"""
import hashlib

_md5_hello = hashlib.md5(b"hello").hexdigest()
assert _md5_hello == "5d41402abc4b2a76b9719d911017c592", f"md5('hello') = {_md5_hello!r}"

print("md5_hello_known_vector OK")
"###);
    assert_output(&out, r###"md5_hello_known_vector OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/new_equals_direct_constructor.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_new_equals_direct_constructor() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "new_equals_direct_constructor"
# subject = "hashlib.new"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.new: new('sha256', data) produces the same digest as the direct sha256(data) constructor"""
import hashlib

_by_name = hashlib.new("sha256", b"hello")
_by_direct = hashlib.sha256(b"hello")
assert _by_name.digest() == _by_direct.digest(), "new() == direct constructor"

print("new_equals_direct_constructor OK")
"###);
    assert_output(&out, r###"new_equals_direct_constructor OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/new_name_case_normalized.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_new_name_case_normalized() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "new_name_case_normalized"
# subject = "hashlib.new"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.new: new() is case-insensitive: new('SHA256').name=='sha256', new('Sha1').name=='sha1', and the uppercase-name digest matches the direct constructor"""
import hashlib

assert hashlib.new("SHA256").name == "sha256", "new('SHA256').name canonicalized"
assert hashlib.new("Sha1").name == "sha1", "new('Sha1').name canonicalized"
assert hashlib.new("SHA256", b"hello").digest() == hashlib.sha256(b"hello").digest(), \
    "uppercase new() == direct"

print("new_name_case_normalized OK")
"###);
    assert_output(&out, r###"new_name_case_normalized OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/pbkdf2_dklen_controls_length.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_pbkdf2_dklen_controls_length() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "pbkdf2_dklen_controls_length"
# subject = "hashlib.pbkdf2_hmac"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.pbkdf2_hmac: default output length equals the hash's digest_size (32 for sha256); dklen= overrides it exactly (16 over sha256, 40 over sha1)"""
import hashlib

assert len(hashlib.pbkdf2_hmac("sha256", b"pw", b"salt", 1)) == 32, "default dklen == 32 for sha256"
assert len(hashlib.pbkdf2_hmac("sha256", b"pw", b"salt", 10, dklen=16)) == 16, "dklen=16"
assert len(hashlib.pbkdf2_hmac("sha1", b"pw", b"salt", 10, dklen=40)) == 40, "dklen=40 over sha1"

print("pbkdf2_dklen_controls_length OK")
"###);
    assert_output(&out, r###"pbkdf2_dklen_controls_length OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/pbkdf2_inputs_change_key.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_pbkdf2_inputs_change_key() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "pbkdf2_inputs_change_key"
# subject = "hashlib.pbkdf2_hmac"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.pbkdf2_hmac: pbkdf2 is deterministic for identical inputs; changing the iteration count or the salt changes the derived key"""
import hashlib

assert hashlib.pbkdf2_hmac("sha256", b"pw", b"salt", 5) == \
    hashlib.pbkdf2_hmac("sha256", b"pw", b"salt", 5), "pbkdf2 deterministic"
_a = hashlib.pbkdf2_hmac("sha256", b"pw", b"salt", 1)
_b = hashlib.pbkdf2_hmac("sha256", b"pw", b"salt", 2)
assert _a != _b, "iteration count changes derived key"
_s1 = hashlib.pbkdf2_hmac("sha256", b"pw", b"salt1", 5)
_s2 = hashlib.pbkdf2_hmac("sha256", b"pw", b"salt2", 5)
assert _s1 != _s2, "salt changes derived key"

print("pbkdf2_inputs_change_key OK")
"###);
    assert_output(&out, r###"pbkdf2_inputs_change_key OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/pbkdf2_known_vector.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_pbkdf2_known_vector() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "pbkdf2_known_vector"
# subject = "hashlib.pbkdf2_hmac"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.pbkdf2_hmac: one iteration of PBKDF2-HMAC-SHA256 over ('password','salt') matches the known prefix 120fb6cffcf8b32c43e7225256c4f837"""
import hashlib

_dk = hashlib.pbkdf2_hmac("sha256", b"password", b"salt", 1)
assert _dk.hex()[:32] == "120fb6cffcf8b32c43e7225256c4f837", f"pbkdf2 1-iter = {_dk.hex()[:32]!r}"

print("pbkdf2_known_vector OK")
"###);
    assert_output(&out, r###"pbkdf2_known_vector OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/sha1_hello_known_vector.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_sha1_hello_known_vector() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "sha1_hello_known_vector"
# subject = "hashlib.sha1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha1: sha1(b'hello') hexdigest is the reference aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"""
import hashlib

_sha1_hello = hashlib.sha1(b"hello").hexdigest()
assert _sha1_hello == "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d", f"sha1('hello') = {_sha1_hello!r}"

print("sha1_hello_known_vector OK")
"###);
    assert_output(&out, r###"sha1_hello_known_vector OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/sha256_empty_known_vector.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_sha256_empty_known_vector() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "sha256_empty_known_vector"
# subject = "hashlib.sha256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha256: sha256 of the empty input is the fixed constant e3b0c442...b7852b855"""
import hashlib

_empty = hashlib.sha256(b"").hexdigest()
assert _empty == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", \
    f"sha256('') = {_empty!r}"

print("sha256_empty_known_vector OK")
"###);
    assert_output(&out, r###"sha256_empty_known_vector OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/sha3_256_block_size_sponge_rate.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_sha3_256_block_size_sponge_rate() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "sha3_256_block_size_sponge_rate"
# subject = "hashlib.sha3_256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.sha3_256: the sponge rate gives sha3_256 a non-power-of-two block_size of 136, unlike SHA-2; .name is the canonical 'sha3_512'"""
import hashlib

assert hashlib.sha3_256().block_size == 136, \
    f"sha3_256 block_size = {hashlib.sha3_256().block_size!r}"
assert hashlib.sha3_512().name == "sha3_512", "sha3_512 .name"

print("sha3_256_block_size_sponge_rate OK")
"###);
    assert_output(&out, r###"sha3_256_block_size_sponge_rate OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/sha3_256_incremental_and_copy.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_sha3_256_incremental_and_copy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "sha3_256_incremental_and_copy"
# subject = "hashlib.sha3_256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.sha3_256: sha3_256 incremental update equals one-shot, and copy() is an independent snapshot unaffected by later updates"""
import hashlib

_one = hashlib.sha3_256(b"hello world")
_inc = hashlib.sha3_256()
_inc.update(b"hello")
_inc.update(b" world")
assert _one.digest() == _inc.digest(), "sha3_256 incremental == one-shot"
_h = hashlib.sha3_256(b"base")
_c = _h.copy()
_h.update(b"_more")
assert _c.hexdigest() == hashlib.sha3_256(b"base").hexdigest(), "sha3 copy independent"

print("sha3_256_incremental_and_copy OK")
"###);
    assert_output(&out, r###"sha3_256_incremental_and_copy OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/sha3_256_known_vectors.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_sha3_256_known_vectors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "sha3_256_known_vectors"
# subject = "hashlib.sha3_256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.sha3_256: sha3_256 NIST FIPS-202 known answers: sha3_256(b'abc') and sha3_256(b'') match their reference hexdigests"""
import hashlib

assert hashlib.sha3_256(b"abc").hexdigest() == \
    "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532", "sha3_256('abc')"
assert hashlib.sha3_256(b"").hexdigest() == \
    "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a", "sha3_256('')"

print("sha3_256_known_vectors OK")
"###);
    assert_output(&out, r###"sha3_256_known_vectors OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/sha3_digest_sizes.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_sha3_digest_sizes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "sha3_digest_sizes"
# subject = "hashlib.sha3_256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.sha3_256: sha3 digest_size matches the trailing bit count / 8: sha3_224=28, sha3_256=32, sha3_384=48, sha3_512=64"""
import hashlib

assert hashlib.sha3_224().digest_size == 28, "sha3_224 digest_size"
assert hashlib.sha3_256().digest_size == 32, "sha3_256 digest_size"
assert hashlib.sha3_384().digest_size == 48, "sha3_384 digest_size"
assert hashlib.sha3_512().digest_size == 64, "sha3_512 digest_size"

print("sha3_digest_sizes OK")
"###);
    assert_output(&out, r###"sha3_digest_sizes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/sha512_digest_sizes.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_sha512_digest_sizes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "sha512_digest_sizes"
# subject = "hashlib.sha512"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha512: sha512 has digest_size 64 bytes and a 128-char hexdigest"""
import hashlib

_h512 = hashlib.sha512(b"abc")
assert _h512.digest_size == 64, f"sha512 digest_size = {_h512.digest_size!r}"
assert len(_h512.hexdigest()) == 128, f"sha512 hexdigest len = {len(_h512.hexdigest())!r}"

print("sha512_digest_sizes OK")
"###);
    assert_output(&out, r###"sha512_digest_sizes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/shake_incremental_and_attrs.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_shake_incremental_and_attrs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "shake_incremental_and_attrs"
# subject = "hashlib.shake_128"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.shake_128: shake_128 incremental update equals one-shot, .name is 'shake_128', and digest_size is 0 because length is per-call"""
import hashlib

_one = hashlib.shake_128(b"hello world")
_inc = hashlib.shake_128()
_inc.update(b"hello")
_inc.update(b" world")
assert _one.digest(16) == _inc.digest(16), "shake incremental == one-shot"
assert hashlib.shake_128().name == "shake_128", "shake_128 .name"
assert hashlib.shake_128().digest_size == 0, "shake digest_size is 0"

print("shake_incremental_and_attrs OK")
"###);
    assert_output(&out, r###"shake_incremental_and_attrs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/shake_known_vectors.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_shake_known_vectors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "shake_known_vectors"
# subject = "hashlib.shake_128"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.shake_128: variable-length hexdigest: shake_128(b'abc').hexdigest(16) and shake_256(b'abc').hexdigest(32) match their reference values"""
import hashlib

assert hashlib.shake_128(b"abc").hexdigest(16) == \
    "5881092dd818bf5cf8a3ddb793fbcba7", "shake_128('abc', 16)"
assert hashlib.shake_256(b"abc").hexdigest(32) == \
    "483366601360a8771c6863080cc4114d8db44530f8f1e1ee4f94ea37e78b5739", \
    "shake_256('abc', 32)"

print("shake_known_vectors OK")
"###);
    assert_output(&out, r###"shake_known_vectors OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/shake_length_controls_output.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_shake_length_controls_output() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "shake_length_controls_output"
# subject = "hashlib.shake_128"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.shake_128: the requested length sizes the output: shake_128.digest(8) is 8 bytes, hexdigest(8) is 16 hex chars, shake_256.digest(100) is 100 bytes"""
import hashlib

assert len(hashlib.shake_128(b"x").digest(8)) == 8, "shake digest(8) is 8 bytes"
assert len(hashlib.shake_128(b"x").hexdigest(8)) == 16, "shake hexdigest(8) is 16 hex chars"
assert len(hashlib.shake_256(b"x").digest(100)) == 100, "shake digest(100) is 100 bytes"

print("shake_length_controls_output OK")
"###);
    assert_output(&out, r###"shake_length_controls_output OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/shake_output_prefix_stable.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_shake_output_prefix_stable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "shake_output_prefix_stable"
# subject = "hashlib.shake_128"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
"""hashlib.shake_128: the XOF stream is prefix-stable: a 32-byte shake_128 output starts with the same bytes as the 8-byte output of the same input"""
import hashlib

_short = hashlib.shake_128(b"prefix").digest(8)
_long = hashlib.shake_128(b"prefix").digest(32)
assert _long[:8] == _short, "shake output is prefix-stable"

print("shake_output_prefix_stable OK")
"###);
    assert_output(&out, r###"shake_output_prefix_stable OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/hashlib/update_equivalent_to_concatenation.py`.
#[test]
fn test_gen_behavior_std_libs_hashlib_update_equivalent_to_concatenation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "update_equivalent_to_concatenation"
# subject = "hashlib.sha256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha256: incremental update() chained over 'hello'+'world' equals the one-shot digest of 'helloworld'"""
import hashlib

_h_cat = hashlib.sha256(b"helloworld")
_h_inc = hashlib.sha256()
_h_inc.update(b"hello")
_h_inc.update(b"world")
assert _h_cat.digest() == _h_inc.digest(), "update equivalent to cat"

print("update_equivalent_to_concatenation OK")
"###);
    assert_output(&out, r###"update_equivalent_to_concatenation OK
"###);
}
