# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "surface"
# case = "api_real_is_present"
# subject = "numbers.Real"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""numbers.Real: api_real_is_present (surface)."""
import numbers

assert hasattr(numbers, "Real")
print("api_real_is_present OK")
