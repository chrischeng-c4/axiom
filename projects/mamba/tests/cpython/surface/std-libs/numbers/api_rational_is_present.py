# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "surface"
# case = "api_rational_is_present"
# subject = "numbers.Rational"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""numbers.Rational: api_rational_is_present (surface)."""
import numbers

assert hasattr(numbers, "Rational")
print("api_rational_is_present OK")
