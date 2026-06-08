# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_typeshed"
# dimension = "type"
# case = "SupportsContainsAndGetItem____getitem____key_as__KT_contra_wrong"
# subject = "_typeshed.SupportsContainsAndGetItem.__getitem__(key: _KT_contra)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed key"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_typeshed.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed key
# mamba-strict-type: TypeError
"""Type wall: _typeshed.SupportsContainsAndGetItem.__getitem__(key: _KT_contra); call it with the wrong type.

typeshed contract: key is _KT_contra. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _typeshed import SupportsContainsAndGetItem
obj = object.__new__(SupportsContainsAndGetItem)
try:
    obj.__getitem__(_W())  # key: _KT_contra <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
