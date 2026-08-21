# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "type"
# case = "WIFSTOPPED__status_as_int_wrong"
# subject = "os.WIFSTOPPED(status: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/os.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: os.WIFSTOPPED(status: int); call it with the wrong type.

typeshed contract: status is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from os import WIFSTOPPED
try:
    WIFSTOPPED("not_an_int")  # status: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
