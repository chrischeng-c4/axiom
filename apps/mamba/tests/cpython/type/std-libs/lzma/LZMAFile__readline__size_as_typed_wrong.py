# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "type"
# case = "LZMAFile__readline__size_as_typed_wrong"
# subject = "lzma.LZMAFile.readline(size: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lzma.LZMAFile.readline(size: typed); call it with the wrong type.

typeshed contract: size is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lzma import LZMAFile
obj = object.__new__(LZMAFile)
try:
    obj.readline(_W())  # size: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
