# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_typeshed"
# dimension = "type"
# case = "SupportsGetItemBuffer____getitem____slice_as_slice_wrong"
# subject = "_typeshed.SupportsGetItemBuffer.__getitem__(slice: slice)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed slice"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_typeshed.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed slice
# mamba-strict-type: TypeError
"""Type wall: _typeshed.SupportsGetItemBuffer.__getitem__(slice: slice); call it with the wrong type.

typeshed contract: slice is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _typeshed import SupportsGetItemBuffer
obj = object.__new__(SupportsGetItemBuffer)
try:
    obj.__getitem__(_W())  # slice: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
