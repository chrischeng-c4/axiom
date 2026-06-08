# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "comb_perm"
# subject = "math.comb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.comb: binomial coefficient and permutations: comb(5, 2)==10, comb(10, 0)==1, perm(5, 2)==20"""
import math

assert math.comb(5, 2) == 10, f"comb(5,2) = {math.comb(5, 2)!r}"
assert math.comb(10, 0) == 1, f"comb(10,0) = {math.comb(10, 0)!r}"
assert math.perm(5, 2) == 20, f"perm(5,2) = {math.perm(5, 2)!r}"

print("comb_perm OK")
