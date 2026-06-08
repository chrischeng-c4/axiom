# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_struct"
# dimension = "type"
# case = "Struct__pack_into__buffer_as_WriteableBuffer_wrong"
# subject = "_struct.Struct.pack_into(buffer: WriteableBuffer)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed buffer"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_struct.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed buffer
# mamba-strict-type: TypeError
"""Type wall: _struct.Struct.pack_into(buffer: WriteableBuffer); call it with the wrong type.

typeshed contract: buffer is WriteableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _struct import Struct
obj = object.__new__(Struct)
try:
    obj.pack_into(_W(), 0)  # buffer: WriteableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
