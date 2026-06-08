# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "real_world"
# case = "stats_summary_pipeline"
# subject = "math"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math: a numeric-analytics pipeline drives sqrt/fsum/dist/hypot/log together: compute an extended-precision mean and population standard deviation over a sample, the Euclidean norm of a vector, and a geometric-mean via fsum of logs, asserting each deterministic result"""
import math

# A small numeric-analytics pipeline that leans on several math primitives
# together, the way a downstream consumer would.
sample = [2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]
n = len(sample)

# Extended-precision mean and population standard deviation.
mean = math.fsum(sample) / n
variance = math.fsum((x - mean) ** 2 for x in sample) / n
stddev = math.sqrt(variance)
assert mean == 5.0, f"mean = {mean!r}"
assert stddev == 2.0, f"stddev = {stddev!r}"

# Euclidean norm of a vector, two ways: dist-from-origin and hypot.
vec = [3.0, 4.0, 12.0]
norm_dist = math.dist(vec, [0.0, 0.0, 0.0])
norm_hypot = math.hypot(*vec)
assert norm_dist == 13.0, f"norm via dist = {norm_dist!r}"
assert norm_hypot == 13.0, f"norm via hypot = {norm_hypot!r}"

# Geometric mean of positive values via fsum of logs (numerically stable).
vals = [1.0, 2.0, 4.0, 8.0]
geo = math.exp(math.fsum(math.log(v) for v in vals) / len(vals))
assert abs(geo - math.sqrt(8.0)) < 1e-9, f"geometric mean = {geo!r}"

print("stats_summary_pipeline OK")
