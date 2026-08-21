# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_hashlib_ops"
# subject = "cpython321.test_hashlib_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_hashlib_ops.py"
# status = "filled"
# ///
"""cpython321.test_hashlib_ops: execute CPython 3.12 seed test_hashlib_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `hashlib` stdlib module.
# Surface: md5/sha1/sha256 hexdigest of canonical inputs, hex length.
# Companion to stub/test_hashlib.py — vendored unittest seed.
import hashlib
_ledger: list[int] = []
assert hashlib.md5(b"abc").hexdigest() == "900150983cd24fb0d6963f7d28e17f72"; _ledger.append(1)
assert hashlib.sha1(b"abc").hexdigest() == "a9993e364706816aba3e25717850c26c9cd0d89d"; _ledger.append(1)
assert hashlib.sha256(b"abc").hexdigest() == "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"; _ledger.append(1)
assert hashlib.sha256(b"").hexdigest() == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"; _ledger.append(1)
assert len(hashlib.md5(b"x").hexdigest()) == 32; _ledger.append(1)
assert len(hashlib.sha256(b"x").hexdigest()) == 64; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_hashlib_ops {sum(_ledger)} asserts")
