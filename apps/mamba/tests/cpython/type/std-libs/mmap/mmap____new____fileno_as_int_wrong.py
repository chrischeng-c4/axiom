# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mmap"
# dimension = "type"
# case = "mmap____new____fileno_as_int_wrong"
# subject = "mmap.mmap.__new__(fileno: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mmap.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: mmap.mmap.__new__(fileno: int); call it with the wrong type.

typeshed contract: fileno is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from mmap import mmap
obj = object.__new__(mmap)
try:
    obj.__new__("not_an_int", 0)  # fileno: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
