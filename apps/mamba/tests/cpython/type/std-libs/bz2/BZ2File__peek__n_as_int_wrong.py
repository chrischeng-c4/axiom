# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "type"
# case = "BZ2File__peek__n_as_int_wrong"
# subject = "bz2.BZ2File.peek(n: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bz2.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bz2.BZ2File.peek(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from bz2 import BZ2File
obj = object.__new__(BZ2File)
try:
    obj.peek("not_an_int")  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
