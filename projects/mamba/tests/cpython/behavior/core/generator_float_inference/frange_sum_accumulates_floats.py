# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "frange_sum_accumulates_floats"
# subject = "float-param generator accumulated across yield, reduced by sum()"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""frange-style generator reduced by sum(): the total of the yielded floats is the correct float."""


def frange(start, stop, step):
    while start <= stop:
        yield start
        start += step


total = sum(frange(0.0, 2.0, 0.5))
assert isinstance(total, float), type(total)
assert total == 5.0, total
print("frange_sum_accumulates_floats OK")
