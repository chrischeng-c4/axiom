# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "chunk"
# dimension = "type"
# case = "Chunk__read__size_as_int_wrong"
# subject = "chunk.Chunk.read(size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/chunk.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: chunk.Chunk.read(size: int); call it with the wrong type.

typeshed contract: size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from chunk import Chunk
obj = object.__new__(Chunk)
try:
    obj.read("not_an_int")  # size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
