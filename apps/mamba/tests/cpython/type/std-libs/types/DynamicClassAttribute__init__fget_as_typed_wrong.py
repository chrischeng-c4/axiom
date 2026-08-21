# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "type"
# case = "DynamicClassAttribute__init__fget_as_typed_wrong"
# subject = "types.DynamicClassAttribute.__init__(fget: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/types.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: types.DynamicClassAttribute.__init__(fget: typed); call it with the wrong type.

typeshed contract: fget is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from types import DynamicClassAttribute
try:
    DynamicClassAttribute(_W())  # fget: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
