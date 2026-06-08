# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_typeshed"
# dimension = "type"
# case = "SupportsGetItemBuffer____getitem____i_as_int_wrong"
# subject = "_typeshed.SupportsGetItemBuffer.__getitem__(i: int)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed i"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_typeshed.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed i
# mamba-strict-type: TypeError
"""Type wall: _typeshed.SupportsGetItemBuffer.__getitem__(i: int); call it with the wrong type.

typeshed contract: i is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _typeshed import SupportsGetItemBuffer
obj = object.__new__(SupportsGetItemBuffer)
try:
    obj.__getitem__("not_an_int")  # i: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
