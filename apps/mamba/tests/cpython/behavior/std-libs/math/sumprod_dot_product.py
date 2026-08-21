# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "sumprod_dot_product"
# subject = "math.sumprod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.sumprod: math.sumprod is an extended-precision dot product: int operands stay int (sumprod([1,2,3],[4,5,6])==32), empty operands give 0, bool coerces like 0/1, and catastrophic 1e101 cancellation leaves 2.0 exactly"""
import math

assert math.sumprod([1, 2, 3], [4, 5, 6]) == 32, "sumprod ints"
assert isinstance(math.sumprod([1, 2, 3], [4, 5, 6]), int), "int result type"
assert math.sumprod([], []) == 0, "sumprod empty"
assert math.sumprod([0.1] * 20, [True, False] * 10) == 1.0, "bool second arg"
assert math.sumprod([True, False] * 10, [0.1] * 20) == 1.0, "bool first arg"
got = math.sumprod([1.0, 1e101, 1.0, -1e101], [1.0] * 4)
assert got == 2.0, f"cancellation = {got!r}"

print("sumprod_dot_product OK")
