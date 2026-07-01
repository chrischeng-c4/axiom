# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "type"
# case = "PickleBuffer____new____buffer_as_ReadableBuffer_wrong"
# subject = "pickle.PickleBuffer.__new__(buffer: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickle.PickleBuffer.__new__(buffer: ReadableBuffer); call it with the wrong type.

typeshed contract: buffer is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickle import PickleBuffer
obj = object.__new__(PickleBuffer)
try:
    obj.__new__(_W())  # buffer: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
