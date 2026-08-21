# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "type"
# case = "umask__mask_as_int_wrong"
# subject = "os.umask(mask: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/os.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: os.umask(mask: int); call it with the wrong type.

typeshed contract: mask is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from os import umask
try:
    umask("not_an_int")  # mask: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
