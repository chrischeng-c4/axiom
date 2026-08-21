# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_struct"
# dimension = "type"
# case = "Struct__unpack_from__buffer_as_ReadableBuffer_wrong"
# subject = "_struct.Struct.unpack_from(buffer: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_struct.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _struct.Struct.unpack_from(buffer: ReadableBuffer); call it with the wrong type.

typeshed contract: buffer is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _struct import Struct
obj = object.__new__(Struct)
try:
    obj.unpack_from(_W())  # buffer: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
