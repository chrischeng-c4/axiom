# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "type"
# case = "open__filename_as__PathOrFile_wrong"
# subject = "lzma.open(filename: _PathOrFile)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lzma.open(filename: _PathOrFile); call it with the wrong type.

typeshed contract: filename is _PathOrFile. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lzma import open
try:
    open(_W())  # filename: _PathOrFile <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
