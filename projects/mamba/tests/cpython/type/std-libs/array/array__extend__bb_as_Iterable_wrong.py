# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "type"
# case = "array__extend__bb_as_Iterable_wrong"
# subject = "array.array.extend(bb: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/array.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: array.array.extend(bb: Iterable); call it with the wrong type.

typeshed contract: bb is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from array import array
obj = object.__new__(array)
try:
    obj.extend(_W())  # bb: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
