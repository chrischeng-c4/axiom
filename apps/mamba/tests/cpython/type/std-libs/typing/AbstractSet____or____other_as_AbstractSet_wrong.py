# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "type"
# case = "AbstractSet____or____other_as_AbstractSet_wrong"
# subject = "typing.AbstractSet.__or__(other: AbstractSet)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed other"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed other
# mamba-strict-type: TypeError
"""Type wall: typing.AbstractSet.__or__(other: AbstractSet); call it with the wrong type.

typeshed contract: other is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from typing import AbstractSet
obj = object.__new__(AbstractSet)
try:
    obj.__or__(_W())  # other: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
