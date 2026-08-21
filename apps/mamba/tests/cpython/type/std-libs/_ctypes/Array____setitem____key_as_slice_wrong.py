# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "Array____setitem____key_as_slice_wrong"
# subject = "_ctypes.Array.__setitem__(key: slice)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.Array.__setitem__(key: slice); call it with the wrong type.

typeshed contract: key is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


import _ctypes
import ctypes

obj = (ctypes.c_int * 2)(1, 2)
try:
    _ctypes.Array.__setitem__(obj, _W(), 1)  # key: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
