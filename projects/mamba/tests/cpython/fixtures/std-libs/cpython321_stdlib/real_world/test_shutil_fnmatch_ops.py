# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_shutil_fnmatch_ops"
# subject = "cpython321.test_shutil_fnmatch_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_shutil_fnmatch_ops.py"
# status = "filled"
# ///
"""cpython321.test_shutil_fnmatch_ops: execute CPython 3.12 seed test_shutil_fnmatch_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `shutil.which` and `fnmatch`
# globbing helpers.
# Surface: shutil.which present/absent, fnmatch.fnmatch glob match,
# fnmatch.filter list filter, fnmatch.fnmatchcase case-sensitive form.
# Companion to stub/test_shutil.py + stub/test_fnmatch.py — vendored
# unittest seeds.
import shutil
import fnmatch
_ledger: list[int] = []
# shutil.which — /bin/ls is on PATH on macOS + Linux POSIX runners
assert shutil.which("ls") is not None; _ledger.append(1)
assert shutil.which("nonexistent_cmd_xyz_zzz") is None; _ledger.append(1)
# fnmatch.fnmatch — case-insensitive on Windows, case-sensitive on POSIX;
# only assert the deterministic POSIX-equivalent cases here.
assert fnmatch.fnmatch("file.txt", "*.txt"); _ledger.append(1)
assert not fnmatch.fnmatch("file.py", "*.txt"); _ledger.append(1)
assert fnmatch.fnmatch("readme.md", "readme.*"); _ledger.append(1)
assert fnmatch.fnmatch("a.txt", "?.txt"); _ledger.append(1)
assert not fnmatch.fnmatch("ab.txt", "?.txt"); _ledger.append(1)
# fnmatch.filter
got = fnmatch.filter(["a.txt", "b.py", "c.txt", "d.md"], "*.txt")
assert got == ["a.txt", "c.txt"]; _ledger.append(1)
# fnmatch.fnmatchcase — always case-sensitive
assert fnmatch.fnmatchcase("file.txt", "*.txt"); _ledger.append(1)
assert not fnmatch.fnmatchcase("File.TXT", "*.txt"); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_shutil_fnmatch_ops {sum(_ledger)} asserts")
