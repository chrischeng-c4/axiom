# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "count_lazy_infinite"
# subject = "itertools.count"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.count: count(start[, step]) is a lazy infinite counter; first N pulls give start, start+step, ..."""
import itertools

c = itertools.count(0)
first5 = [next(c) for _ in range(5)]
assert first5 == [0, 1, 2, 3, 4], f"count 5 = {first5!r}"

stepped = itertools.count(10, 2)
assert next(stepped) == 10, "count start"
assert next(stepped) == 12, "count step"

print("count_lazy_infinite OK")
