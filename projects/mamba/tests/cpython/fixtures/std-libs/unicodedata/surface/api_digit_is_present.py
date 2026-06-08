# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicodedata"
# dimension = "surface"
# case = "api_digit_is_present"
# subject = "unicodedata.digit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""unicodedata.digit: api_digit_is_present (surface)."""
import unicodedata

assert hasattr(unicodedata, "digit")
print("api_digit_is_present OK")
