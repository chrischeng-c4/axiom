# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "type"
# case = "MutableSequence____iadd____values_as_Iterable_wrong"
# subject = "typing.MutableSequence.__iadd__(values: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: typing.MutableSequence.__iadd__(values: Iterable); call it with the wrong type.

typeshed contract: values is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from typing import MutableSequence
obj = object.__new__(MutableSequence)
try:
    obj.__iadd__(_W())  # values: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
