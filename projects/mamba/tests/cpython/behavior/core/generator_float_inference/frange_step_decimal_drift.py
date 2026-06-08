# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "frange_step_decimal_drift"
# subject = "float-param generator with binary-inexact step reproduces CPython drift exactly"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A frange-style float generator with a 0.1 step must reproduce CPython's binary float drift exactly."""


def frange(start, stop, step):
    while start <= stop:
        yield start
        start += step


# 0.1 is not exactly representable in binary float; after three additions the
# accumulator is 0.30000000000000004, which is > 0.3, so the loop stops before
# yielding a fourth value. The pinned list captures CPython's exact drift.
result = list(frange(0.0, 0.3, 0.1))
assert result == [0.0, 0.1, 0.2], result
assert all(isinstance(x, float) for x in result), result
print("frange_step_decimal_drift OK")
