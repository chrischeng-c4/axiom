# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_io_tempfile_glob_fnmatch_value_ops"
# subject = "cpython321.test_io_tempfile_glob_fnmatch_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_io_tempfile_glob_fnmatch_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_io_tempfile_glob_fnmatch_value_ops: execute CPython 3.12 seed test_io_tempfile_glob_fnmatch_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 272 pass conformance — io module (hasattr StringIO/BytesIO) +
# tempfile module (hasattr gettempdir/gettempprefix/mkdtemp/mkstemp/
# NamedTemporaryFile/TemporaryFile/TemporaryDirectory/SpooledTemporary
# File/tempdir + gettempdir is non-empty str + gettempprefix is str) +
# glob module (hasattr glob/iglob/escape/has_magic + escape '*'/'?'/
# '[]' + glob of nonexistent yields []) + fnmatch module (hasattr
# fnmatch/fnmatchcase/filter/translate + fnmatch '*.txt' / mismatch /
# '?' / literal / fnmatchcase + filter matches / filter none / filter
# empty + translate returns str).
# All asserts match between CPython 3.12 and mamba.
import io
import tempfile
import glob
import fnmatch


_ledger: list[int] = []

# 1) io — minimal conforming hasattr surface
assert hasattr(io, "StringIO") == True; _ledger.append(1)
assert hasattr(io, "BytesIO") == True; _ledger.append(1)

# 2) tempfile — hasattr surface
assert hasattr(tempfile, "gettempdir") == True; _ledger.append(1)
assert hasattr(tempfile, "gettempprefix") == True; _ledger.append(1)
assert hasattr(tempfile, "mkdtemp") == True; _ledger.append(1)
assert hasattr(tempfile, "mkstemp") == True; _ledger.append(1)
assert hasattr(tempfile, "NamedTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "TemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "TemporaryDirectory") == True; _ledger.append(1)
assert hasattr(tempfile, "SpooledTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "tempdir") == True; _ledger.append(1)

# 3) tempfile — gettempdir/gettempprefix type contracts
assert isinstance(tempfile.gettempdir(), str) == True; _ledger.append(1)
assert (len(tempfile.gettempdir()) > 0) == True; _ledger.append(1)
assert isinstance(tempfile.gettempprefix(), str) == True; _ledger.append(1)

# 4) glob — hasattr surface
assert hasattr(glob, "glob") == True; _ledger.append(1)
assert hasattr(glob, "iglob") == True; _ledger.append(1)
assert hasattr(glob, "escape") == True; _ledger.append(1)
assert hasattr(glob, "has_magic") == True; _ledger.append(1)

# 5) glob — escape contracts
assert glob.escape("a*b") == "a[*]b"; _ledger.append(1)
assert glob.escape("a?b") == "a[?]b"; _ledger.append(1)
assert glob.escape("a[1]b") == "a[[]1]b"; _ledger.append(1)

# 6) glob — nonexistent path yields empty list
assert glob.glob("/nonexistent/path/*") == []; _ledger.append(1)

# 7) fnmatch — hasattr surface
assert hasattr(fnmatch, "fnmatch") == True; _ledger.append(1)
assert hasattr(fnmatch, "fnmatchcase") == True; _ledger.append(1)
assert hasattr(fnmatch, "filter") == True; _ledger.append(1)
assert hasattr(fnmatch, "translate") == True; _ledger.append(1)

# 8) fnmatch — pattern matching contracts
assert fnmatch.fnmatch("foo.txt", "*.txt") == True; _ledger.append(1)
assert fnmatch.fnmatch("foo.txt", "*.py") == False; _ledger.append(1)
assert fnmatch.fnmatch("foo.txt", "f?o.txt") == True; _ledger.append(1)
assert fnmatch.fnmatch("foo.txt", "foo.txt") == True; _ledger.append(1)
assert fnmatch.fnmatchcase("Foo.txt", "*.txt") == True; _ledger.append(1)

# 9) fnmatch — filter contracts
assert fnmatch.filter(["a.txt", "b.py", "c.txt"], "*.txt") == ["a.txt", "c.txt"]; _ledger.append(1)
assert fnmatch.filter(["a.txt", "b.py"], "*.cpp") == []; _ledger.append(1)
assert fnmatch.filter([], "*") == []; _ledger.append(1)

# 10) fnmatch — translate returns str
assert isinstance(fnmatch.translate("*.txt"), str) == True; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_io_tempfile_glob_fnmatch_value_ops {sum(_ledger)} asserts")
