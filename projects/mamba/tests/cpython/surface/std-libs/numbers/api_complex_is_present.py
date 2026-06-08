# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "surface"
# case = "api_complex_is_present"
# subject = "numbers.Complex"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""numbers.Complex: api_complex_is_present (surface)."""
import numbers

assert hasattr(numbers, "Complex")
print("api_complex_is_present OK")
