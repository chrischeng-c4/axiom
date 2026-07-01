# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "mod__a_as__T_contra_wrong"
# subject = "_operator.mod(a: _T_contra)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.mod(a: _T_contra); call it with the wrong type.

typeshed contract: a is _T_contra. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import mod
try:
    mod(_W(), None)  # a: _T_contra <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
