# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "degenerate_params_collapse_to_constant"
# subject = "random.Random"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random: degenerate parameters collapse distributions to a constant: uniform(10,10)=10, triangular(10,10,10)=10, gauss(10,0)=10, expovariate(inf)=0, binomialvariate(10,1.0)=10, etc."""
import random

gen = random.Random(0)
constants = [
    (gen.uniform, (10.0, 10.0), 10.0),
    (gen.triangular, (10.0, 10.0, 10.0), 10.0),
    (gen.gauss, (10.0, 0.0), 10.0),
    (gen.normalvariate, (10.0, 0.0), 10.0),
    (gen.expovariate, (float("inf"),), 0.0),
    (gen.paretovariate, (float("inf"),), 1.0),
    (gen.binomialvariate, (10, 0.0), 0),
    (gen.binomialvariate, (10, 1.0), 10),
]
for variate, args, expected in constants:
    assert variate(*args) == expected, f"{variate.__name__}{args} != {expected!r}"

print("degenerate_params_collapse_to_constant OK")
