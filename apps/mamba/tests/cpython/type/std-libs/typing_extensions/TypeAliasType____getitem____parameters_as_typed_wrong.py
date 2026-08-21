# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing_extensions"
# dimension = "type"
# case = "TypeAliasType____getitem____parameters_as_typed_wrong"
# subject = "typing_extensions.TypeAliasType.__getitem__(parameters: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed parameters"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing_extensions.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed parameters
# mamba-strict-type: TypeError
"""Type wall: typing_extensions.TypeAliasType.__getitem__(parameters: typed); call it with the wrong type.

typeshed contract: parameters is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from typing_extensions import TypeAliasType
obj = object.__new__(TypeAliasType)
try:
    obj.__getitem__(_W())  # parameters: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
