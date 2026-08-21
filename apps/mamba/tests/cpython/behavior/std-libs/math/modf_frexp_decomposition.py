# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "behavior"
# case = "modf_frexp_decomposition"
# subject = "math.modf"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""math.modf: modf(3.75) splits into (0.75, 3.0) fractional/integral float parts and frexp(8.0) returns the (mantissa, exponent) pair (0.5, 4)"""
import math

frac_part, int_part = math.modf(3.75)
assert frac_part == 0.75, f"modf frac = {frac_part!r}"
assert int_part == 3.0, f"modf int = {int_part!r}"
assert isinstance(frac_part, float) and isinstance(int_part, float), "modf parts are float"
m, e = math.frexp(8.0)
assert m == 0.5, f"frexp mantissa = {m!r}"
assert e == 4, f"frexp exponent = {e!r}"

print("modf_frexp_decomposition OK")
