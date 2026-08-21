# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "CDLL____getitem____name_or_ordinal_as_str_wrong"
# subject = "ctypes.CDLL.__getitem__(name_or_ordinal: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.CDLL.__getitem__(name_or_ordinal: str); call it with the wrong type.

typeshed contract: name_or_ordinal is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ctypes import CDLL
obj = object.__new__(CDLL)
try:
    obj.__getitem__(12345)  # name_or_ordinal: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
