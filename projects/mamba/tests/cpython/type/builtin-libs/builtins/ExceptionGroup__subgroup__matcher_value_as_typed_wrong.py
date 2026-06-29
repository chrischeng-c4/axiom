# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "ExceptionGroup__subgroup__matcher_value_as_typed_wrong"
# subject = "builtins.ExceptionGroup.subgroup(matcher_value: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.ExceptionGroup.subgroup(matcher_value: typed); call it with the wrong type.

typeshed contract: matcher_value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import ExceptionGroup
obj = ExceptionGroup("msg", [ValueError("x")])
try:
    obj.subgroup(_W())  # matcher_value: typed <- wrong-typed
    print("no_typeerror:")  # mamba must reject the wrong-typed arg
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
