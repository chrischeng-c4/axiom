# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "type"
# case = "loads__bytes_as_ReadableBuffer_wrong"
# subject = "marshal.loads(bytes: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/marshal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: marshal.loads(bytes: ReadableBuffer); call it with the wrong type.

typeshed contract: bytes is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from marshal import loads
try:
    loads(_W())  # bytes: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
