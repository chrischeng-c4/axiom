# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ctypes"
# dimension = "type"
# case = "dlopen__name_as_StrOrBytesPath_wrong"
# subject = "_ctypes.dlopen(name: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ctypes.dlopen(name: StrOrBytesPath); call it with the wrong type.

typeshed contract: name is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _ctypes import dlopen
try:
    dlopen(_W())  # name: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
