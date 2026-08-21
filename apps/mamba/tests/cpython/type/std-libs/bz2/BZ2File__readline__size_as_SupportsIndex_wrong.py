# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "type"
# case = "BZ2File__readline__size_as_SupportsIndex_wrong"
# subject = "bz2.BZ2File.readline(size: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bz2.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bz2.BZ2File.readline(size: SupportsIndex); call it with the wrong type.

typeshed contract: size is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bz2 import BZ2File
obj = object.__new__(BZ2File)
try:
    obj.readline(_W())  # size: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
