# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "type"
# case = "MethodDescriptorType____get____owner_as_typed_wrong"
# subject = "types.MethodDescriptorType.__get__(owner: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/types.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: types.MethodDescriptorType.__get__(owner: typed); call it with the wrong type.

typeshed contract: owner is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from types import MethodDescriptorType
obj = object.__new__(MethodDescriptorType)
try:
    obj.__get__(None, _W())  # owner: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
