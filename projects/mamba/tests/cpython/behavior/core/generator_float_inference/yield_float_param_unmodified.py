# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "yield_float_param_unmodified"
# subject = "generator yielding a float parameter unchanged"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A generator that yields a float parameter unmodified must produce that exact float, not NaN-box bits."""


def repeat_float(value, times):
    for _ in range(times):
        yield value


result = list(repeat_float(3.14, 3))
assert result == [3.14, 3.14, 3.14], result
assert all(isinstance(x, float) for x in result), result
print("yield_float_param_unmodified OK")
