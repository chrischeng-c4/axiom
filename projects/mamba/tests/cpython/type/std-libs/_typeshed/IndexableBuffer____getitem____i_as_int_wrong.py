# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_typeshed"
# dimension = "type"
# case = "IndexableBuffer____getitem____i_as_int_wrong"
# subject = "_typeshed.IndexableBuffer.__getitem__(i: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_typeshed.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _typeshed.IndexableBuffer.__getitem__(i: int); call it with the wrong type.

typeshed contract: i is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _typeshed import IndexableBuffer
obj = object.__new__(IndexableBuffer)
try:
    obj.__getitem__("not_an_int")  # i: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
