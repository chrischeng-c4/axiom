# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_collections_abc"
# dimension = "type"
# case = "dict_keys__isdisjoint__other_as_Iterable_wrong"
# subject = "_collections_abc.dict_keys.isdisjoint(other: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_collections_abc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _collections_abc.dict_keys.isdisjoint(other: Iterable); call it with the wrong type.

typeshed contract: other is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _collections_abc import dict_keys
obj = object.__new__(dict_keys)
try:
    obj.isdisjoint(_W())  # other: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
