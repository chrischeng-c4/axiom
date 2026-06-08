# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "type"
# case = "open__filename_as__ReadableFileobj_wrong"
# subject = "bz2.open(filename: _ReadableFileobj)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bz2.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: bz2.open(filename: _ReadableFileobj); call it with the wrong type.

typeshed contract: filename is _ReadableFileobj. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bz2 import open
try:
    open(_W())  # filename: _ReadableFileobj <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
