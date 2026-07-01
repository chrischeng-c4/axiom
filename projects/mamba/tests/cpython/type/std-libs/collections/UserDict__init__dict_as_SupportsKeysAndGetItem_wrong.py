# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "type"
# case = "UserDict__init__dict_as_SupportsKeysAndGetItem_wrong"
# subject = "collections.UserDict.__init__(dict: SupportsKeysAndGetItem)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/collections.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: collections.UserDict.__init__(dict: SupportsKeysAndGetItem); call it with the wrong type.

typeshed contract: dict is SupportsKeysAndGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from collections import UserDict
try:
    UserDict(_W())  # dict: SupportsKeysAndGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
