# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "LibraryLoader__init__dlltype_as_type_wrong"
# subject = "ctypes.LibraryLoader.__init__(dlltype: type)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed dlltype"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed dlltype
# mamba-strict-type: TypeError
"""Type wall: ctypes.LibraryLoader.__init__(dlltype: type); call it with the wrong type.

typeshed contract: dlltype is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ctypes import LibraryLoader
try:
    LibraryLoader(_W())  # dlltype: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
