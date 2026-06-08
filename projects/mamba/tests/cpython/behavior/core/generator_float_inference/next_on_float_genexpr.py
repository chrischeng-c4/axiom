# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "generator_float_inference"
# dimension = "behavior"
# case = "next_on_float_genexpr"
# subject = "next() drawing successive floats from a generator expression"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""next() pulled from a float generator expression must return the correct float at each step."""

gen = (i * 0.25 for i in range(4))
first = next(gen)
second = next(gen)
assert isinstance(first, float) and isinstance(second, float), (first, second)
assert first == 0.0, first
assert second == 0.25, second
assert next(gen) == 0.5
assert next(gen) == 0.75
print("next_on_float_genexpr OK")
