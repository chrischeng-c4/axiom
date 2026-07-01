# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_lzma"
# dimension = "type"
# case = "LZMACompressor__compress__data_as_ReadableBuffer_wrong"
# subject = "_lzma.LZMACompressor.compress(data: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _lzma.LZMACompressor.compress(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _lzma import LZMACompressor
obj = object.__new__(LZMACompressor)
try:
    obj.compress(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
