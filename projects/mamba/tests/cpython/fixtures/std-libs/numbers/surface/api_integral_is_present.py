# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "surface"
# case = "api_integral_is_present"
# subject = "numbers.Integral"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""numbers.Integral: api_integral_is_present (surface)."""
import numbers

assert hasattr(numbers, "Integral")
print("api_integral_is_present OK")
