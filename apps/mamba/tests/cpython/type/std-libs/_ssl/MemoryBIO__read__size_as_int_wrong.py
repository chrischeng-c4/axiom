# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "type"
# case = "MemoryBIO__read__size_as_int_wrong"
# subject = "_ssl.MemoryBIO.read(size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ssl.MemoryBIO.read(size: int); call it with the wrong type.

typeshed contract: size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ssl import MemoryBIO
obj = object.__new__(MemoryBIO)
try:
    obj.read("not_an_int")  # size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
