# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_charmap"
# dimension = "type"
# case = "IncrementalDecoder__decode__input_as_ReadableBuffer_wrong"
# subject = "encodings.charmap.IncrementalDecoder.decode(input: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/charmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.charmap.IncrementalDecoder.decode(input: ReadableBuffer); call it with the wrong type.

typeshed contract: input is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.charmap import IncrementalDecoder
obj = object.__new__(IncrementalDecoder)
try:
    obj.decode(_W())  # input: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
