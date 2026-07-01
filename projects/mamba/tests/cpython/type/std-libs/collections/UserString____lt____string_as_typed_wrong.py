# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "type"
# case = "UserString____lt____string_as_typed_wrong"
# subject = "collections.UserString.__lt__(string: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/collections.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: collections.UserString.__lt__(string: typed); call it with the wrong type.

typeshed contract: string is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from collections import UserString
obj = object.__new__(UserString)
try:
    obj.__lt__(_W())  # string: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
