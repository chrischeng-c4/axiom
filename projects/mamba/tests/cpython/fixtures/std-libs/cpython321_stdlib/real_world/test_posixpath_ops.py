# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_posixpath_ops"
# subject = "cpython321.test_posixpath_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_posixpath_ops.py"
# status = "filled"
# ///
"""cpython321.test_posixpath_ops: execute CPython 3.12 seed test_posixpath_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `posixpath` path-manipulation
# surface — the POSIX-specific path module used on Linux + macOS.
# Surface: join / basename / dirname / split / splitext / normpath /
# isabs on canonical POSIX paths.
# Companion to stub/test_posixpath.py — vendored unittest seed.
import posixpath
_ledger: list[int] = []
assert posixpath.join("/usr", "bin", "python") == "/usr/bin/python"; _ledger.append(1)
assert posixpath.basename("/usr/bin/python") == "python"; _ledger.append(1)
assert posixpath.dirname("/usr/bin/python") == "/usr/bin"; _ledger.append(1)
assert posixpath.split("/usr/bin/python") == ("/usr/bin", "python"); _ledger.append(1)
# splitext splits on the LAST dot, not the first
assert posixpath.splitext("/tmp/file.tar.gz") == ("/tmp/file.tar", ".gz"); _ledger.append(1)
assert posixpath.splitext("/tmp/no_ext") == ("/tmp/no_ext", ""); _ledger.append(1)
# normpath collapses doubled separators and "/./" components
assert posixpath.normpath("/usr//bin/./python") == "/usr/bin/python"; _ledger.append(1)
# isabs is true for paths starting with "/"
assert posixpath.isabs("/foo"); _ledger.append(1)
assert not posixpath.isabs("foo"); _ledger.append(1)
assert not posixpath.isabs("relative/path"); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_posixpath_ops {sum(_ledger)} asserts")
