# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "trunc__x_as__SupportsTrunc_wrong"
# subject = "math.trunc(x: _SupportsTrunc)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.trunc(x: _SupportsTrunc); call it with the wrong type.

typeshed contract: x is _SupportsTrunc. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import trunc
try:
    trunc(_W())  # x: _SupportsTrunc <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
