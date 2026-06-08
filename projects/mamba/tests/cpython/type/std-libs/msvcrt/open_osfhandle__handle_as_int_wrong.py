# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msvcrt"
# dimension = "type"
# case = "open_osfhandle__handle_as_int_wrong"
# subject = "msvcrt.open_osfhandle(handle: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msvcrt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msvcrt.open_osfhandle(handle: int); call it with the wrong type.

typeshed contract: handle is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from msvcrt import open_osfhandle
try:
    open_osfhandle("not_an_int", 0)  # handle: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
