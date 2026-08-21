# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "type"
# case = "GeneratorType__send__arg_as__SendT_contra_wrong"
# subject = "types.GeneratorType.send(arg: _SendT_contra)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/types.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: types.GeneratorType.send(arg: _SendT_contra); call it with the wrong type.

typeshed contract: arg is _SendT_contra. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from types import GeneratorType
obj = object.__new__(GeneratorType)
try:
    obj.send(_W())  # arg: _SendT_contra <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
