# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bz2"
# dimension = "type"
# case = "BZ2File__init__filename_as__WritableFileobj_wrong"
# subject = "bz2.BZ2File.__init__(filename: _WritableFileobj)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed filename"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/bz2.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed filename
# mamba-strict-type: TypeError
"""Type wall: bz2.BZ2File.__init__(filename: _WritableFileobj); call it with the wrong type.

typeshed contract: filename is _WritableFileobj. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from bz2 import BZ2File
try:
    BZ2File(_W(), None)  # filename: _WritableFileobj <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
