# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "cross_flavour_inequality"
# subject = "pathlib.PurePosixPath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePosixPath: POSIX and Windows pure paths are never equal even with identical strings, ordering comparisons across flavours raise TypeError, the non-host concrete flavour cannot be instantiated, and path APIs reject bytes arguments"""
import pathlib

import os
PurePosixPath = pathlib.PurePosixPath
PureWindowsPath = pathlib.PureWindowsPath
PosixPath = pathlib.PosixPath
WindowsPath = pathlib.WindowsPath
PurePath = pathlib.PurePath

# Different flavours are never equal, even with identical string content.
assert PurePosixPath("a") != PureWindowsPath("a"), "cross-flavour inequality"

# Different flavours are unordered: every ordering comparison raises TypeError.
_p = PurePosixPath("a")
_q = PureWindowsPath("a")
for _cmp in (
    lambda: _p < _q,
    lambda: _p <= _q,
    lambda: _p > _q,
    lambda: _p >= _q,
):
    try:
        _cmp()
        raise AssertionError("ordering across flavours should raise TypeError")
    except TypeError:
        pass

# The concrete flavour that does not match the host OS cannot be instantiated.
if os.name == "nt":
    try:
        PosixPath()
        raise AssertionError("PosixPath() on Windows should raise")
    except NotImplementedError:
        pass
else:
    try:
        WindowsPath()
        raise AssertionError("WindowsPath() on POSIX should raise")
    except NotImplementedError:
        pass

# Path APIs reject bytes; only str / os.PathLike-returning-str is accepted.
_P = PurePath
for _make in (
    lambda: _P(b"a"),
    lambda: _P(b"a", "b"),
    lambda: _P("a", b"b"),
    lambda: _P("a").joinpath(b"b"),
    lambda: _P("a") / b"b",
    lambda: b"a" / _P("b"),
    lambda: _P("a").match(b"b"),
    lambda: _P("a").with_name(b"b"),
    lambda: _P("a").with_suffix(b"b"),
):
    try:
        _make()
        raise AssertionError("bytes argument should raise TypeError")
    except TypeError:
        pass
print("cross_flavour_inequality OK")
