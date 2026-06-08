# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "surface"
# case = "api_printable_is_present"
# subject = "string.printable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""string.printable: api_printable_is_present (surface)."""
import string

assert hasattr(string, "printable")
print("api_printable_is_present OK")
