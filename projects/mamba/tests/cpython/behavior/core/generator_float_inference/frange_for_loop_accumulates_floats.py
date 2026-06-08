# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "frange_for_loop_accumulates_floats"
# subject = "float-param generator accumulated across yield, consumed by for loop"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""frange-style generator consumed by a for loop: each yielded value is the correct accumulated float."""


def frange(start, stop, step):
    while start <= stop:
        yield start
        start += step


collected = []
for value in frange(1.0, 3.0, 0.5):
    collected.append(value)
assert collected == [1.0, 1.5, 2.0, 2.5, 3.0], collected
assert all(isinstance(x, float) for x in collected), collected
print("frange_for_loop_accumulates_floats OK")
