# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_weakrefset"
# dimension = "type"
# case = "WeakSet__isdisjoint__other_as_Iterable_wrong"
# subject = "_weakrefset.WeakSet.isdisjoint(other: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_weakrefset.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _weakrefset.WeakSet.isdisjoint(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _weakrefset import WeakSet
obj = object.__new__(WeakSet)
try:
    obj.isdisjoint(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
