# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_tempfile"
# subject = "cpython321.test_tempfile"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_tempfile.py"
# status = "filled"
# ///
"""cpython321.test_tempfile: execute CPython 3.12 seed test_tempfile"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: tempfile (mkdtemp, mkstemp, gettempdir, gettempprefix).
# NamedTemporaryFile and TemporaryDirectory return tuple / str stubs (not
# context-manager objects) under mamba and are intentionally NOT exercised
# here; tracked separately.
import tempfile
import os

_ledger: list[int] = []

# gettempdir returns a non-empty string path
_gtd = tempfile.gettempdir()
assert isinstance(_gtd, str) and len(_gtd) > 0, "gettempdir() is a non-empty string"
_ledger.append(1)

# gettempprefix returns a non-empty string
_gtp = tempfile.gettempprefix()
assert isinstance(_gtp, str) and len(_gtp) > 0, "gettempprefix() is a non-empty string"
_ledger.append(1)

# mkdtemp returns a str path that exists on disk
_d = tempfile.mkdtemp()
assert isinstance(_d, str), "mkdtemp() returns a str"
_ledger.append(1)

assert os.path.exists(_d), "mkdtemp() path exists on disk"
_ledger.append(1)

# The fresh temp dir is a directory, not a file
assert os.path.isdir(_d), "mkdtemp() path is a directory"
_ledger.append(1)

# Successive mkdtemp calls return distinct paths
_d2 = tempfile.mkdtemp()
assert _d != _d2, "two mkdtemp() calls return distinct paths"
_ledger.append(1)

# mkdtemp output lives under gettempdir
assert _d.startswith(_gtd), "mkdtemp() path is under gettempdir()"
_ledger.append(1)

# mkstemp returns a (fd, path) tuple
_r = tempfile.mkstemp()
assert isinstance(_r, tuple), "mkstemp() returns a tuple"
_ledger.append(1)

assert len(_r) == 2, "mkstemp() tuple has two entries"
_ledger.append(1)

# The path component exists on disk and is a file
_fd, _p = _r
assert os.path.exists(_p), "mkstemp() path exists on disk"
_ledger.append(1)

# Cleanup
os.rmdir(_d)
os.rmdir(_d2)
os.remove(_p)

# After cleanup the paths no longer exist
assert not os.path.exists(_d), "mkdtemp() path removed after rmdir"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_tempfile {sum(_ledger)} asserts")
