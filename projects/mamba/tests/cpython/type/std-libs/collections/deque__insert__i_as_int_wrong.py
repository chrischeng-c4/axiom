# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "type"
# case = "deque__insert__i_as_int_wrong"
# subject = "collections.deque.insert(i: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/collections.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: collections.deque.insert(i: int); call it with the wrong type.

typeshed contract: i is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from collections import deque
obj = object.__new__(deque)
try:
    obj.insert("not_an_int", None)  # i: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
