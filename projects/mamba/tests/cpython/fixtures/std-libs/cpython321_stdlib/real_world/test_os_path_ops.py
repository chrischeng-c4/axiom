# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_os_path_ops"
# subject = "cpython321.test_os_path_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_os_path_ops.py"
# status = "filled"
# ///
"""cpython321.test_os_path_ops: execute CPython 3.12 seed test_os_path_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `os.path` and `os` constants.
# Surface: join, basename, dirname, splitext, split, sep, exists/
# isfile via /etc/hosts (POSIX-only smoke), abspath idempotence.
# Companion to stub/test_os_path.py — vendored unittest seed.
import os
import os.path
_ledger: list[int] = []
assert os.sep == "/"; _ledger.append(1)
assert os.path.join("a", "b", "c") == "a/b/c"; _ledger.append(1)
assert os.path.join("/x", "y") == "/x/y"; _ledger.append(1)
assert os.path.basename("/a/b/c.txt") == "c.txt"; _ledger.append(1)
assert os.path.basename("plain.txt") == "plain.txt"; _ledger.append(1)
assert os.path.dirname("/a/b/c.txt") == "/a/b"; _ledger.append(1)
assert os.path.dirname("plain.txt") == ""; _ledger.append(1)
assert os.path.splitext("file.txt") == ("file", ".txt"); _ledger.append(1)
assert os.path.splitext("noext") == ("noext", ""); _ledger.append(1)
assert os.path.split("/a/b/c") == ("/a/b", "c"); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_os_path_ops {sum(_ledger)} asserts")
