# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "type"
# case = "ParamSpecKwargs____new____origin_as_ParamSpec_wrong"
# subject = "typing.ParamSpecKwargs.__new__(origin: ParamSpec)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: typing.ParamSpecKwargs.__new__(origin: ParamSpec); call it with the wrong type.

typeshed contract: origin is ParamSpec. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from typing import ParamSpecKwargs
obj = object.__new__(ParamSpecKwargs)
try:
    obj.__new__(_W())  # origin: ParamSpec <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
