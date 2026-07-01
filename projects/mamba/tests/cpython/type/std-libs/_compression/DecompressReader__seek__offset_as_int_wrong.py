# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_compression"
# dimension = "type"
# case = "DecompressReader__seek__offset_as_int_wrong"
# subject = "_compression.DecompressReader.seek(offset: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_compression.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _compression.DecompressReader.seek(offset: int); call it with the wrong type.

typeshed contract: offset is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _compression import DecompressReader
obj = object.__new__(DecompressReader)
try:
    obj.seek("not_an_int")  # offset: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
