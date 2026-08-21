# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "IntFlag____or____other_as_int_wrong"
# subject = "enum.IntFlag.__or__(other: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.IntFlag.__or__(other: int); call it with the wrong type.

typeshed contract: other is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from enum import IntFlag
obj = object.__new__(IntFlag)
try:
    obj.__or__("not_an_int")  # other: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
