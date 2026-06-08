# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_float_roundtrip"
# dimension = "behavior"
# case = "float_through_function_param"
# subject = "float pulled from a container and passed through a function param"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""A float read out of a list and passed through a function param must arrive unchanged."""
def echo(x):
    return x

xs = [7.5, 8.25]
got = echo(xs[0])
assert got == 7.5, got
assert isinstance(got, float), type(got)
assert echo(xs[1]) == 8.25, echo(xs[1])
print("float_through_function_param OK")
