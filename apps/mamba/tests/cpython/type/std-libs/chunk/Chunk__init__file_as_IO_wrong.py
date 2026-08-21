# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "chunk"
# dimension = "type"
# case = "Chunk__init__file_as_IO_wrong"
# subject = "chunk.Chunk.__init__(file: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/chunk.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: chunk.Chunk.__init__(file: IO); call it with the wrong type.

typeshed contract: file is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from chunk import Chunk
try:
    Chunk(_W())  # file: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
