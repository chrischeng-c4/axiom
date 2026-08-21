# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "isnan_isinf_isfinite"
# subject = "math.isnan"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.isnan: the float predicates classify correctly: isnan(nan) and isnan(float('nan')) are True but isnan(1.0) False; isinf(inf) and isinf(-inf) True; isfinite(1.0) True but isfinite(inf)/isfinite(nan) False"""
import math

assert math.isnan(math.nan), "isnan(nan)"
assert math.isnan(float("nan")), "isnan(float('nan'))"
assert not math.isnan(1.0), "not isnan(1.0)"
assert math.isinf(math.inf), "isinf(inf)"
assert math.isinf(-math.inf), "isinf(-inf)"
assert not math.isinf(1.0), "not isinf(1.0)"
assert math.isfinite(1.0), "isfinite(1.0)"
assert not math.isfinite(math.inf), "not isfinite(inf)"
assert not math.isfinite(math.nan), "not isfinite(nan)"

print("isnan_isinf_isfinite OK")
