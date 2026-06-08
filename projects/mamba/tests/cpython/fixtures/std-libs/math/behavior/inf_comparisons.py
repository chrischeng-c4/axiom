# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "inf_comparisons"
# subject = "math.inf"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.inf: math.inf compares as positive infinity (inf > 0, inf == float('inf')) and -math.inf == float('-inf'); nan compares unequal to itself"""
import math

assert math.inf > 0.0, "inf > 0"
assert math.inf == float("inf"), "inf == float('inf')"
assert -math.inf == float("-inf"), "-inf == float('-inf')"
assert math.isinf(math.inf), "inf is inf"
assert math.nan != math.nan, "nan != nan"

print("inf_comparisons OK")
