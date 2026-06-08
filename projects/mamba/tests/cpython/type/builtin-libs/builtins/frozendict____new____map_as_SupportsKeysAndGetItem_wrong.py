# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "frozendict____new____map_as_SupportsKeysAndGetItem_wrong"
# subject = "builtins.frozendict.__new__(map: SupportsKeysAndGetItem)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed map"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed map
# mamba-strict-type: TypeError
"""Type wall: builtins.frozendict.__new__(map: SupportsKeysAndGetItem); call it with the wrong type.

typeshed contract: map is SupportsKeysAndGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import frozendict
obj = object.__new__(frozendict)
try:
    obj.__new__(_W())  # map: SupportsKeysAndGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
