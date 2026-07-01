# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "type"
# case = "AsyncGeneratorType__asend__val_as__SendT_contra_wrong"
# subject = "types.AsyncGeneratorType.asend(val: _SendT_contra)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/types.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: types.AsyncGeneratorType.asend(val: _SendT_contra); call it with the wrong type.

typeshed contract: val is _SendT_contra. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from types import AsyncGeneratorType
obj = object.__new__(AsyncGeneratorType)
try:
    obj.asend(_W())  # val: _SendT_contra <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
