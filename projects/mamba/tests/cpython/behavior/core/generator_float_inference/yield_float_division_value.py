# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "yield_float_division_value"
# subject = "float division return value yielded by a generator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A generator yielding true-division (/) results must produce correct float quotients, not NaN-box bits."""


def reciprocals(values):
    for v in values:
        yield 1 / v


result = list(reciprocals([2, 4, 8]))
assert result == [0.5, 0.25, 0.125], result
assert all(isinstance(x, float) for x in result), result
print("yield_float_division_value OK")
