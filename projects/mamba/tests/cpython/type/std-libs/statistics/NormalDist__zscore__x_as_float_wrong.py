# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "statistics"
# dimension = "type"
# case = "NormalDist__zscore__x_as_float_wrong"
# subject = "statistics.NormalDist.zscore(x: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/statistics.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: statistics.NormalDist.zscore(x: float); call it with the wrong type.

typeshed contract: x is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from statistics import NormalDist
obj = object.__new__(NormalDist)
try:
    obj.zscore("not_a_float")  # x: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
