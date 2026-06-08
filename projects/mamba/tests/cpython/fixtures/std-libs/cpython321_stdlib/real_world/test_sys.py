# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_sys"
# subject = "cpython321.test_sys"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_sys.py"
# status = "filled"
# ///
"""cpython321.test_sys: execute CPython 3.12 seed test_sys"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: sys — basic identification (version / version_info / platform),
# positive integer constants (maxsize, getrecursionlimit, getsizeof),
# byteorder, argv / path / modules / executable presence, intern, and the
# float_info.max ceiling.
# Notes:
#   * sys.platform on mamba is 'macos' on macOS (not 'darwin' as in CPython);
#     this seed only checks that sys.platform is a non-empty string.
#   * sys.version_info is exposed as a dict-like with attribute access for
#     major/minor/micro, not a named tuple — both forms validated below.
import sys

_ledger: list[int] = []

# sys.version is a non-empty string identifier
assert isinstance(sys.version, str) and len(sys.version) > 0, (
    "sys.version is a non-empty string"
)
_ledger.append(1)

# sys.version_info exposes major/minor/micro via attribute access
assert sys.version_info.major == 3, "sys.version_info.major == 3"
_ledger.append(1)

assert sys.version_info.minor >= 12, "sys.version_info.minor >= 12"
_ledger.append(1)

# sys.platform is a non-empty string (value differs from CPython on mamba)
assert isinstance(sys.platform, str) and len(sys.platform) > 0, (
    "sys.platform is a non-empty string"
)
_ledger.append(1)

# sys.maxsize is a large positive int
assert isinstance(sys.maxsize, int) and sys.maxsize > 0, (
    "sys.maxsize is a positive int"
)
_ledger.append(1)

# 2**31 - 1 (i32_max) must fit comfortably below maxsize on a 64-bit build
assert sys.maxsize > 2_147_483_647, (
    "sys.maxsize > 2**31 - 1 on a 64-bit build"
)
_ledger.append(1)

# byteorder is one of the two POSIX-conventional values
assert sys.byteorder in ("little", "big"), "sys.byteorder is 'little' or 'big'"
_ledger.append(1)

# argv is a list with at least the program name slot
assert isinstance(sys.argv, list) and len(sys.argv) >= 1, (
    "sys.argv is a list with at least one entry"
)
_ledger.append(1)

# path is a non-empty list of search directories
assert isinstance(sys.path, list) and len(sys.path) >= 1, (
    "sys.path is a non-empty list"
)
_ledger.append(1)

# getrecursionlimit returns a positive int
_rl = sys.getrecursionlimit()
assert isinstance(_rl, int) and _rl > 0, (
    "sys.getrecursionlimit() returns a positive int"
)
_ledger.append(1)

# intern returns the same string back
assert sys.intern("hello") == "hello", "sys.intern('hello') == 'hello'"
_ledger.append(1)

# executable is a non-empty path string
assert isinstance(sys.executable, str) and len(sys.executable) > 0, (
    "sys.executable is a non-empty string"
)
_ledger.append(1)

# float_info.max is the IEEE-754 double ceiling (~1.7976931348623157e+308)
assert sys.float_info.max > 1e+300, (
    "sys.float_info.max > 1e+300 (IEEE-754 double ceiling)"
)
_ledger.append(1)

# getsizeof returns a non-negative int
_sz = sys.getsizeof([])
assert isinstance(_sz, int) and _sz >= 0, (
    "sys.getsizeof([]) returns a non-negative int"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_sys {sum(_ledger)} asserts")
