# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binhex"
# dimension = "type"
# case = "BinHex__write__data_as_SizedBuffer_wrong"
# subject = "binhex.BinHex.write(data: SizedBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binhex.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binhex.BinHex.write(data: SizedBuffer); call it with the wrong type.

typeshed contract: data is SizedBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binhex import BinHex
obj = object.__new__(BinHex)
try:
    obj.write(_W())  # data: SizedBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
