# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes_macholib_dyld"
# dimension = "type"
# case = "framework_find__fn_as_str_wrong"
# subject = "ctypes.macholib.dyld.framework_find(fn: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes/macholib/dyld.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.macholib.dyld.framework_find(fn: str); call it with the wrong type.

typeshed contract: fn is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ctypes.macholib.dyld import framework_find
try:
    framework_find(12345)  # fn: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
