# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "BaseExceptionGroup__split__matcher_value_as_Callable_wrong"
# subject = "builtins.BaseExceptionGroup.split(matcher_value: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.BaseExceptionGroup.split(matcher_value: Callable); call it with the wrong type.

typeshed contract: matcher_value is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import BaseExceptionGroup
obj = BaseExceptionGroup("msg", [ValueError("x")])
try:
    obj.split(_W())  # matcher_value: Callable <- wrong-typed
    print("no_typeerror:")  # mamba must reject the wrong-typed arg
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
