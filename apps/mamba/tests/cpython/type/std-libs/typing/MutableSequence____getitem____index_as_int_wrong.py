# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "type"
# case = "MutableSequence____getitem____index_as_int_wrong"
# subject = "typing.MutableSequence.__getitem__(index: int)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed index"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed index
# mamba-strict-type: TypeError
"""Type wall: typing.MutableSequence.__getitem__(index: int); call it with the wrong type.

typeshed contract: index is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from typing import MutableSequence
obj = object.__new__(MutableSequence)
try:
    obj.__getitem__("not_an_int")  # index: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
