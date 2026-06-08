# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "numbers"
# dimension = "surface"
# case = "api_number_is_present"
# subject = "numbers.Number"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""numbers.Number: api_number_is_present (surface)."""
import numbers

assert hasattr(numbers, "Number")
print("api_number_is_present OK")
