# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "msvcrt"
# dimension = "type"
# case = "locking__fd_as_int_wrong"
# subject = "msvcrt.locking(fd: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/msvcrt.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: msvcrt.locking(fd: int); call it with the wrong type.

typeshed contract: fd is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from msvcrt import locking
try:
    locking("not_an_int", 0, 0)  # fd: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
