# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "fsum_exact_summation"
# subject = "math.fsum"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.fsum: math.fsum sums ten 0.1 terms to exactly 1.0 where a naive float sum would drift, demonstrating extended-precision (Shewchuk) accumulation"""
import math

# Ten 0.1 terms sum to exactly 1.0 under Shewchuk accumulation.
assert math.fsum([0.1] * 10) == 1.0, f"fsum(10*0.1) = {math.fsum([0.1] * 10)!r}"
# Catastrophic cancellation: the 1e101 terms cancel exactly, leaving 2.0,
# which a single running float accumulator cannot represent mid-sum.
assert math.fsum([1.0, 1e101, 1.0, -1e101]) == 2.0, "fsum cancellation"
# Empty sum is the additive identity 0.0.
assert math.fsum([]) == 0.0, "fsum empty"

print("fsum_exact_summation OK")
