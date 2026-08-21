# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math_integer"
# dimension = "type"
# case = "comb__n_as_SupportsIndex_wrong"
# subject = "math.integer.comb(n: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math/integer.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.integer.comb(n: SupportsIndex); call it with the wrong type.

typeshed contract: n is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math.integer import comb
try:
    comb(_W(), None)  # n: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
