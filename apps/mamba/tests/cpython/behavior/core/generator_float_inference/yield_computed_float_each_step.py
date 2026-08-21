# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "yield_computed_float_each_step"
# subject = "generator yielding a freshly computed float each step"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A generator that computes a float each step (i / 2.0) must yield the correct float values."""


def halves(n):
    for i in range(n):
        yield i / 2.0


result = list(halves(5))
assert result == [0.0, 0.5, 1.0, 1.5, 2.0], result
assert all(isinstance(x, float) for x in result), result
print("yield_computed_float_each_step OK")
