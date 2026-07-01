# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "type"
# case = "DynamicClassAttribute__setter__fset_as_Callable_wrong"
# subject = "types.DynamicClassAttribute.setter(fset: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/types.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: types.DynamicClassAttribute.setter(fset: Callable); call it with the wrong type.

typeshed contract: fset is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from types import DynamicClassAttribute
obj = object.__new__(DynamicClassAttribute)
try:
    obj.setter(_W())  # fset: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
