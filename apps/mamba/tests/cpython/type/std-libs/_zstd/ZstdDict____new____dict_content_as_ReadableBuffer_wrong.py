# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_zstd"
# dimension = "type"
# case = "ZstdDict____new____dict_content_as_ReadableBuffer_wrong"
# subject = "_zstd.ZstdDict.__new__(dict_content: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _zstd.ZstdDict.__new__(dict_content: ReadableBuffer); call it with the wrong type.

typeshed contract: dict_content is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _zstd import ZstdDict
obj = object.__new__(ZstdDict)
try:
    obj.__new__(_W())  # dict_content: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
