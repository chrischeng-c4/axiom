# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "generator_float_local_accumulator"
# subject = "generator with a float local accumulator carried across yield"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A generator carrying a float local accumulator across yields must yield correct running totals."""


def running_total(values):
    acc = 0.0
    for v in values:
        acc += v
        yield acc


result = list(running_total([0.5, 1.5, 2.0]))
assert result == [0.5, 2.0, 4.0], result
assert all(isinstance(x, float) for x in result), result
print("generator_float_local_accumulator OK")
