# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____lt____value_as_ReadableBuffer_wrong"
# subject = "builtins.bytearray.__lt__(value: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__lt__(value: ReadableBuffer); call it with the wrong type.

typeshed contract: value is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.__lt__(_W())  # value: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
