# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "type"
# case = "Random__uniform__a_as_float_wrong"
# subject = "random.Random.uniform(a: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/random.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: random.Random.uniform(a: float); call it with the wrong type.

typeshed contract: a is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from random import Random
obj = object.__new__(Random)
try:
    obj.uniform("not_a_float", 0.0)  # a: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
