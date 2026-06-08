# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "next_on_frange_yields_float"
# subject = "next() on a float-param generator returns the correct accumulated float"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""next() on a frange-style float-param generator must return the correct accumulated float each call."""


def frange(start, stop, step):
    while start <= stop:
        yield start
        start += step


gen = frange(0.0, 10.0, 2.5)
assert next(gen) == 0.0
assert next(gen) == 2.5
third = next(gen)
assert isinstance(third, float), type(third)
assert third == 5.0, third
assert next(gen) == 7.5
print("next_on_frange_yields_float OK")
