# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "frange_list_accumulates_floats"
# subject = "float-param generator accumulated across yield, consumed by list()"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""frange-style generator: FLOAT params accumulated across yield must yield correct floats into list()."""


def frange(start, stop, step):
    while start <= stop:
        yield start
        start += step


result = list(frange(0.0, 1.0, 0.25))
assert result == [0.0, 0.25, 0.5, 0.75, 1.0], result
assert all(isinstance(x, float) for x in result), result
print("frange_list_accumulates_floats OK")
