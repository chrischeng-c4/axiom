# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "type"
# case = "TypeVarTuple____typing_subst____arg_as_Never_wrong"
# subject = "typing.TypeVarTuple.__typing_subst__(arg: Never)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed arg"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed arg
# mamba-strict-type: TypeError
"""Type wall: typing.TypeVarTuple.__typing_subst__(arg: Never); call it with the wrong type.

typeshed contract: arg is Never. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from typing import TypeVarTuple
obj = object.__new__(TypeVarTuple)
try:
    obj.__typing_subst__(_W())  # arg: Never <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
