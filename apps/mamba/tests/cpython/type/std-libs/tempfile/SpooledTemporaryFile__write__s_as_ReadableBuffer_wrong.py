# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "SpooledTemporaryFile__write__s_as_ReadableBuffer_wrong"
# subject = "tempfile.SpooledTemporaryFile.write(s: ReadableBuffer)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed s"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed s
# mamba-strict-type: TypeError
"""Type wall: tempfile.SpooledTemporaryFile.write(s: ReadableBuffer); call it with the wrong type.

typeshed contract: s is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tempfile import SpooledTemporaryFile
obj = object.__new__(SpooledTemporaryFile)
try:
    obj.write(_W())  # s: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
