# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_gdbm"
# dimension = "type"
# case = "open__filename_as_StrOrBytesPath_wrong"
# subject = "_gdbm.open(filename: StrOrBytesPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_gdbm.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _gdbm.open(filename: StrOrBytesPath); call it with the wrong type.

typeshed contract: filename is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _gdbm import open
try:
    open(_W())  # filename: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
