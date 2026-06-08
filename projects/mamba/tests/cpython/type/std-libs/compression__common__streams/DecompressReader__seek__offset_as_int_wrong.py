# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression__common__streams"
# dimension = "type"
# case = "DecompressReader__seek__offset_as_int_wrong"
# subject = "compression._common._streams.DecompressReader.seek(offset: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/_common/_streams.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression._common._streams.DecompressReader.seek(offset: int); call it with the wrong type.

typeshed contract: offset is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from compression._common._streams import DecompressReader
obj = object.__new__(DecompressReader)
try:
    obj.seek("not_an_int")  # offset: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
