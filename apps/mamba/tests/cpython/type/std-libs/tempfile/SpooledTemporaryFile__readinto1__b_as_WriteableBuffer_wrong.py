# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "SpooledTemporaryFile__readinto1__b_as_WriteableBuffer_wrong"
# subject = "tempfile.SpooledTemporaryFile.readinto1(b: WriteableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tempfile.SpooledTemporaryFile.readinto1(b: WriteableBuffer); call it with the wrong type.

typeshed contract: b is WriteableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tempfile import SpooledTemporaryFile
obj = object.__new__(SpooledTemporaryFile)
try:
    obj.readinto1(_W())  # b: WriteableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
