# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "binomialvariate_rejects_bad_params"
# subject = "random.Random.binomialvariate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.binomialvariate: binomialvariate rejects negative n and out-of-range p with ValueError: n=-1, (n=1,p=-0.5), (n=1,p=1.5) all raise"""
import random

gen = random.Random(0)
for kwargs in [dict(n=-1), dict(n=1, p=-0.5), dict(n=1, p=1.5)]:
    try:
        gen.binomialvariate(**kwargs)
        raise AssertionError(f"expected ValueError for binomialvariate({kwargs})")
    except ValueError:
        pass

print("binomialvariate_rejects_bad_params OK")
