# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "type"
# case = "Container____contains____x_as__ContainerT_contra_wrong"
# subject = "typing.Container.__contains__(x: _ContainerT_contra)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed x"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed x
# mamba-strict-type: TypeError
"""Type wall: typing.Container.__contains__(x: _ContainerT_contra); call it with the wrong type.

typeshed contract: x is _ContainerT_contra. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from typing import Container
obj = object.__new__(Container)
try:
    obj.__contains__(_W())  # x: _ContainerT_contra <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
