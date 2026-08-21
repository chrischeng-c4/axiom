# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "type"
# case = "Random__lognormvariate__mu_as_float_wrong"
# subject = "random.Random.lognormvariate(mu: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/random.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: random.Random.lognormvariate(mu: float); call it with the wrong type.

typeshed contract: mu is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from random import Random
obj = object.__new__(Random)
try:
    obj.lognormvariate("not_a_float", 0.0)  # mu: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
