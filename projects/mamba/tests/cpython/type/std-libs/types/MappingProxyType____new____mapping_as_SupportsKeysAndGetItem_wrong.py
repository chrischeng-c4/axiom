# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "type"
# case = "MappingProxyType____new____mapping_as_SupportsKeysAndGetItem_wrong"
# subject = "types.MappingProxyType.__new__(mapping: SupportsKeysAndGetItem)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/types.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: types.MappingProxyType.__new__(mapping: SupportsKeysAndGetItem); call it with the wrong type.

typeshed contract: mapping is SupportsKeysAndGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from types import MappingProxyType
obj = object.__new__(MappingProxyType)
try:
    obj.__new__(_W())  # mapping: SupportsKeysAndGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
