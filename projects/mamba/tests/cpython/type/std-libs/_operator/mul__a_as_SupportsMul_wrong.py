# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_operator"
# dimension = "type"
# case = "mul__a_as_SupportsMul_wrong"
# subject = "_operator.mul(a: SupportsMul)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_operator.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _operator.mul(a: SupportsMul); call it with the wrong type.

typeshed contract: a is SupportsMul. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _operator import mul
try:
    mul(_W(), None)  # a: SupportsMul <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
