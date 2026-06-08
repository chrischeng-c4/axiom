# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "api_get_int_max_str_digits_is_present"
# subject = "sys.get_int_max_str_digits"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sys.get_int_max_str_digits: api_get_int_max_str_digits_is_present (surface)."""
import sys

assert hasattr(sys, "get_int_max_str_digits")
print("api_get_int_max_str_digits_is_present OK")
