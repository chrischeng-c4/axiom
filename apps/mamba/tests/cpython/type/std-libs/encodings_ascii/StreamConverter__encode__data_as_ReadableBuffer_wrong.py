# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_ascii"
# dimension = "type"
# case = "StreamConverter__encode__data_as_ReadableBuffer_wrong"
# subject = "encodings.ascii.StreamConverter.encode(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/ascii.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.ascii.StreamConverter.encode(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.ascii import StreamConverter
try:
    StreamConverter.encode(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
