# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "dict____ior____value_as_SupportsKeysAndGetItem_wrong"
# subject = "builtins.dict.__ior__(value: SupportsKeysAndGetItem)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed value"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed value
# mamba-strict-type: TypeError
"""Type wall: builtins.dict.__ior__(value: SupportsKeysAndGetItem); call it with the wrong type.

typeshed contract: value is SupportsKeysAndGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import dict
obj: dict[str, int] = {}
try:
    obj.__ior__(_W())  # value: SupportsKeysAndGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
