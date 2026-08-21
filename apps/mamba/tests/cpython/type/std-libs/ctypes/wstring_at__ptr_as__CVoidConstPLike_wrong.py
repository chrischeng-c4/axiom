# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "type"
# case = "wstring_at__ptr_as__CVoidConstPLike_wrong"
# subject = "ctypes.wstring_at(ptr: _CVoidConstPLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.wstring_at(ptr: _CVoidConstPLike); call it with the wrong type.

typeshed contract: ptr is _CVoidConstPLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ctypes import wstring_at
try:
    wstring_at(_W())  # ptr: _CVoidConstPLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
