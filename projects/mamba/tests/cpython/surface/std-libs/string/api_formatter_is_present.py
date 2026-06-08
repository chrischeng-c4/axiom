# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "api_formatter_is_present"
# subject = "string.Formatter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""string.Formatter: api_formatter_is_present (surface)."""
import string

assert hasattr(string, "Formatter")
print("api_formatter_is_present OK")
