# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_pathlib_tempfile_glob_fnmatch_value_ops"
# subject = "cpython321.test_pathlib_tempfile_glob_fnmatch_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_pathlib_tempfile_glob_fnmatch_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_pathlib_tempfile_glob_fnmatch_value_ops: execute CPython 3.12 seed test_pathlib_tempfile_glob_fnmatch_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Atomic 299 pass conformance — pathlib module (hasattr Path/PurePath/
# PurePosixPath/PureWindowsPath + type Path('/a/b') == 'PosixPath' +
# type PurePath('/a/b') == 'PurePosixPath') + tempfile module (hasattr
# gettempdir/NamedTemporaryFile/TemporaryDirectory/TemporaryFile/
# SpooledTemporaryFile/mkstemp/mkdtemp + gettempdir returns absolute
# str path) + glob module (hasattr glob/iglob/escape + escape) +
# fnmatch module (hasattr fnmatch/fnmatchcase/filter/translate +
# fnmatch/filter/translate value contracts) + mimetypes module
# (hasattr guess_type/guess_extension/add_type/MimeTypes/init/
# knownfiles + guess_type contracts) + quopri module (hasattr
# encodestring/decodestring/encode/decode + encodestring/decodestring
# round-trip) + binascii module (hasattr hexlify/unhexlify/a2b_hex/
# b2a_hex/a2b_base64/b2a_base64 + hexlify/unhexlify round-trip).
# All asserts match between CPython 3.12 and mamba.
import pathlib
import tempfile
import glob
import fnmatch
import mimetypes
import quopri
import binascii


_ledger: list[int] = []

# 1) pathlib — hasattr core surface
assert hasattr(pathlib, "Path") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePath") == True; _ledger.append(1)
assert hasattr(pathlib, "PurePosixPath") == True; _ledger.append(1)
assert hasattr(pathlib, "PureWindowsPath") == True; _ledger.append(1)

# 2) pathlib — type contracts (conformant subset)
assert type(pathlib.Path("/a/b")).__name__ == "PosixPath"; _ledger.append(1)
assert type(pathlib.PurePath("/a/b")).__name__ == "PurePosixPath"; _ledger.append(1)

# 3) tempfile — hasattr core surface
assert hasattr(tempfile, "gettempdir") == True; _ledger.append(1)
assert hasattr(tempfile, "NamedTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "TemporaryDirectory") == True; _ledger.append(1)
assert hasattr(tempfile, "TemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "SpooledTemporaryFile") == True; _ledger.append(1)
assert hasattr(tempfile, "mkstemp") == True; _ledger.append(1)
assert hasattr(tempfile, "mkdtemp") == True; _ledger.append(1)

# 4) tempfile — value contracts
assert isinstance(tempfile.gettempdir(), str) == True; _ledger.append(1)
assert tempfile.gettempdir().startswith("/") == True; _ledger.append(1)

# 5) glob — hasattr core surface
assert hasattr(glob, "glob") == True; _ledger.append(1)
assert hasattr(glob, "iglob") == True; _ledger.append(1)
assert hasattr(glob, "escape") == True; _ledger.append(1)

# 6) glob — value contracts
assert glob.escape("a[b]c") == "a[[]b]c"; _ledger.append(1)

# 7) fnmatch — hasattr core surface
assert hasattr(fnmatch, "fnmatch") == True; _ledger.append(1)
assert hasattr(fnmatch, "fnmatchcase") == True; _ledger.append(1)
assert hasattr(fnmatch, "filter") == True; _ledger.append(1)
assert hasattr(fnmatch, "translate") == True; _ledger.append(1)

# 8) fnmatch — value contracts
assert fnmatch.fnmatch("a.txt", "*.txt") == True; _ledger.append(1)
assert fnmatch.fnmatch("a.py", "*.txt") == False; _ledger.append(1)
assert fnmatch.filter(["a.txt", "b.py"], "*.txt") == ["a.txt"]; _ledger.append(1)
assert fnmatch.translate("*.txt") == "(?s:.*\\.txt)\\Z"; _ledger.append(1)

# 9) mimetypes — hasattr core surface
assert hasattr(mimetypes, "guess_type") == True; _ledger.append(1)
assert hasattr(mimetypes, "guess_extension") == True; _ledger.append(1)
assert hasattr(mimetypes, "add_type") == True; _ledger.append(1)
assert hasattr(mimetypes, "MimeTypes") == True; _ledger.append(1)
assert hasattr(mimetypes, "init") == True; _ledger.append(1)
assert hasattr(mimetypes, "knownfiles") == True; _ledger.append(1)

# 10) mimetypes — value contracts
assert mimetypes.guess_type("a.html") == ("text/html", None); _ledger.append(1)
assert mimetypes.guess_type("a.txt") == ("text/plain", None); _ledger.append(1)
assert mimetypes.guess_type("a.png") == ("image/png", None); _ledger.append(1)

# 11) quopri — hasattr core surface
assert hasattr(quopri, "encodestring") == True; _ledger.append(1)
assert hasattr(quopri, "decodestring") == True; _ledger.append(1)
assert hasattr(quopri, "encode") == True; _ledger.append(1)
assert hasattr(quopri, "decode") == True; _ledger.append(1)

# 12) quopri — round-trip
assert quopri.encodestring(b"hello") == b"hello"; _ledger.append(1)
assert quopri.encodestring(b"h=ello") == b"h=3Dello"; _ledger.append(1)
assert quopri.decodestring(b"h=3Dello") == b"h=ello"; _ledger.append(1)

# 13) binascii — hasattr core surface (conformant subset)
assert hasattr(binascii, "hexlify") == True; _ledger.append(1)
assert hasattr(binascii, "unhexlify") == True; _ledger.append(1)
assert hasattr(binascii, "a2b_hex") == True; _ledger.append(1)
assert hasattr(binascii, "b2a_hex") == True; _ledger.append(1)
assert hasattr(binascii, "a2b_base64") == True; _ledger.append(1)
assert hasattr(binascii, "b2a_base64") == True; _ledger.append(1)

# 14) binascii — value contracts
assert binascii.hexlify(b"abc") == b"616263"; _ledger.append(1)
assert binascii.unhexlify(b"616263") == b"abc"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_pathlib_tempfile_glob_fnmatch_value_ops {sum(_ledger)} asserts")
