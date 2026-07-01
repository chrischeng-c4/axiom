# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "sumprod__p_as_Iterable_wrong"
# subject = "math.sumprod(p: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.sumprod(p: Iterable); call it with the wrong type.

typeshed contract: p is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import sumprod
try:
    sumprod(_W(), None)  # p: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
