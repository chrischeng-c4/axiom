# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "random"
# dimension = "behavior"
# case = "vonmisesvariate_stays_in_circle"
# subject = "random.Random.vonmisesvariate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_random.py"
# status = "filled"
# ///
"""random.Random.vonmisesvariate: vonmisesvariate(mu, kappa) stays within [0, 2*pi] across a grid of mu/kappa combinations including kappa=0"""
import random

gen = random.Random(0)
for mu in (0.0, 0.1, 3.1, 6.2):
    for kappa in (0.0, 2.3, 500.0):
        for _ in range(20):
            v = gen.vonmisesvariate(mu, kappa)
            assert 0.0 <= v <= random.TWOPI, f"vonmises out of range: {v!r}"

print("vonmisesvariate_stays_in_circle OK")
